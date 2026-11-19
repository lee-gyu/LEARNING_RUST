# ch15. 대여 검사기 이해하기

본편에서 대여 규칙을 배웠고 E0499와 E0502를 여러 번 만났습니다. 그런데 규칙을 안다고 에러가 쉽게 풀리지는 않습니다. 에러를 만나면 `clone()`을 붙이거나, `Rc<RefCell<T>>`로 감싸거나, 참조 대신 번호를 들고 다니는 식으로 "일단 컴파일되게" 만들기 쉽습니다. 이런 임시방편은 대개 값을 쓸데없이 복제하거나, 실행 중에 실패하거나, 틀린 결과를 냅니다. 이 챕터는 대여 검사기(borrow checker)가 무엇을 막는지 표 하나로 정리한 뒤, **에러를 만났을 때 어떤 순서로 무엇을 시도할지**를 지침으로 정리합니다. 먼저 함수 안에서 풀 수 있는지 보고(지침 1~4), 안 되면 자료구조를 바꾸고(지침 5), 그래도 안 되면 스마트 포인터를 씁니다(지침 6). 자기 참조 구조체는 따로 다룹니다(지침 7). 앞 챕터들이 미뤄 둔 숙제도 함께 풉니다. [ch8](../ch8-references-pointers/README.md)의 "인덱스로 가리키기와 서로 얽힌 구조", [ch9](../ch9-iterator-transforms/README.md)의 "두 클로저가 같은 상태를 고칠 때", [ch14](../ch14-lifetimes/README.md)의 "자기 참조 구조체"입니다.

- 원제: [Item 15: Understand the borrow checker](https://effective-rust.com/borrows.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch15-borrow-checker`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 챕터의 내용을 안다고 보고 진행합니다. 규칙과 API 설명은 그쪽으로 넘깁니다.

- [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 2.8~2.10절, 4절: 대여 규칙, 마지막 사용까지만 유효한 참조, E0499·E0502
- [Part 0 ch14](../../part0-getting-started/ch14-smart-pointers/README.md) 2.7~2.11절: `Rc`, `RefCell`, `Weak`, 순환 참조
- [Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md) 2.8절, [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.5·2.10절: `Option::take`/`replace`, `mem::swap`/`take`/`replace`, `RefCell`의 대여 가드
- [Part 1 ch4](../../part1-standard-lib/ch4-collections/README.md) 2.10절, 4.5절: entry API, `get_disjoint_mut`
- [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 2.7절: `Cow`
- [ch8](../ch8-references-pointers/README.md) 2.6~2.7절: 스마트 포인터를 고르는 순서
- [ch11](../ch11-drop-raii/README.md) 2.3절: `Drop`이 있는 가드가 붙잡는 대여
- [ch14](../ch14-lifetimes/README.md) 2.3·2.8절: 값·참조·임시값이 끝나는 곳, 뷰 타입과 소유하는 타입

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 소유자·불변 참조·가변 참조가 각각 할 수 있는 일과, 대여가 살아 있는 동안 소유자도 할 수 없게 되는 일을 설명하고, 대여 검사기의 에러 코드(E0499, E0502, E0503, E0505, E0506, E0507)를 어느 규칙을 어긴 것인지 연결할 수 있다.
2. 에러 메시지의 세 표시(빌린 곳, 충돌한 곳, 그 뒤에 쓰는 곳)를 읽고, 함수 안의 리팩터링(대여 범위 줄이기, 체인 쪼개기, `take`/`replace`로 바꿔 끼우기, 필드·슬라이스 나눠 빌리기)으로 에러를 풀 수 있다.
3. 여러 곳이 같은 데이터를 가리켜야 할 때 복제판·인덱스판·`Rc<RefCell<T>>`판의 비용(사본이 어긋남, 삭제 뒤 인덱스가 어긋남, 실행 중 대여 실패)을 설명하고, 세대 ID나 `Weak`로 약점을 막을 수 있다.
4. 대여 검사기 에러를 덮으려고 쓴 `clone()`과 `RefCell`을 알아보고, 데이터를 돌려주는 구조로 바꿔 쓸 수 있다.
5. 자기 참조 구조체가 왜 만들어지지 않는지 설명하고, 원문과 뷰를 나누거나 범위를 감추는 설계로 대신할 수 있다.

## 2. 핵심 개념

2.1·2.2절은 대여 검사기가 무엇을 검사하는지 정리하는 **이해** 부분이고, 2.3~2.9절은 에러를 만났을 때의 **지침**입니다. 지침은 "문제 상황 → 비권장 코드 → 권장 코드 → 지침을 따르지 않아도 되는 경우" 순서로 씁니다. 비권장 코드는 대부분 에러를 없애려고 쓴 임시방편입니다. 컴파일되지 않는 출발점 코드는 `compile_fail/`에 두고(4절), 컴파일되지만 비용이 있거나 틀린 결과를 내는 코드는 `examples/`에서 권장 코드와 함께 실행합니다.

### 2.1 누가 무엇을 할 수 있나: 접근 권한과 에러 코드

값에 닿는 길은 소유자, 불변 참조 `&T`(책은 공유 참조, shared reference라고 부릅니다), 가변 참조 `&mut T` 셋입니다. 길마다 할 수 있는 일이 다르고, 대여가 살아 있는 동안에는 소유자도 제약을 받습니다. 아래 두 표의 에러 코드는 모두 1.100.0에서 같은 모양의 코드를 컴파일해 확인했습니다.

| 하려는 일 | 소유자 | `&T`를 거쳐 | `&mut T`를 거쳐 |
| --- | --- | --- | --- |
| 읽기 | 됨 | 됨 | 됨 |
| 고치기 | 됨(`let mut`) | 안 됨(E0596). 내부 가변성은 예외 | 됨 |
| 옮기기(이동) | 됨 | 안 됨(E0507) | 안 됨(E0507). 다른 값을 넣으며 꺼내기만(2.5절) |
| drop | 됨 | 안 됨 | 덮어써서 옛 값을 버리는 것만 |

| 대여가 살아 있는 동안 소유자가 | `&` 대여 중 | `&mut` 대여 중 |
| --- | --- | --- |
| 읽기 | 됨 | 값을 직접 쓰면 E0503, 빌려 읽으면(`println!`, `len()`) E0502 |
| 대입으로 덮어쓰기 | E0506 | E0506 |
| 메서드로 고치기(`push` 등) | E0502 | E0499 |
| 옮기기, `drop(x)` | E0505 | E0505 |
| `&`로 또 빌리기 | 됨 | E0502 |
| `&mut`로 또 빌리기 | E0502 | E0499 |

세 가지만 짚습니다.

- **규칙은 하나입니다.** "값을 바꾸는 길이 열려 있으면 다른 길은 모두 닫히고, 읽는 길이 열려 있으면 바꾸는 길이 닫힌다." 소유자도 예외가 아닙니다. 대여 검사기는 소유자의 읽기·쓰기·이동을 그 자리에서 잠깐 빌렸다 돌려주는 것처럼 검사하므로, 빌려준 동안에는 소유자도 막힙니다. 이 규칙이 막는 것은 해제되었거나 다른 곳으로 옮겨진 메모리를 읽는 일(`push`가 버퍼를 다시 할당하는 경우, [Part 0 ch4](../../part0-getting-started/ch4-ownership/README.md) 2.9절)과, 읽는 쪽 모르게 값이 바뀌는 일입니다. 스레드 사이에서는 같은 규칙이 데이터 경쟁을 막습니다(`ch17-shared-state-parallelism`).
- **"살아 있다"는 마지막 사용까지입니다.** 대여는 중괄호가 아니라 마지막으로 쓴 곳에서 끝납니다(마지막 사용에서 끝나는 대여, NLL. [ch14](../ch14-lifetimes/README.md) 2.3절). 그래서 위 표의 에러는 모두 "빌림 → 충돌 → 그 뒤에 빌린 것을 씀"의 세 지점이 있어야 납니다. 예외는 빌린 것을 품은 값이 `Drop`을 구현한 경우로, 그 값의 스코프 끝까지 대여가 이어집니다([ch11](../ch11-drop-raii/README.md) 2.3절).
- **E0507은 대여 규칙이 아니라 접근 권한의 에러입니다.** 값을 옮길 수 있는 것은 소유자뿐입니다. 빌린 쪽이 옮겨 가면 빌려준 쪽에 빈자리가 남기 때문입니다.

소유자 쪽 에러 중 본편에 예제가 없던 E0506과 E0503은 4.1·4.2절에 있습니다. E0505는 [ch14](../ch14-lifetimes/README.md) 4.1절에, 이 챕터에서는 자기 참조 구조체의 모양으로 4.5절에 있습니다.

### 2.2 에러를 만났을 때: 세 표시를 읽고, 싼 것부터 시도한다

**세 표시를 찾습니다.** 대여 검사기 에러에는 거의 언제나 표시가 셋 있습니다. 4.1절의 E0506에서 빈 줄을 빼고 ①②③ 표시를 덧붙여 옮겼습니다.

```text
12 |     let first = &checklist[0]; // checklist를 빌림
   |                  --------- `checklist` is borrowed here                     ← ① 빌린 곳
14 |     checklist = vec![String::from("우산")]; // E0506: 빌려준 값을 덮어씀
   |     ^^^^^^^^^ `checklist` is assigned to here but it was already borrowed  ← ② 충돌한 곳
16 |     println!("맨 위 항목: {first}"); // 빌린 것을 그 뒤에 씀
   |                            ----- borrow later used here                     ← ③ 그 뒤에 쓰는 곳
```

푸는 길은 셋 중 하나를 바꾸는 것입니다. ③을 ② 앞으로 당기거나(지침 1), ①이 오래 빌리지 않게 필요한 값만 꺼내거나(지침 1·3), ①과 ②가 서로 다른 부분을 빌리게 합니다(지침 4). 셋 다 안 되면 코드가 아니라 자료구조의 문제입니다(지침 5~7). 책도 에러 메시지를 처음부터 차근차근 읽는 것을 첫 조언으로 꼽습니다.

**도움말(`help:`)은 마지막에 봅니다.** 이 챕터에서 만난 도움말 중 여럿은 문제를 다른 곳으로 옮기거나 비용을 숨깁니다.

| 에러 | 1.100.0의 도움말 | 따르면 | 절 |
| --- | --- | --- | --- |
| E0507 `&mut self` 뒤의 필드를 옮김 | `&self.calling`으로 빌리기, (`Clone`이었다면) 복제 | 빌리면 E0308, 복제하면 호출마다 사본 | 2.5 |
| E0505 빌려준 값을 옮김 | `&v.clone()` | E0716 ([ch14](../ch14-lifetimes/README.md) 4.1절) | — |
| E0499 `for` 안에서 `&mut self` 메서드 | 인덱스 반복문, 고칠 것을 따로 모으기 | 컴파일되지만 더 나은 방법이 있음 | 2.6 |
| E0502 참조를 담은 구조체가 있는데 원본을 고침 | `split_at_mut` | 이 상황과 관계없음 | 2.7 |
| E0716 임시값을 빌린 결과를 다음 문장에서 씀 | `let binding = ...` | 맞는 제안 | 2.4 |

**싼 것부터 시도합니다.** 책은 함수 안의 리팩터링, 자료구조 설계, 스마트 포인터, 자기 참조의 순서로 다룹니다. 이 순서가 그대로 시도할 순서입니다. 뒤로 갈수록 바뀌는 범위가 넓어지고 비용이 커집니다.

| 단계 | 물을 것 | 시도할 것 | 비용 | 절 |
| --- | --- | --- | --- | --- |
| 1. 함수 안에서 | 빌린 것을 더 일찍 다 쓸 수 있나 | 순서 바꾸기, 필요한 값(인덱스, 수)만 꺼내기 | 없음 | 2.3 |
| | 체인의 어느 단계가 막혔나 | 단계마다 타입을 적어 쪼개고, 원인 단계를 옮기기 | 없음 | 2.4 |
| | `&mut` 뒤에서 값을 옮기려 했나 | `take`/`replace`로 바꿔 끼우기 | 자리 표시 값 하나 | 2.5 |
| | 사실은 서로 다른 부분을 빌리나 | 필드·슬라이스를 나눠 빌리기 | 없음 | 2.6 |
| 2. 자료구조 | 여러 곳이 같은 데이터를 가리켜야 하나 | 한 곳이 소유하고 인덱스로 가리키기 | 삭제 뒤 인덱스가 어긋남 → 세대 ID | 2.7 |
| 3. 스마트 포인터 | 정말 서로 얽힌 데이터인가 | `Rc<RefCell<T>>`, 소유하지 않는 연결은 `Weak` | 실행 중 대여 실패(패닉), 카운트 | 2.8 |
| 따로 | 자기 데이터를 가리키나 | 원문과 뷰 나누기, 범위로 가리키고 원문 감추기 | 경우마다 다름 | 2.9 |

원시 포인터와 `unsafe`는 이 표에 없습니다. 책도 마지막 수단으로만 언급하며, `ch16-avoid-unsafe`에서 다룹니다.

### 2.3 지침 1: 대여는 쓰는 곳까지만 살게 한다

**문제 상황.** 여행 짐 목록에서 가장 무거운 짐을 찾아 두고, 우비를 하나 더 넣은 뒤, 찾아 둔 짐을 보고합니다.

```rust
let heaviest = bag.iter().max_by_key(|g| g.grams).unwrap(); // ① bag을 빌림
bag.push(gear("우비", 300));                                // ② E0502
println!("{}", heaviest.name);                              // ③
```

**비권장 코드.** 대여를 끊으려고 목록을 통째로 복제하고 사본에서 찾습니다.

```rust
let snapshot = bag.clone(); // 짐 4개가 모두 복제됨
let heaviest = snapshot.iter().max_by_key(|g| g.grams).unwrap();
bag.push(gear("우비", 300));
```

[`01_shrink_borrow_scope.rs`](examples/01_shrink_borrow_scope.rs)는 `Gear`의 `Clone`을 손으로 구현해 복제 횟수를 셉니다. 결과는 맞지만 `Gear`가 4번 복제됩니다. 필요한 것은 짐 하나의 이름과 무게인데 목록 크기만큼 비용을 냅니다. clippy도 경고하지 않습니다. 사본을 실제로 쓰고 있기 때문입니다.

**권장 코드.** ③에서 필요한 것을 ① 직후에 `Copy` 값으로 꺼내 두면 대여가 그 자리에서 끝납니다.

```rust
let index = bag.iter().enumerate().max_by_key(|(_, g)| g.grams).map(|(i, _)| i).unwrap();
bag.push(gear("우비", 300)); // push는 앞쪽 원소의 인덱스를 바꾸지 않음
let heaviest = &bag[index];  // 고친 뒤에 다시 빌림
```

예제의 권장 1이 이 판이고, 권장 2는 이름(`String` 하나)과 무게를 먼저 꺼낸 뒤 고칩니다. 둘 다 `Gear` 복제는 0번입니다. 인덱스를 들고 있다가 다시 빌리는 방법은 `insert`나 `remove`처럼 원소를 밀고 당기는 수정과는 함께 쓸 수 없습니다(2.7절).

범위를 줄이는 방법이 하나 더 있습니다. 블록 `{ }`으로 참조가 사는 곳을 감싸는 것인데, 참조는 마지막 사용에서 끝나므로 보통의 참조에는 거의 필요 없습니다(책도 같은 이유로 드물다고 말합니다). 블록이 필요한 것은 `Drop`을 구현한 가드이고, [ch11](../ch11-drop-raii/README.md) 2.3절에서 다뤘습니다. 반대로 `bag.push(gear(&format!("{} 커버", heaviest.name), 50))`처럼 고치는 메서드의 **인자** 안에서 빌린 것을 마지막으로 쓰는 것은 컴파일됩니다. 인자를 다 계산할 때까지는 가변 대여가 실제로 시작되지 않기 때문입니다(2단계 대여, two-phase borrow).

**NLL이 아직 받아 주지 않는 모양(책에 없는 지침).** "찾으면 그 원소의 `&mut`를, 못 찾으면 새로 넣고 그 `&mut`를" 돌려주는 함수입니다.

```rust
fn pouch_mut<'a>(pouches: &'a mut Vec<Pouch>, label: &'static str) -> &'a mut Pouch {
    if let Some(p) = pouches.iter_mut().find(|p| p.label == label) {
        return p;
    }
    pouches.push(Pouch { label, items: Vec::new() }); // E0499
    pouches.last_mut().unwrap()
}
```

```text
error[E0499]: cannot borrow `*pouches` as mutable more than once at a time
2 |     if let Some(p) = pouches.iter_mut().find(|p| p.label == label) {
  |                      ------- first mutable borrow occurs here
3 |         return p;
  |                - returning this value requires that `*pouches` is borrowed for `'a`
4 |     }
5 |     pouches.push(Pouch { label, items: Vec::new() }); // E0499
  |     ^^^^^^^ second mutable borrow occurs here
```

사람이 보기에는 `return`하지 않은 갈래에서 대여가 끝났지만, 1.100.0의 대여 검사기는 "돌려줄 수도 있는 대여"를 함수 끝까지 살아 있는 것으로 봅니다. 맵에서 같은 일을 `get_mut`으로 하면 같은 에러입니다. 맵이라면 entry API(`pouches.entry(label).or_default()`, [Part 1 ch4](../../part1-standard-lib/ch4-collections/README.md) 2.10절)가 정답이고, `Vec`이라면 찾기가 인덱스만 돌려주게 해 참조를 마지막에 한 번만 만듭니다.

```rust
let index = match pouches.iter().position(|p| p.label == label) {
    Some(i) => i,
    None => { pouches.push(Pouch { label, items: Vec::new() }); pouches.len() - 1 }
};
&mut pouches[index]
```

**지침을 따르지 않아도 되는 경우.** 그 시점의 목록을 따로 보존해야 한다면(고치기 전과 후를 비교하는 보고서 등) 복제가 곧 목적이므로 비권장이 아닙니다. 복제할 값이 작으면 복제가 가장 단순한 해결일 때도 많습니다. 복제를 지나치게 두려워하지 않는 이유는 `ch20-avoid-over-optimizing`에서 다룹니다.

### 2.4 지침 2: 막힌 체인은 쪼개 보고, 원인이 된 단계를 옮긴다

임시값을 빌린 결과를 다음 문장에서 쓰는 E0716은 도움말이 제안하는 대로 임시값을 `let`으로 묶으면 풀립니다. 원리는 [ch14](../ch14-lifetimes/README.md) 2.3절, 에러 예제는 [Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md) 4.2절에 있습니다. 이 절은 도움말이 이렇게 분명하지 않을 때입니다.

**문제 상황.** 이메일 주소에서 도메인을 소문자로 꺼내는 함수를 체인 한 줄로 씁니다.

```rust
fn domain(email: &str) -> &str {
    email.trim().to_lowercase().rsplit('@').next().unwrap_or("")
}
```

```text
error[E0515]: cannot return value referencing temporary value
2 |     email.trim().to_lowercase().rsplit('@').next().unwrap_or("")
  |     ---------------------------^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |     |
  |     returns a value referencing data owned by the current function
  |     temporary value created here
```

책의 조언대로 단계마다 타입을 적어 쪼갭니다. 고치는 것이 아니라 어느 단계가 문제인지 찾는 작업입니다.

```rust
let trimmed: &str = email.trim();                     // 입력을 빌림
let lowered: String = trimmed.to_lowercase();         // 새 값
let part: Option<&str> = lowered.rsplit('@').next();  // 새 값을 빌림
part.unwrap_or("")                                    // E0515
```

에러가 ``cannot return value referencing local variable `lowered` ``로 바뀝니다. 적어 놓은 타입을 보면 `String`은 한 단계뿐이고 나머지는 모두 빌린 값입니다. 입력을 빌리는 단계 사이에 새 값을 만드는 단계가 끼어 있어서, 그 뒤의 단계가 입력이 아니라 함수 안의 지역 값을 빌리게 된 것이 원인입니다.

**비권장 코드.** 에러가 사라지도록 언제나 `String`을 돌려줍니다(`-> String`, 끝에 `.to_string()`). [`02_split_the_chain.rs`](examples/02_split_the_chain.rs)에서 주소 4개에 `String`이 8개 생깁니다. 이미 소문자인 주소에도 `to_lowercase`와 `to_string`이 하나씩 만듭니다.

**권장 코드.** 원인 단계를 뒤로 옮깁니다. 입력을 빌리는 단계를 먼저 끝내고, 바꿀 글자가 있을 때만 새 값을 만듭니다. "빌린 것 또는 새로 만든 것"을 돌려주는 타입은 표준 라이브러리의 `Cow`입니다([Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md) 2.7절).

```rust
fn domain(email: &str) -> Cow<'_, str> {
    let part: &str = email.trim().rsplit('@').next().unwrap_or("");
    if part.chars().any(char::is_uppercase) {
        Cow::Owned(part.to_lowercase())
    } else {
        Cow::Borrowed(part)
    }
}
```

새로 만든 문자열은 대문자가 있던 주소 하나뿐입니다. 부르는 쪽이 비교만 한다면 `eq_ignore_ascii_case`로 견주면 되므로 새 값이 하나도 필요 없습니다(예제의 `raw_domain`). 책은 대여 문제를 스마트 포인터로 풀기 전에 표준 라이브러리에 맞는 도구가 있는지 먼저 찾아보라고 권하며 `Cow`를 예로 듭니다.

**지침을 따르지 않아도 되는 경우.** 결과를 오래 보관하거나 대부분의 입력이 바뀌어야 한다면, 부르는 쪽이 어차피 `into_owned()`를 부르게 되므로 처음부터 `String`을 돌려주는 편이 단순합니다([ch14](../ch14-lifetimes/README.md) 2.8절).

### 2.5 지침 3: `&mut` 뒤의 값은 옮기지 말고 바꿔 끼운다

**문제 상황.** 식당의 대기 호출판입니다. 다음 손님을 부르면서, 앞서 불렀지만 오지 않은 손님을 돌려받으려 합니다.

```rust
fn call_next(&mut self) -> Option<Party> {
    let previous = self.calling;             // E0507
    self.calling = self.waiting.pop_front();
    previous
}
```

다음 줄에서 바로 채우더라도 그 사이에 `self`에 빈자리가 생깁니다. 그 사이에 패닉이 나면 빈자리가 있는 구조체가 drop되며 이미 옮겨 간 값을 또 해제하게 되므로, 컴파일러는 그 순간을 허락하지 않습니다(4.3절, [Part 1 ch7](../../part1-standard-lib/ch7-smart-pointers-memory/README.md) 2.5절).

**비권장 코드.** 컴파일러의 제안 중 복제를 따릅니다. `let previous = self.calling.clone();`처럼 복제해서 꺼내고, 원래 자리는 따로 비우거나 덮어씁니다. 다른 도움말 `&self.calling`을 따르면 `&Option<Party>`를 돌려주게 되어 E0308입니다. [`03_swap_out_of_mut.rs`](examples/03_swap_out_of_mut.rs)에서 손님 넷 중 셋을 부르고 둘을 앉힌 뒤 근무를 마치기까지 `Party`가 5번 복제됩니다. 꺼내자마자 원래 자리를 지우므로 사본만 남는데, 처음부터 원래 값을 옮겼어야 할 일입니다.

**권장 코드.** 꺼내는 순간 그 자리에 다른 값을 채웁니다. 무엇을 채울지에 따라 도구를 고릅니다.

| 꺼낸 자리에 남길 것 | 쓸 것 | 예제 03 |
| --- | --- | --- |
| `None` | `opt.take()` | `seat` |
| `Some(새 값)` | `opt.replace(새 값)` | — |
| 그 타입의 기본값(빈 `Vec`, 0, 빈 문자열)이 뜻이 맞는 빈 상태 | `mem::take(&mut x)` | `end_shift`의 착석 목록 |
| 기본값이 없거나 뜻이 없음. 다음 상태를 직접 넣음 | `mem::replace(&mut x, 다음 값)` | `end_shift`의 근무 기록, `call_next` |
| 두 자리의 값을 맞바꿈 | `mem::swap(&mut a, &mut b)` | — |

```rust
fn call_next(&mut self) -> Option<Party> {
    let next = self.waiting.pop_front();
    mem::replace(&mut self.calling, next)
}
fn end_shift(&mut self, next: &'static str) -> (Shift, Vec<Party>) {
    let done = mem::replace(&mut self.shift, Shift::new(next)); // Shift에는 Default가 없음
    let seated = mem::take(&mut self.seated);
    (done, seated)
}
```

결과는 같고 `Party` 복제는 0번입니다. 더 짧은 쪽이 있으면 clippy가 알려 줍니다. 모두 기본으로 켜진(warn) 린트이고, 같은 모양의 코드에서 경고를 확인했습니다.

| 쓴 코드 | 린트 | 제안 |
| --- | --- | --- |
| `mem::replace(&mut v, Vec::new())` | `mem_replace_with_default` | `std::mem::take(&mut v)` |
| `mem::replace(&mut o, None)` | `mem_replace_option_with_none` | `o.take()` |
| `mem::replace(&mut o, Some(x))` | `mem_replace_option_with_some` | `o.replace(x)` |

`Drop`을 구현한 타입은 `self`를 값으로 받아도 필드를 옮길 수 없는데(E0509), 같은 도구로 풉니다([ch11](../ch11-drop-raii/README.md) 2.4절).

**지침을 따르지 않아도 되는 경우.** 원래 자리에도 값이 남아 있어야 한다면(되돌리기 기록에 옛 상태를 남기는 경우) 복제가 맞습니다. `Copy` 타입은 그냥 복사됩니다.

### 2.6 지침 4: 서로 다른 부분은 따로 빌린다 (책에 없는 지침)

**문제 상황.** 정원의 화분을 돌며 물을 줍니다. 물탱크에서 물을 덜고(`pour`), 일지에 적습니다(`note`). 둘 다 `&mut self` 메서드입니다.

```rust
for plant in &mut self.plants {
    let used = self.pour(2); // E0499: self 전체를 다시 가변으로 빌림
    plant.moisture += used * 10;
}
```

`pour`는 `plants`를 건드리지 않지만 대여 검사기는 함수의 본문이 아니라 **시그니처**만 보고 판단합니다. `&mut self`는 "구조체 전체"라는 뜻입니다(4.4절). 반면 한 함수 안에서 필드에 직접 접근하면 대여 검사기는 필드 단위로 추적합니다.

**비권장 코드.** 에러를 피하려고 복제본을 돕니다.

```rust
for mut plant in self.plants.clone() {
    let used = self.pour(2);
    plant.moisture += used * 10;               // 사본이 젖음
    self.note(format!("{} {}%", plant.name, plant.moisture));
}
```

[`04_split_borrows.rs`](examples/04_split_borrows.rs)에서 탱크의 물은 0L로 줄고 일지에는 "바질 40%"라고 적혔는데, 화분은 처음 그대로 20%입니다. 컴파일러도 clippy도 아무 말을 하지 않는, 지침 1의 복제보다 나쁜 경우입니다. 복제 비용에 더해 결과가 틀립니다.

**권장 코드.** 두 가지 방법이 있습니다.

```rust
// 권장 1: 구조 분해로 필드를 각각 빌림
let Garden { plants, tank_liters, journal } = self;
for plant in plants.iter_mut() { /* tank_liters, journal을 따로 고침 */ }

// 권장 2: 필요한 필드만 받는 함수로 쪼갬
fn pour_from(tank_liters: &mut u32, liters: u32) -> u32 { /* .. */ }
let used = Self::pour_from(&mut self.tank_liters, 2);
```

에러 메시지의 도움말이 제안하는 인덱스 반복문(`for i in 0..self.plants.len()`)도 컴파일되고 이 모양에는 clippy 경고도 없었습니다. 다만 매번 `self.plants[i]`로 다시 찾고, `pour`가 나중에 `plants`를 고치게 바뀌어도 컴파일러가 알려 주지 않습니다. 클로저는 2021 에디션부터 쓰는 필드만 캡처하므로 `self.plants.iter_mut().for_each(|p| self.journal.push(..))`도 컴파일됩니다([Part 2 ch3](../../part2-editions/ch3-rust-2021/README.md) 2.5절).

**슬라이스와 맵.** `&mut v[0]`과 `&mut v[2]`를 함께 만들면 E0499입니다. 인덱스로 빌리면 슬라이스 전체를 빌린 것으로 보기 때문입니다. 겹치지 않는 두 부분이 필요하면 표준 라이브러리가 나눠 줍니다.

| 메서드 | 나누는 방법 | 겹치거나 범위 밖이면 |
| --- | --- | --- |
| `split_at_mut(mid)` | 앞뒤 두 슬라이스 | `mid`가 길이보다 크면 패닉 |
| `split_first_mut()`, `split_last_mut()` | 첫(마지막) 원소와 나머지 | 빈 슬라이스면 `None` |
| 슬라이스의 `get_disjoint_mut([i, j])` (1.86.0) | 원소 N개 | `Err`: `there were overlapping indices` / `an index is out of bounds` |
| `HashMap::get_disjoint_mut([k1, k2])` (1.86.0) | 값 N개 | 키가 겹치면 패닉([Part 1 ch4](../../part1-standard-lib/ch4-collections/README.md) 4.5절) |

예제의 마지막 구역은 심지로 이은 두 화분(0번과 2번)의 수분을 `get_disjoint_mut`으로 함께 고칩니다.

**지침을 따르지 않아도 되는 경우.** 필드가 비공개인 공개 타입을 쓰는 쪽에서는 구조 분해를 할 수 없습니다. 이때는 타입을 만드는 쪽이 `fn parts_mut(&mut self) -> (&mut [Plant], &mut u32)`처럼 필요한 필드들을 한꺼번에 빌려주는 메서드를 두면 됩니다(같은 모양을 컴파일해 확인). 고칠 곳이 많아 복잡해지면 고칠 내용을 먼저 모은 뒤 반복이 끝나고 적용하는 방법(도움말의 둘째 제안)도 있습니다.

### 2.7 지침 5: 여러 곳이 가리키는 데이터는 한 곳이 소유하고, 나머지는 인덱스로 가리킨다

책은 "가능하면 데이터를 모두 소유하라"고 권합니다. 짧게 들여다보는 뷰에는 참조, 오래 보관하는 값에는 소유라는 판단은 [ch14](../ch14-lifetimes/README.md) 2.8절에서 다뤘습니다. 이 절은 그다음 질문입니다. **여러 자료구조가 같은 데이터를 가리켜야** 해서 소유자를 하나로 정할 수 없어 보일 때입니다.

**문제 상황.** 채팅 서비스에서 한 사용자가 여러 방에 들어가 있습니다. 사용자는 닉네임을 바꾸고, 탈퇴하기도 합니다. 방이 `&User`를 담으면(`RefRoom<'a>`) 닉네임을 바꾸는 순간 E0502이고, 도움말은 이 상황과 관계없는 `split_at_mut`을 제안합니다. 게다가 `RefRoom<'a>`는 사용자 목록과 한 구조체에 담을 수 없습니다(자기 참조, 2.9절).

**비권장 코드: 복제판.** 방마다 `User` 사본을 담습니다. 라이프타임은 사라지지만 [`05_clone_or_index.rs`](examples/05_clone_or_index.rs)에서 "산"이 "산들바람"으로 바뀐 뒤에도 등산 모임에는 "산"이 보입니다. 닉네임이 바뀔 때마다 모든 방의 사본을 찾아 고쳐야 합니다. 책의 표현대로 복제는 작고 바뀌지 않는 데이터에는 괜찮지만, 바뀌는 데이터에서는 사본끼리 어긋납니다.

**권장 코드: 인덱스판.** 사용자 목록 하나가 소유하고, 방은 번호만 담습니다.

```rust
struct IndexRoom { title: &'static str, members: Vec<usize> }
struct Chat { users: Vec<User>, rooms: Vec<IndexRoom> } // 라이프타임 없음. 함께 담을 수 있음
```

닉네임은 한 곳에서만 고치면 모든 방에 보입니다. 노드가 서로 복잡하게 가리키는 그래프도 같은 방법으로 만듭니다. 노드는 `Vec` 하나가 소유하고 간선은 번호 쌍으로 둡니다([ch8](../ch8-references-pointers/README.md) 2.7절의 예고).

**인덱스의 비용: 삭제.** 인덱스는 책의 말로 "가짜 포인터"입니다. 컴파일러는 번호가 무엇을 가리키는지 모릅니다. [`06_stale_index.rs`](examples/06_stale_index.rs)에서 바다가 탈퇴해 `users.remove(1)`을 부르면 뒤의 원소가 당겨집니다.

```text
  바다 탈퇴 뒤
    등산 모임: 하늘, (없는 번호 2)
    독서 모임: 산들바람, 하늘
  별 가입 뒤
    등산 모임: 하늘, 별
```

독서 모임에 들어온 적 없는 산들바람이 보이고, 새로 가입한 별이 남의 방에 들어가 있습니다. 컴파일러도 clippy도 알려 주지 않으며, 05처럼 `users[i]`로 읽었다면 범위 밖 번호에서 패닉합니다. 메모리는 안전하지만 결과는 틀립니다. 댕글링 포인터가 했을 일을 논리 오류로 바꿔 놓은 것입니다.

**권장 코드: 세대를 붙인 ID(책에 없는 지침).** 칸을 지우지 않고 비워 두어 다른 칸의 번호가 바뀌지 않게 하고, 칸마다 세대 번호를 셉니다. 탈퇴하면 세대가 1 올라가므로, 그 칸을 가리키던 옛 ID는 더 이상 맞지 않습니다.

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
struct UserId { slot: usize, generation: u32 }

fn get(&self, id: UserId) -> Option<&User> {
    let s = self.slots.get(id.slot)?;
    if s.generation == id.generation { s.user.as_ref() } else { None }
}
```

예제의 권장 판에서 탈퇴한 바다는 "(탈퇴한 사용자)"로 보이고, 별이 바다가 쓰던 칸 1을 다시 써도(세대 1) 바다의 옛 ID(세대 0)로는 별을 찾지 못합니다. 같은 칸을 다시 쓰는 것이 가능하므로 메모리도 늘지 않습니다. 이 설계를 일반화한 크레이트로 `slotmap`(crates.io에서 1.1.1 확인)이 있습니다.

**지침을 따르지 않아도 되는 경우.**

- 작고 바뀌지 않는 데이터(코드표, 설정값)라면 복제가 가장 단순합니다.
- 원소를 지우지 않고 덧붙이기만 하는 목록이라면 평범한 인덱스로 충분합니다. 세대가 지키는 것은 "지운 뒤 재사용"뿐입니다.

### 2.8 지침 6: 서로 얽힌 데이터에만 `Rc<RefCell<T>>`를 쓰고, 소유하지 않는 연결은 `Weak`로 둔다

스마트 포인터를 고르는 순서(소유자 수 → 스레드 → 내부 가변성 → 순환)는 [ch8](../ch8-references-pointers/README.md) 2.6~2.7절에서 다뤘습니다. 이 절은 같은 채팅 서비스를 `Rc<RefCell<User>>`로 만든 세 번째 판으로 비용을 비교하고, 대여 에러를 덮으려고 쓰는 `RefCell`을 봅니다.

**문제 상황.** 사용자 명부가 `Rc<RefCell<User>>`를 소유하고 방이 사용자를 가리킵니다. 사본이 없으니 닉네임 변경이 모든 방에 보이고, 번호가 아니라 포인터를 담으니 명부에서 `remove`해도 어긋날 번호가 없습니다.

**비권장 코드 1.** 방도 `Rc`를 담습니다. [`07_rc_refcell_links.rs`](examples/07_rc_refcell_links.rs)에서 바다를 명부에서 빼도 독서 모임에 그대로 보이고, 바다를 붙잡은 강한 참조가 1개 남습니다. 탈퇴한 사용자가 방이 사라질 때까지 살아 있습니다.

**권장 코드.** 사용자의 수명은 명부가 정하고, 방은 `Weak`로 가리키기만 합니다. 바다의 강한 참조는 0개가 되고 방에는 "(탈퇴한 사용자)"가 보입니다. 어느 연결을 `Weak`로 둘지는 "이 연결이 끊기면 상대가 사라져야 하는가"로 정합니다(ch8 2.7절).

**비용: 대여 규칙 위반이 실행 중 실패가 됩니다.** `RefCell`은 대여 규칙을 컴파일할 때가 아니라 실행 중에 검사합니다(Part 0 ch14 2.8절). 예제의 3번 구역은 방의 회원 이름을 화면에 그리는 동안(`member.borrow()`를 쥔 채) 새 메시지 알림 처리기가 같은 사용자의 안 읽은 수를 고칩니다.

```text
  Ref를 쥔 채 알림
    하늘: 알림 실패 (RefCell already borrowed)
    산들바람: 알림 실패 (RefCell already borrowed)
```

예제는 `try_borrow_mut()`으로 실패를 관찰했지만, 보통처럼 `borrow_mut()`을 썼다면 패닉입니다. 인덱스판에서 같은 실수(`&User`를 쥔 채 `&mut` 메서드 호출)를 하면 E0502로 컴파일되지 않습니다. 해결도 지침 1과 같습니다. 필요한 값만 꺼내 대여를 끝낸 뒤 고칩니다. 바뀌는 칸이 숫자 하나라면 그 칸만 `Cell`로 두는 방법도 있습니다(연습 4). 패닉을 피하는 일반적인 방법은 `ch18-dont-panic`에서, 스레드를 넘는 짝인 `Arc<Mutex<T>>`와 교착 상태는 `ch17-shared-state-parallelism`에서 다룹니다.

| 같은 시나리오의 세 판 | 닉네임 변경 | 탈퇴 | 대여 검사 | 예제 |
| --- | --- | --- | --- | --- |
| 복제판 | 사본이 어긋남 | 사본마다 지워야 함 | 컴파일 시 | 05 |
| 인덱스판 | 한 곳만 고침 | 번호가 어긋남. 세대 ID로 알아챔 | 컴파일 시 | 05, 06 |
| `Rc<RefCell<T>>` + `Weak`판 | 한 곳만 고침 | `upgrade()`가 `None` | **실행 중** | 07 |

**비권장 코드 2: 대여 에러를 덮으려고 쓴 `RefCell`.** [ch9](../ch9-iterator-transforms/README.md) 4.3절에서 `filter`와 `filter_map`의 클로저가 둘 다 기록 `notes`에 `push`하면 E0499였습니다. 어댑터가 두 클로저를 함께 들고 있기 때문입니다. `notes`를 `RefCell<Vec<String>>`으로 감싸면 컴파일되고 결과도 맞습니다(예제의 4번 구역). 그러나 이 코드에는 서로 얽힌 데이터가 없습니다. 대여 검사를 실행 중으로 미뤘을 뿐입니다. 반복자는 게으르므로 클로저는 `collect`가 돌 때 불리는데, 반복자를 만든 뒤 `collect` 전에 다른 코드가 `let peek = notes.borrow();`를 쥐고 있으면 클로저의 `borrow_mut()`이 `RefCell already borrowed`로 패닉합니다(같은 모양으로 확인). 컴파일러는 이 실수를 알려 주지 않습니다.

**권장 코드 2.** 상태를 고치는 대신 줄마다 결과를 **데이터로** 돌려주고, 모으는 일은 한 곳에서 합니다.

```rust
fn classify(line: &str) -> Result<&str, String> { /* 명령이면 Ok, 아니면 Err(기록할 말) */ }

let (ok, bad): (Vec<_>, Vec<_>) = lines.iter().map(|line| classify(line)).partition(Result::is_ok);
```

클로저가 아무것도 캡처하지 않으므로 대여할 것이 없고, `classify`는 따로 테스트할 수 있습니다. ch9가 권한 `for` + `match`로 되돌리기와 함께, 상태를 공유하지 않게 구조를 바꾸는 방법입니다.

**지침을 따르지 않아도 되는 경우.** GUI 요소와 이벤트 처리기처럼 여러 곳이 같은 상태를 들고 있다가 순서를 알 수 없게 고치는 구조라면 `Rc<RefCell<T>>`가 자연스럽습니다. 대여가 늘 한 문장 안에서 끝나게 쓰면 실행 중 실패의 위험도 작습니다.

### 2.9 지침 7: 자기 참조 구조체는 만들지 않는다

**문제 상황.** 셸 명령 한 줄(`String`)과 그 안의 토큰들을 한 구조체에 담고 싶습니다. 토큰마다 `String`을 만들지 않으려고 원문을 빌린 `&str`로 담습니다.

```rust
struct ParsedCommand<'a> { line: String, program: &'a str }

let program = line.split_whitespace().next().unwrap_or("");
let cmd = ParsedCommand { line, program }; // E0505
```

`program`은 지역 변수 `line`을 빌렸는데, 그 `line`을 구조체 안으로 옮기므로 E0505입니다(4.5절). 순서를 바꿔 구조체를 먼저 만들고 `cmd.program = cmd.line.split_whitespace()...`로 자기 필드를 빌려 넣으면 컴파일은 됩니다. 하지만 그 값은 자기 자신에게 빌려진 상태라, 그 뒤로 옮기려 하면 다시 E0505입니다(같은 모양으로 확인). 함수에서 돌려줄 수도, `Vec`에 넣을 수도 없는 값입니다.

이유는 두 가지입니다. 구조체를 옮기면 필드도 새 자리로 복사되므로, 참조가 필드 자체(배열처럼 구조체 안에 든 데이터)를 가리켰다면 옛 자리를 가리키게 됩니다. `String`처럼 힙을 가리키는 필드라면 글자는 그 자리에 있지만, 대여 검사기는 메모리가 아니라 소유자를 추적하므로 둘을 구별하지 않습니다([ch14](../ch14-lifetimes/README.md) 2.3절). 그리고 라이프타임 매개변수 `'a` 자리에 "이 구조체 자신"을 적을 문법이 없습니다.

**비권장 코드.** 참조 대신 바이트 범위(`Range<usize>`)로 가리키면 구조체를 옮겨도 범위가 그대로 맞습니다. 그런데 원문을 아무나 고칠 수 있게 열어 두면, 범위도 인덱스처럼 어긋납니다. [`08_self_referential.rs`](examples/08_self_referential.rs)에서 원문 앞에 `"sudo "`를 붙이면 첫 토큰이 `"su"`가 되고, 나머지 범위는 한글 글자의 중간을 가릅니다. 예제는 `str::get`으로 `None`을 받아 "(경계가 어긋남)"으로 출력했고, `&line[범위]`로 썼다면 ``end byte index 13 is not a char boundary``로 패닉합니다(같은 범위로 확인).

**권장 코드 A: 원문과 뷰를 나눕니다.** 원문은 부르는 쪽이 갖고, 토큰만 빌리는 뷰 타입을 따로 둡니다.

```rust
struct CommandView<'a> { program: &'a str, args: Vec<&'a str> }

let line = String::from(raw);
let view = CommandView::parse(&line); // 복사 없음, 컴파일러가 원문보다 오래 쓰지 못하게 막음
```

대가는 [ch14](../ch14-lifetimes/README.md) 2.8절의 뷰 타입과 같습니다. 원문이 따로 살아 있어야 하므로 "원문을 읽어 뷰를 돌려주는 함수"는 만들 수 없습니다(``cannot return value referencing local variable `line` ``, E0515).

**권장 코드 B: 범위로 가리키고 원문을 감춥니다.** 원문과 범위를 함께 소유하되 원문 필드를 비공개로 두어, 원문을 바꾸는 길은 새 값을 만들며 다시 파싱하는 메서드뿐이게 합니다.

```rust
pub struct Command { line: String, spans: Vec<Range<usize>> } // 필드는 모듈 밖에서 보이지 않음

impl Command {
    pub fn parse(line: String) -> Command { /* 범위를 구함 */ }
    pub fn with_prefix(&self, word: &str) -> Command { Command::parse(format!("{word} {}", self.line)) }
}
```

이 값은 라이프타임이 없어 `Vec`에 옮겨 담고 함수에서 돌려줘도 범위가 맞습니다(예제의 `load`). 비공개 필드로 불변식을 지키는 것은 [ch22](../ch22-minimize-visibility/README.md)의 주제이기도 합니다. 토큰마다 `String`을 만드는 소유 판(`Vec<String>`)도 언제나 가능한 선택지이고, 확신이 없으면 그쪽으로 시작합니다.

**정말 자기 참조가 필요할 때.** `async` 블록이 만드는 퓨처는 자기 참조가 생길 수 있어서, 옮기지 못하게 고정하는 `Pin`이 생겼습니다([Part 1 ch13](../../part1-standard-lib/ch13-async-primitives/README.md) 2.7절). 책은 `Pin`이 까다로우니 자기 참조를 피하거나 크레이트를 쓰라고 권하며 `ouroboros`를 소개합니다. 비슷한 일을 하는 크레이트로 `self_cell`, `yoke`도 있습니다(2026-11-19에 crates.io에서 각각 0.18.5, 1.3.0, 0.8.4를 확인했고 이 저장소에서는 써 보지 않았습니다). 이런 크레이트는 내부에 `unsafe` 코드가 있습니다. `ouroboros`도 0.15.0의 건전성 문제가 보고되어(RUSTSEC-2023-0042) 0.16.0에서 고쳐진 적이 있으므로, 쓰기 전에 권고 데이터베이스를 확인합니다. 직접 `unsafe`로 만드는 것은 `ch16-avoid-unsafe`의 범위이며 이 챕터는 다루지 않습니다.

**지침을 따르지 않아도 되는 경우.** 파싱 비용이 매우 크고, 원문과 결과를 반드시 한 값으로 들고 다녀야 하며, 범위로 표현하기 어려운 구조(트리 등)라면 위의 크레이트를 검토합니다.

### 2.10 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. rustc 1.100.0에서 확인한 차이는 다음과 같습니다.

- **복제 제안이 붙습니다.** 책이 보여 주는 E0505·E0507 출력에는 복제하라는 도움말이 없습니다(E0507에는 `consider removing the dereference here`만 있음). 1.100.0에서 같은 모양의 코드(빌려준 값을 옮기기, `*참조`에서 값을 옮기기)를 컴파일하면 ``help: consider cloning the value if the performance cost is acceptable``이 함께 나옵니다. 2.2절의 표대로 이 제안은 대개 임시방편이거나 다른 에러로 이어집니다.
- **2024 에디션의 임시값 범위.** 블록의 마지막 표현식과 `if let` 조건식의 임시값이 더 일찍 버려지게 바뀌었습니다([Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md) 2.4·2.6절). 보통의 참조가 사는 범위(지침 1)는 그대로이고, 바뀐 것은 임시값이 버려지는 때입니다. 그래서 `RefCell`이나 잠금의 가드처럼 `Drop`이 있는 임시값의 대여가 더 일찍 끝나고([ch11](../ch11-drop-raii/README.md) 2.3절), 블록 밖으로 임시값의 참조를 내보내던 코드는 E0716이 됩니다(Part 2 ch4 4.3절. 지침 2처럼 `let`으로 묶어 풉니다).
- **나눠 빌리는 도구가 늘었습니다.** 슬라이스와 `HashMap`의 `get_disjoint_mut`이 1.86.0에서 안정화됐습니다(로컬 문서의 안정화 버전 표시로 확인).
- **NLL의 한계는 그대로입니다.** 조건에 따라 `&mut`를 돌려주는 함수(2.3절의 `pouch_mut`)는 1.100.0에서도 E0499입니다.
- **책에 없는 지침.** 다음은 책의 Item 15가 다루지 않는 내용입니다(2026-11-19에 온라인판에서 확인).
  - 2.3절의 "조건부로 참조를 돌려주는 함수"와 그 해결(인덱스, entry API): 직접 컴파일한 결과와 [Part 1 ch4](../../part1-standard-lib/ch4-collections/README.md) 2.10절이 근거입니다.
  - 2.5절의 `mem::take`·`Option::take`와 고르는 표: 책은 `mem::replace`와 `Option::replace`만 다룹니다. `std::mem` 문서와 clippy의 세 린트가 근거입니다.
  - 2.6절 전체(필드·슬라이스 나눠 빌리기): 책은 메서드가 `self` 전체를 빌리는 문제를 다루지 않습니다. 직접 컴파일한 결과와 표준 라이브러리 문서(`split_at_mut`, `get_disjoint_mut`)가 근거입니다.
  - 2.7절의 세대를 붙인 ID: 책은 인덱스가 삭제 뒤 어긋난다는 데까지 다룹니다. 예제 06의 실행 결과가 근거이고, `slotmap` 같은 크레이트가 같은 설계를 씁니다.
  - 2.8절의 비권장 코드 2와 데이터로 돌려주는 구조: ch9가 넘긴 판단입니다.
- **달라지지 않은 점.** 자기 참조 구조체는 여전히 안전한 Rust만으로는 만들 수 없고, `Pin`이나 크레이트가 필요합니다. 책이 권한 "블록으로 범위 줄이기는 NLL 덕분에 드물다"와 "임시값은 `let`으로 묶는다"도 그대로입니다.

### 2.11 기억할 것

- 대여 검사기 에러는 "빌린 곳, 충돌한 곳, 그 뒤에 쓰는 곳" 세 표시로 읽습니다. 도움말의 `clone()` 제안은 마지막에 따져 봅니다.
- 함수 안에서 먼저 풉니다. 쓸 값만 꺼내 대여를 끝내고, 막힌 체인은 타입을 적어 쪼개고, `&mut` 뒤의 값은 `take`/`replace`로 바꿔 끼우고, 서로 다른 필드와 원소는 나눠 빌립니다.
- 여러 곳이 같은 데이터를 가리켜야 하면 한 곳이 소유하고 인덱스로 가리킵니다. 삭제가 있으면 세대 ID로 어긋남을 알아챕니다.
- `Rc<RefCell<T>>`는 서로 얽힌 데이터에만 쓰고, 소유하지 않는 연결은 `Weak`로 둡니다. 대여 위반은 실행 중 실패가 됩니다.
- 자기 참조 구조체는 만들지 않습니다. 원문과 뷰를 나누거나, 범위로 가리키고 원문을 감춥니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행하며, 모두 종료 코드 0으로 끝나고 출력은 실행마다 같습니다.

| 파일 | 지침 | 비권장 형태 | 권장 형태 | 절 |
| --- | --- | --- | --- | --- |
| [`01_shrink_borrow_scope.rs`](examples/01_shrink_borrow_scope.rs) | 1 | 목록 통째로 복제 | 인덱스·값만 꺼내기, 인덱스로 찾기 | 2.3 |
| [`02_split_the_chain.rs`](examples/02_split_the_chain.rs) | 2 | 언제나 `String` 반환 | 원인 단계를 옮기고 `Cow` | 2.4 |
| [`03_swap_out_of_mut.rs`](examples/03_swap_out_of_mut.rs) | 3 | `clone()`으로 꺼내기 | `take`/`replace` | 2.5 |
| [`04_split_borrows.rs`](examples/04_split_borrows.rs) | 4 | 복제본을 돌며 고치기 | 필드 나눠 빌리기, 함수 쪼개기, `get_disjoint_mut` | 2.6 |
| [`05_clone_or_index.rs`](examples/05_clone_or_index.rs) | 5 | 복제판 | 인덱스판 | 2.7 |
| [`06_stale_index.rs`](examples/06_stale_index.rs) | 5 | `Vec::remove`한 인덱스판 | 세대 ID | 2.7 |
| [`07_rc_refcell_links.rs`](examples/07_rc_refcell_links.rs) | 6 | 방이 `Rc`를 담음, 클로저 사이의 `RefCell` | `Weak`, 데이터로 돌려주기 | 2.8 |
| [`08_self_referential.rs`](examples/08_self_referential.rs) | 7 | 열어 둔 범위 | 원문과 뷰 나누기, 감춘 범위 | 2.9 |

### 01_shrink_borrow_scope — 대여 범위 줄이기

짐 4개 목록에서 가장 무거운 짐을 찾아 둔 채 우비를 넣습니다. 세 판의 결과는 같고 `Gear` 복제 횟수만 다릅니다. 마지막 구역은 있던 "세면" 파우치와 새로 만든 "비상" 파우치에 짐을 넣습니다.

```sh
cargo run -p er-ch15-borrow-checker --example 01_shrink_borrow_scope
```

```text
짐 4개, 총 4700g
[비권장] 대여를 끊으려고 목록을 통째로 복제
  가장 무거운 짐: 텐트(2100g), 우비를 넣은 뒤 5개
  복제한 Gear: 4개
[권장 1] 인덱스만 꺼내 두었다가 고친 뒤 다시 빌리기
  가장 무거운 짐: 텐트(2100g), 우비를 넣은 뒤 5개
  복제한 Gear: 0개
[권장 2] 쓸 값을 먼저 꺼내고 나서 고치기
  가장 무거운 짐: 텐트(2100g), 우비를 넣은 뒤 5개
  복제한 Gear: 0개
[NLL이 받지 않는 모양] 찾거나 새로 만들어 &mut를 돌려주는 함수
  세면 파우치: ["칫솔", "치약", "수건"]
  비상 파우치: ["밴드"]
```

### 02_split_the_chain — 체인 쪼개기와 `Cow`

주소 4개에서 도메인을 꺼냅니다. 비권장 판의 "8개"는 주소마다 `to_lowercase`와 `to_string`을 한 번씩 부른 수를 센 것입니다.

```sh
cargo run -p er-ch15-borrow-checker --example 02_split_the_chain
```

```text
[비권장] 언제나 String을 돌려줌
  ["example.com", "example.com", "mail.kr", "example.com"]
  새로 만든 String: 8개 (주소마다 to_lowercase 1 + to_string 1)
[권장] 원인 단계(to_lowercase)를 뒤로 옮기고 Cow로 돌려줌
  example.com  빌림
  example.com  새로 만듦
  mail.kr      빌림
  example.com  빌림
  새로 만든 String: 1개
  example.com 주소: 3개 (새로 만든 String 0개)
```

### 03_swap_out_of_mut — `take`와 `replace`

두 판 모두 "김 호출 → 이 호출(김은 오지 않음) → 이 착석 → 박 호출 → 박 착석 → 근무 마감" 순서로 부릅니다. 비권장 판의 5번은 오지 않은 김 1번, 착석한 이와 박 2번, 근무를 마치며 목록을 복제한 2번입니다.

```sh
cargo run -p er-ch15-borrow-checker --example 03_swap_out_of_mut
```

```text
[비권장] clone()으로 꺼내기
  오지 않은 손님: Some("김"), 점심 근무 착석 2팀 ["이", "박"]
  착석 인원 7명, 대기 1팀
  Party 복제: 5번
[권장] take/replace로 바꿔 끼우기
  오지 않은 손님: Some("김"), 점심 근무 착석 2팀 ["이", "박"]
  착석 인원 7명, 대기 1팀
  Party 복제: 0번
  새 근무: 저녁 0팀, 착석 목록 0팀
```

### 04_split_borrows — 나눠 빌리기

화분 세 개에 2L씩 물을 줍니다(탱크 5L라 토마토는 1L). 비권장 판은 탱크와 일지만 바뀌고 화분은 그대로입니다.

```sh
cargo run -p er-ch15-borrow-checker --example 04_split_borrows
```

```text
[비권장] 복제본을 돌며 물 주기
  화분: 바질 20%, 로즈메리 35%, 토마토 10%
  탱크 남은 물: 0L, 일지: ["바질 40%", "로즈메리 55%", "토마토 20%"]
[권장 1] 필드를 나눠 빌리기
  화분: 바질 40%, 로즈메리 55%, 토마토 20%
  탱크 남은 물: 0L, 일지: ["바질 40%", "로즈메리 55%", "토마토 20%"]
[권장 2] 메서드를 필드 단위 함수로 쪼개기
  화분: 바질 40%, 로즈메리 55%, 토마토 20%
  탱크 남은 물: 0L, 일지: ["바질 40%", "로즈메리 55%", "토마토 20%"]
[슬라이스] 심지로 이은 두 화분의 수분을 고르게 맞추기
  바질 15%, 토마토 15%
  [1, 1]: there were overlapping indices
  [0, 3]: an index is out of bounds
  split_at_mut 뒤 로즈메리: 15%
```

### 05_clone_or_index — 복제판과 인덱스판

사용자 0 하늘, 1 바다, 2 산. 등산 모임은 `[0, 2]`, 독서 모임은 `[1, 0]`입니다. 산이 닉네임을 산들바람으로 바꿉니다.

```sh
cargo run -p er-ch15-borrow-checker --example 05_clone_or_index
```

```text
[비권장] 복제판: 방마다 User 사본
  사용자 목록: ["하늘", "바다", "산들바람"]
  등산 모임: 하늘, 산
  독서 모임: 바다, 하늘
  방에 든 사본: 4개 (닉네임이 바뀌면 모두 찾아 고쳐야 함)
[권장] 인덱스판: 사용자 목록이 소유하고 방은 번호만
  사용자 목록: ["하늘", "바다", "산들바람"]
  등산 모임: 하늘, 산들바람
  독서 모임: 바다, 하늘
```

### 06_stale_index — 삭제 뒤의 인덱스와 세대 ID

05의 인덱스판에서 바다가 탈퇴하고 별이 가입합니다. 마지막 줄은 방 명단에서 탈퇴한 ID를 `retain`으로 정리한 결과입니다.

```sh
cargo run -p er-ch15-borrow-checker --example 06_stale_index
```

```text
[비권장] Vec::remove로 탈퇴 처리한 인덱스판
  바다 탈퇴 뒤
    등산 모임: 하늘, (없는 번호 2)
    독서 모임: 산들바람, 하늘
  별 가입 뒤
    등산 모임: 하늘, 별
    독서 모임: 산들바람, 하늘
[권장] 세대를 붙인 ID
  Some("바다") 탈퇴 뒤
    등산 모임: 하늘, 산들바람
    독서 모임: (탈퇴한 사용자), 하늘
  별 가입 뒤: 별의 ID UserId { slot: 1, generation: 1 }, 바다의 옛 ID UserId { slot: 1, generation: 0 }
    등산 모임: 하늘, 산들바람
    독서 모임: (탈퇴한 사용자), 하늘
  옛 ID로 다시 탈퇴 시도: None
    정리 뒤 독서 모임: 하늘
```

### 07_rc_refcell_links — `Rc<RefCell<T>>`판과 `Weak`

두 판은 명부를 따로 만듭니다(한 명부를 함께 쓰면 `Rc`판의 방이 바다를 살려 두어 `Weak`판에도 보입니다). 3번 구역은 `Weak`판의 등산 모임으로 실행 중 대여 실패를 관찰하고, 패닉하지 않도록 `try_borrow_mut()`을 씁니다.

```sh
cargo run -p er-ch15-borrow-checker --example 07_rc_refcell_links
```

```text
[비권장] 방이 Rc를 담음
  닉네임 변경과 바다 탈퇴 뒤
    등산 모임: 하늘, 산들바람
    독서 모임: 바다, 하늘
    명부에서 뺀 바다를 붙잡은 강한 참조: 1개 (독서 모임의 Rc)
[권장] 방이 Weak를 담음
  닉네임 변경과 바다 탈퇴 뒤
    등산 모임: 하늘, 산들바람
    독서 모임: (탈퇴한 사용자), 하늘
    바다의 강한 참조: 0개
[실행 중 대여 검사] 이름을 보여 주는 동안 알림 처리기가 같은 사용자를 고침
  Ref를 쥔 채 알림
    하늘: 알림 실패 (RefCell already borrowed)
    산들바람: 알림 실패 (RefCell already borrowed)
  필요한 값만 꺼내 대여를 끝낸 뒤 알림
    하늘: 안 읽은 메시지 1
    산들바람: 안 읽은 메시지 1
[비권장] 두 클로저가 RefCell로 같은 기록을 고침
  명령 ["join 등산", "leave 독서"]
  기록 ["빈 줄", "명령 아님: 안녕하세요", "명령 아님: /join"]
[권장] 줄마다 결과를 데이터로 돌려주고, 고치는 일은 한 곳에서
  명령 ["join 등산", "leave 독서"]
  기록 ["빈 줄", "명령 아님: 안녕하세요", "명령 아님: /join"]
```

### 08_self_referential — 자기 참조의 대안

원문은 `cp 사진.jpg 백업/`입니다. 권장 B의 둘째 줄은 함수 `load`가 만들어 돌려준 값이고, 셋째 줄은 `with_prefix`로 새로 파싱한 값입니다.

```sh
cargo run -p er-ch15-borrow-checker --example 08_self_referential
```

```text
[비권장] 범위로 가리키되 원문을 고칠 수 있게 열어 둠
  고치기 전: ["cp", "사진.jpg", "백업/"]
  원문 앞에 "sudo "를 붙인 뒤: ["su", "(경계가 어긋남)", "(경계가 어긋남)"]
[권장 A] 원문과 뷰를 나눔: 원문은 부르는 쪽이 소유
  program = cp, args = ["사진.jpg", "백업/"]
[권장 B] 범위로 가리키고 원문은 감춤
  program = Some("cp"), args = ["사진.jpg", "백업/"]
  program = Some("ls"), args = ["-l", "백업/"]
  program = Some("sudo"), args = ["cp", "사진.jpg", "백업/"]
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 본편에 에러 예제가 없던 소유자 쪽 에러(E0506, E0503)와 이 챕터의 지침에 대응하는 출발점 코드를 골랐습니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 확인합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch15-borrow-checker
rustc --edition 2024 compile_fail/e0506_assign_while_borrowed.rs --out-dir /tmp/er-ch15
rustc --edition 2024 compile_fail/e0503_use_while_mutably_borrowed.rs --out-dir /tmp/er-ch15
rustc --edition 2024 compile_fail/e0507_move_out_then_replace.rs --out-dir /tmp/er-ch15
rustc --edition 2024 compile_fail/e0499_method_borrows_whole_self.rs --out-dir /tmp/er-ch15
rustc --edition 2024 compile_fail/e0505_self_referential_struct.rs --out-dir /tmp/er-ch15
```

### 4.1 빌려준 값을 소유자가 덮어씀 — E0506

파일: [`compile_fail/e0506_assign_while_borrowed.rs`](compile_fail/e0506_assign_while_borrowed.rs)

```text
error[E0506]: cannot assign to `checklist` because it is borrowed
  --> compile_fail/e0506_assign_while_borrowed.rs:14:5
   |
12 |     let first = &checklist[0]; // checklist를 빌림
   |                  --------- `checklist` is borrowed here
13 |
14 |     checklist = vec![String::from("우산")]; // E0506: 빌려준 값을 덮어씀
   |     ^^^^^^^^^ `checklist` is assigned to here but it was already borrowed
15 |
16 |     println!("맨 위 항목: {first}"); // 빌린 것을 그 뒤에 씀
   |                            ----- borrow later used here
```

- **읽는 법**: 2.2절의 세 표시 그대로입니다. 빌린 곳(12행), 대입한 곳(14행), 그 뒤에 쓰는 곳(16행)입니다.
- **원인**: 대입하면 옛 목록이 drop되므로 `first`는 해제된 메모리를 가리키게 됩니다. 소유자라도 빌려준 동안에는 덮어쓸 수 없습니다(2.1절의 둘째 표). 같은 자리에서 `checklist.push(..)`처럼 메서드로 고치면 E0502입니다.
- **해결**: `first`를 다 쓴 뒤에 덮어쓰거나, 덮어쓴 뒤에도 필요하면 그 항목만 `first.clone()`으로 꺼내 둡니다(2.3절).

### 4.2 가변으로 빌려준 값을 소유자가 읽음 — E0503

파일: [`compile_fail/e0503_use_while_mutably_borrowed.rs`](compile_fail/e0503_use_while_mutably_borrowed.rs)

```text
error[E0503]: cannot use `weight` because it was mutably borrowed
  --> compile_fail/e0503_use_while_mutably_borrowed.rs:17:16
   |
15 |     let scale = &mut weight; // weight를 가변으로 빌림
   |                 ----------- `weight` is borrowed here
16 |
17 |     let over = weight > LIMIT_GRAMS; // E0503: 가변으로 빌려준 값을 소유자가 읽음
   |                ^^^^^^ use of borrowed `weight`
18 |
19 |     *scale += 500; // 가변 참조를 그 뒤에 씀
   |     ------------- borrow later used here
```

- **읽는 법**: "use"는 값을 직접 쓰는 것(여기서는 비교)입니다. 같은 자리에서 `println!("{weight}")`처럼 값을 빌려 읽으면 메시지가 ``cannot borrow `weight` as immutable because it is also borrowed as mutable``(E0502)로 바뀝니다.
- **원인**: 가변 참조가 살아 있는 동안에는 그 참조를 거쳐 값이 언제 바뀔지 모르므로 소유자의 읽기도 막힙니다. 불변 참조만 있을 때는 읽어도 됩니다.
- **해결**: 비교를 `scale`을 만들기 전으로 옮기거나, 가변 참조를 거쳐 읽습니다(`*scale > LIMIT_GRAMS`).

### 4.3 `&mut self` 뒤의 필드를 옮겼다가 다시 채움 — E0507

파일: [`compile_fail/e0507_move_out_then_replace.rs`](compile_fail/e0507_move_out_then_replace.rs)

```text
error[E0507]: cannot move out of `self.calling` which is behind a mutable reference
  --> compile_fail/e0507_move_out_then_replace.rs:26:24
   |
26 |         let previous = self.calling; // E0507: 빌린 자리에서 옮김
   |                        ^^^^^^^^^^^^ move occurs because `self.calling` has type `Option<Party>`, which does not implement the `Copy` trait
   |
note: if `Party` implemented `Clone`, you could clone the value
  --> compile_fail/e0507_move_out_then_replace.rs:14:1
   |
14 | struct Party {
   | ^^^^^^^^^^^^ consider implementing `Clone` for this type
...
26 |         let previous = self.calling; // E0507: 빌린 자리에서 옮김
   |                        ------------ you could clone this value
help: consider borrowing here
   |
26 |         let previous = &self.calling; // E0507: 빌린 자리에서 옮김
   |                        +
```

- **읽는 법**: "behind a mutable reference"가 핵심입니다. `self`는 빌린 것이라 그 안의 값을 옮길 수 없습니다. `Option`이 `Copy`가 아니라는 설명도 붙습니다.
- **원인**: 다음 줄에서 다시 채우더라도 그 사이에 빈자리가 생깁니다(2.5절). [Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md) 4.6절의 표에 있는 "`self.item`을 그대로 돌려줌"은 꺼내기만 하는 경우이고, 이 파일은 꺼내고 바로 새 값을 채우는 경우입니다.
- **해결**: 도움말을 따르지 않습니다. `&self.calling`은 E0308이 되고, `Party`에 `Clone`을 붙여 복제하면 호출마다 사본이 생깁니다(예제 03의 비권장 판). `mem::replace(&mut self.calling, self.waiting.pop_front())`로 바꿔 끼웁니다.

### 4.4 `&mut self.필드`를 돌며 `&mut self` 메서드를 부름 — E0499

파일: [`compile_fail/e0499_method_borrows_whole_self.rs`](compile_fail/e0499_method_borrows_whole_self.rs)

```text
error[E0499]: cannot borrow `*self` as mutable more than once at a time
  --> compile_fail/e0499_method_borrows_whole_self.rs:31:24
   |
30 |           for plant in &mut self.plants {
   |           -            ----------------
   |           |            |
   |           |            first mutable borrow occurs here
   |  _________|            first borrow later used here
   | |
31 | |             let used = self.pour(2); // E0499: self 전체를 다시 가변으로 빌림
   | |                        ^^^^ second mutable borrow occurs here
32 | |             plant.moisture += used * 10;
33 | |         }
   | |_________- this for loop borrows `*self` mutably, preventing mutation within its body
   |
   = help: consider using an index-based loop instead, or collecting modifications into a separate collection
```

- **읽는 법**: 첫 대여는 `self.plants` 필드인데, 둘째 대여는 `*self` 전체입니다. "first borrow later used here"가 `for` 줄을 가리키는 것은 반복마다 다음 원소를 꺼내며 첫 대여를 다시 쓰기 때문입니다.
- **원인**: `pour`의 시그니처 `&mut self`가 구조체 전체를 요구합니다. 컴파일러는 `pour`가 `plants`를 건드리지 않는다는 것을 본문으로 확인하지 않습니다(2.6절).
- **해결**: 필드를 구조 분해해 각각 빌리거나, `pour`를 `fn pour_from(tank_liters: &mut u32, ..)`처럼 필요한 필드만 받게 쪼갭니다(예제 04). 도움말의 인덱스 반복문도 컴파일되지만, 복제본을 도는 우회는 화분을 고치지 못하는 버그가 됩니다.

### 4.5 원문과 원문을 빌린 토큰을 한 구조체에 담음 — E0505

파일: [`compile_fail/e0505_self_referential_struct.rs`](compile_fail/e0505_self_referential_struct.rs)

```text
error[E0505]: cannot move out of `line` because it is borrowed
  --> compile_fail/e0505_self_referential_struct.rs:21:31
   |
18 |     let line = String::from("cp 사진.jpg 백업/");
   |         ---- binding `line` declared here
19 |     let program = line.split_whitespace().next().unwrap_or("");
   |                   ---- borrow of `line` occurs here
20 |
21 |     let cmd = ParsedCommand { line, program }; // E0505: 빌려준 line을 구조체로 옮김
   |                               ^^^^  ------- borrow later used here
   |                               |
   |                               move out of `line` occurs here
```

- **읽는 법**: 옮긴 곳(`line`)과 그 뒤에 쓰는 곳(`program`)이 같은 줄의 같은 구조체 리터럴 안에 있습니다. 빌린 것과 빌려준 것을 한 값에 담으려 한다는 신호입니다. 빌려준 값을 옮긴 뒤 참조를 쓰는 일반적인 E0505는 [ch14](../ch14-lifetimes/README.md) 4.1절에 있습니다.
- **원인**: 자기 참조 구조체입니다. 라이프타임 `'a`에 "이 구조체 자신"을 적을 방법이 없고, 구조체를 먼저 만든 뒤 자기 필드를 빌려 넣으면 그 값은 옮길 수 없게 됩니다(2.9절).
- **해결**: 원문과 뷰를 두 값으로 나누거나(`CommandView<'a>`), 범위(`Range<usize>`)로 가리키고 원문 필드를 감춥니다(예제 08). 확신이 없으면 토큰을 `String`으로 소유합니다.

### 4.6 그 밖의 에러

| 코드 | 결과 | 다룬 곳 |
| --- | --- | --- |
| 찾으면 `return`, 못 찾으면 넣고 `&mut`를 돌려주는 함수 | E0499(`&mut`), E0502(`get` 뒤 `insert`). ``returning this value requires that `*x` is borrowed for `'a` `` | 2.3절, 예제 01 |
| 체인 한 줄로 임시값을 빌린 결과를 돌려줌 | E0515 `cannot return value referencing temporary value` | 2.4절 |
| `fn next_token(&mut self) -> Option<&str>`의 결과 둘을 함께 들고 있음 | E0499. 돌려받은 `&str`이 살아 있는 동안 `&mut self` 대여가 이어짐 | 2.6절. 범위나 소유한 값을 돌려주거나, 원문을 빌리는 반복자(`split_whitespace`)로 바꿈 |
| `&mut v[0]`과 `&mut v[2]`를 함께 만듦 | E0499, 도움말은 `split_at_mut` | 2.6절 |
| 방이 `&User`를 담은 채 사용자 목록을 고침 | E0502, 도움말은 관계없는 `split_at_mut` | 2.7절 |
| `RefCell`의 `Ref`를 쥔 채 같은 값에 `borrow_mut()` | 실행 중 패닉 `RefCell already borrowed` | 2.8절, 예제 07(`try_borrow_mut`으로 관찰) |
| `Drop`을 구현한 가드가 빌린 값을 스코프 안에서 다시 씀 | E0502 | [ch11](../ch11-drop-raii/README.md) 4.2절 |
| 체인의 두 클로저가 같은 상태를 고침 | E0499 | [ch9](../ch9-iterator-transforms/README.md) 4.3절, 이 챕터 2.8절 |

## 5. 연습 문제

### 연습 1. 가장 무거운 짐을 기억한 채 빈 항목 지우기

[`01_shrink_borrow_scope.rs`](examples/01_shrink_borrow_scope.rs)의 짐 목록에 무게가 0인 항목(`gear("메모", 0)`)을 섞고, 가장 무거운 짐을 찾은 뒤 무게 0인 항목을 `retain`으로 지우고, 찾아 둔 짐의 이름과 무게를 출력하세요. `Gear`는 한 번도 복제하지 않습니다.

<details>
<summary>힌트</summary>

`push`와 달리 `retain`은 원소를 당기므로 권장 1처럼 인덱스를 들고 있다가 다시 빌리면 다른 짐을 가리킬 수 있습니다(2.7절의 함정). 무게 0인 항목이 텐트보다 앞에 있으면 결과가 틀리는 것을 확인해 보세요. 권장 2처럼 이름(`String` 하나)과 무게를 먼저 꺼내 두거나, 지운 뒤에 찾으면 됩니다. 순서를 바꿀 수 있다면 "지운 뒤에 찾기"가 가장 간단합니다.

</details>

### 연습 2. 호출을 미루는 `skip`

[`03_swap_out_of_mut.rs`](examples/03_swap_out_of_mut.rs)의 `Waitlist`에 `fn skip(&mut self)`를 추가하세요. 호출 중인 손님을 대기열의 맨 뒤로 보내고 다음 손님을 부릅니다. 대기열이 비어 있으면 호출 중인 손님을 그대로 둡니다. `Party`를 복제하지 않습니다.

<details>
<summary>힌트</summary>

`self.waiting.is_empty()`이면 바로 돌아갑니다. 아니면 `if let Some(p) = self.calling.take() { self.waiting.push_back(p); }`로 꺼내 뒤에 넣고, `self.calling = self.waiting.pop_front();`로 다음 손님을 부릅니다. 대기열에 한 팀만 있을 때 호출 중인 손님과 그 팀이 자리를 바꾸는지 확인하세요. `take()` 대신 `let p = self.calling;`으로 쓰면 4.3절의 E0507입니다.

</details>

### 연습 3. 세대 ID 명부에 닉네임 맞바꾸기

[`06_stale_index.rs`](examples/06_stale_index.rs)의 `Directory`에 `fn get_mut(&mut self, id: UserId) -> Option<&mut User>`와 `fn swap_nicknames(&mut self, a: UserId, b: UserId) -> bool`을 추가하세요. 두 ID 중 하나라도 탈퇴한 사용자이거나 둘이 같은 칸이면 `false`를 돌려주고 아무것도 바꾸지 않습니다.

<details>
<summary>힌트</summary>

`get_mut`은 `get`과 같은 모양에 `slots.get_mut`과 `user.as_mut()`을 씁니다. `swap_nicknames`에서 `self.get_mut(a)`와 `self.get_mut(b)`를 함께 들고 있으면 E0499입니다(2.6절). `self.slots.get_disjoint_mut([a.slot, b.slot])`로 두 칸을 함께 빌리고(같은 칸이면 `Err`), 세대를 확인한 뒤 `mem::swap(&mut x.nickname, &mut y.nickname)`으로 맞바꿉니다. 두 칸의 `user`가 모두 `Some`인지 확인하는 데는 `if let (Some(x), Some(y)) = (sa.user.as_mut(), sb.user.as_mut())`가 편합니다.

</details>

### 연습 4. 실패하지 않는 알림

[`07_rc_refcell_links.rs`](examples/07_rc_refcell_links.rs)의 3번 구역에서 이름을 보여 주는 동안에도 알림이 성공하도록, `User`의 `unread`를 `Cell<u32>`로 바꾸고 `notify`가 `&User`만으로 동작하게 고치세요.

<details>
<summary>힌트</summary>

`Cell`은 안의 값을 가리키는 참조를 내주지 않고 값을 통째로 넣고 빼므로, `Ref`를 쥔 채여도 `u.unread.set(u.unread.get() + 1)`이 됩니다. `notify(member: &User)`로 바꾸고 부르는 쪽은 `notify(&shown)`처럼 `Ref`를 거쳐 넘깁니다. 바뀌는 칸만 내부 가변성으로 두는 것은 [ch8](../ch8-references-pointers/README.md) 2.7절의 지침입니다. 이제 `RefCell`은 닉네임 변경에만 필요합니다.

</details>

## 6. 참고 자료

- 기준 문서: [Item 15: Understand the borrow checker](https://effective-rust.com/borrows.html). 이 챕터는 이 Item이 다루는 범위를 따르되, 설명과 예제는 새로 구성했습니다.
- 표준 라이브러리 문서
  - [`std::mem`](https://doc.rust-lang.org/std/mem/index.html)의 `take`, `replace`, `swap`, [`Option::take`](https://doc.rust-lang.org/std/option/enum.Option.html#method.take), [`Option::replace`](https://doc.rust-lang.org/std/option/enum.Option.html#method.replace)
  - 슬라이스의 [`split_at_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut), [`get_disjoint_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.get_disjoint_mut), [`HashMap::get_disjoint_mut`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get_disjoint_mut)
  - [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html), [`std::rc::Weak`](https://doc.rust-lang.org/std/rc/struct.Weak.html), [`RefCell::try_borrow_mut`](https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.try_borrow_mut), [`std::pin`](https://doc.rust-lang.org/std/pin/index.html)
- clippy 린트: [`mem_replace_with_default`](https://rust-lang.github.io/rust-clippy/master/index.html#mem_replace_with_default), [`mem_replace_option_with_none`](https://rust-lang.github.io/rust-clippy/master/index.html#mem_replace_option_with_none), [`mem_replace_option_with_some`](https://rust-lang.github.io/rust-clippy/master/index.html#mem_replace_option_with_some)
- 크레이트(이름과 버전만 확인): [`slotmap`](https://crates.io/crates/slotmap), [`ouroboros`](https://crates.io/crates/ouroboros), [`self_cell`](https://crates.io/crates/self_cell), [`yoke`](https://crates.io/crates/yoke), 권고 [RUSTSEC-2023-0042](https://rustsec.org/advisories/RUSTSEC-2023-0042.html)
- 에러 설명: `rustc --explain E0499`, `E0502`, `E0503`, `E0505`, `E0506`, `E0507`, `E0515`, `E0716`
