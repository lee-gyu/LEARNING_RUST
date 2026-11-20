# ch8. 일반적인 컬렉션

컬렉션(collection)은 값 여러 개를 담는 자료 구조입니다. ch3의 배열과 튜플도 값 여러 개를 담지만 길이가 컴파일할 때 정해지는 반면, 이 챕터의 컬렉션은 데이터를 힙에 두기 때문에 실행 중에 늘리고 줄일 수 있습니다. Rust 프로그램에서 가장 많이 쓰는 세 가지, 벡터 `Vec<T>`, 문자열 `String`, 해시 맵 `HashMap<K, V>`를 다룹니다.

[ch4](../ch4-ownership/README.md)에서 `String`이 힙 데이터를 소유한다는 점과 한글 슬라이스의 바이트 경계를 맛보기로 보았습니다. 이 챕터에서는 그 내용을 이어받아, 컬렉션이 **요소를 소유한다**는 사실이 대여 규칙과 어떻게 맞물리는지, 그리고 `String`이 UTF-8이라서 생기는 규칙을 한글 예시로 자세히 살펴봅니다.

- 패키지: `p0-ch8-collections`
- 기준 문서: [The Book 8장 Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html)
- 확인한 툴체인: rustc 1.100.0, Edition 2024. 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `Vec::new`와 `vec!`으로 벡터를 만들어 요소를 추가·읽기·순회·수정할 수 있고, 인덱싱(`v[i]`)과 `get(i)` 중 상황에 맞는 것을 고를 수 있다. 열거형으로 여러 종류의 값을 한 벡터에 담을 수 있다.
2. 벡터가 요소를 소유한다는 점을 바탕으로, 벡터가 버려질 때 요소도 버려지는 것과 요소의 참조를 쥔 채 `push`할 수 없는 이유(재할당과 댕글링 참조)를 설명할 수 있다.
3. `String`과 `&str`의 관계를 설명하고, `push_str`·`push`·`+`·`format!`으로 문자열을 만들 수 있으며, `+`에서 소유권이 어떻게 움직이는지 설명할 수 있다.
4. `String`에 정수 인덱스를 쓸 수 없는 이유를 UTF-8로 설명하고, 바이트(`bytes`)·문자(`chars`) 단위로 순회하거나 글자 경계에 맞게 자를 수 있다.
5. `HashMap<K, V>`에 값을 넣고 찾고 순회할 수 있으며, `entry().or_insert()`로 기존 값을 바탕으로 갱신하는 코드(단어 빈도수, 최빈값)를 작성할 수 있다.

## 2. 핵심 개념

### 2.1 컬렉션은 데이터를 힙에 둡니다

| | 배열 `[T; N]` (ch3) | 벡터 `Vec<T>` | 문자열 `String` | 해시 맵 `HashMap<K, V>` |
| --- | --- | --- | --- | --- |
| 담는 것 | 같은 타입 값 N개 | 같은 타입 값 여러 개 | UTF-8 텍스트 | (키, 값) 쌍 |
| 길이 | 컴파일할 때 고정 | 실행 중에 바뀜 | 실행 중에 바뀜 | 실행 중에 바뀜 |
| 데이터 위치 | 스택(변수 안) | 힙 | 힙 | 힙 |
| 값을 찾는 방법 | 번호(인덱스) | 번호(인덱스) | 바이트 범위, 글자 순회 | 키 |

ch4의 그림에서 본 것처럼 `String` 변수에는 힙을 가리키는 포인터·길이·용량 세 칸이 들어 있습니다. `Vec<T>`도 똑같이 생겼습니다. 사실 `String`은 "항상 올바른 UTF-8인 바이트 벡터(`Vec<u8>`)"를 감싼 타입입니다. 그래서 이 챕터에서 벡터에 대해 배우는 규칙(소유, 용량, 대여)은 `String`에도 그대로 적용됩니다.

### 2.2 벡터 만들기와 요소 추가

`Vec<T>`의 `T`는 요소의 타입입니다. `Vec<i32>`는 "i32 값들을 담는 벡터"라는 뜻입니다. 이렇게 타입을 끼워 넣는 문법(제네릭)은 ch10에서 배웁니다.

```rust
let empty: Vec<i32> = Vec::new(); // 빈 벡터. 넣을 값이 없으므로 타입을 적습니다.
let numbers = vec![10, 20, 30];   // 초깃값이 있으면 vec! 매크로. Vec<i32>로 추론됩니다.
let zeros = vec![0; 4];           // [0, 0, 0, 0]

let mut words = Vec::new();       // 타입을 적지 않아도
words.push("러스트");              // 여기서 넣는 값을 보고 Vec<&str>로 추론합니다.
```

- 빈 벡터를 만들고 아무것도 넣지 않으면 컴파일러가 요소 타입을 알 수 없어 E0282(``type annotations needed for `Vec<_>` ``)가 납니다.
- `push`는 벡터를 바꾸므로 변수가 `mut`여야 합니다.

### 2.3 요소 읽기: 인덱싱과 `get`

| | `&v[i]` 또는 `v[i]` | `v.get(i)` |
| --- | --- | --- |
| 돌려주는 것 | 요소 (또는 요소의 참조) | `Option<&T>` |
| 범위 안 | 요소 | `Some(&요소)` |
| 범위 밖 | **패닉**(프로그램이 멈춤, 종료 코드 101) | `None` |

```rust
let third: &i32 = &numbers[2];   // 30
match numbers.get(99) {
    Some(n) => println!("{n}"),
    None => println!("99번 요소는 없습니다"),
}
```

범위 밖 인덱스로 `numbers[5]`를 읽으면 다음 메시지와 함께 패닉합니다.

```
index out of bounds: the len is 3 but the index is 5
```

ch3에서 배열은 `a[10]`처럼 인덱스가 상수이면 컴파일할 때 막혔지만, 벡터는 `numbers[99]`처럼 상수를 써도 컴파일됩니다. 벡터의 길이는 실행 중에 바뀌므로 컴파일러가 미리 판단하지 않습니다.

둘 중 무엇을 쓸지는 **범위 밖 인덱스가 버그인지, 일어날 수 있는 정상 상황인지**로 정합니다. 프로그램 논리상 절대 범위를 벗어날 수 없다면 인덱싱을 쓰고, 벗어나면 즉시 멈춰 버그를 드러내게 합니다. 사용자가 입력한 번호처럼 범위를 벗어날 수 있는 값이라면 `get`으로 받아 `None` 갈래에서 안내 메시지를 출력하는 식으로 처리합니다. 예제는 [`01_vec_basics.rs`](examples/01_vec_basics.rs)입니다.

### 2.4 벡터 순회

`for` 문에 무엇을 넘기느냐에 따라 요소를 빌리는 방식이 달라집니다.

| 쓰는 법 | 요소의 타입 | 할 수 있는 일 | 반복 후 `v` |
| --- | --- | --- | --- |
| `for x in &v` | `&T` | 읽기 | 그대로 사용 가능 |
| `for x in &mut v` | `&mut T` | 그 자리에서 수정 (`*x += 1`) | 그대로 사용 가능 |
| `for x in v` | `T` | 요소의 소유권을 넘겨받음 | **사용 불가** (이동함) |

```rust
let mut prices = vec![1000, 2500, 4000];
for price in &mut prices {
    *price += 500; // 참조가 가리키는 값 자체를 바꾸려면 *로 역참조합니다(ch4).
}
```

`for x in v`는 벡터의 소유권을 `for` 문에 넘기는 것이어서, 반복이 끝난 뒤 `v`를 쓰면 E0382가 납니다. 반복 중에 같은 벡터에 `push`하는 것도 막힙니다. `for x in &v`가 반복 내내 `v`를 빌리고 있기 때문입니다(4.1절). 예제는 [`02_vec_iterate.rs`](examples/02_vec_iterate.rs)입니다.

### 2.5 벡터는 요소를 소유합니다

ch4의 소유권 규칙이 벡터의 요소에도 그대로 적용됩니다.

- `v.push(x)`: `x`의 소유권이 벡터로 **이동**합니다. 이후 `x`를 쓰면 E0382입니다.
- `v.pop()`: 마지막 요소를 꺼내 `Option<T>`로 돌려줍니다. 꺼낸 값의 소유권은 받은 쪽으로 옮겨 갑니다.
- `v[0] = 새값`: 0번 칸이 새 값을 소유하게 되므로 원래 요소는 그 자리에서 drop됩니다.
- 벡터가 스코프를 벗어나 drop되면, 벡터가 소유한 요소도 **0번부터 차례로** drop됩니다. 다만 이 순서는 지금 구현의 동작일 뿐입니다. `Vec` 문서는 요소가 drop되는 순서를 보장하지 않으며 과거에 바뀐 적이 있다고 적고 있으므로, 이 순서에 기대는 코드는 쓰지 않습니다.

ch4에서 한 스코프의 변수들은 선언의 역순으로 drop되었지만, 벡터의 요소는 앞에서부터 drop된다는 점이 다릅니다. [`05_vec_drop.rs`](examples/05_vec_drop.rs)는 ch4의 관찰용 타입 `Noisy`로 이 시점들을 출력합니다.

### 2.6 요소의 참조를 쥔 채 `push`할 수 없는 이유

벡터에는 **길이(len)** 와 **용량(capacity)** 이 따로 있습니다. 길이는 실제 요소 수, 용량은 다시 할당하지 않고 담을 수 있는 요소 수입니다. 길이가 용량에 닿은 상태에서 `push`하면, 벡터는 더 큰 힙 공간을 새로 할당하고 요소를 그리로 옮긴 다음 예전 공간을 해제할 수 있습니다.

```
push 전: len 3, capacity 3 (꽉 참)

  v                         힙 (예전 공간)
  +----------+---+         +---+---+---+
  | ptr      | --|-------> | 1 | 2 | 3 |
  | len      | 3 |         +---+---+---+
  | capacity | 3 |           ^
  +----------+---+           |
  first = &v[0] -------------+

v.push(4) 후: len 4, capacity 6

  v                         힙 (새 공간)
  +----------+---+         +---+---+---+---+---+---+
  | ptr      | --|-------> | 1 | 2 | 3 | 4 |   |   |
  | len      | 4 |         +---+---+---+---+---+---+
  | capacity | 6 |
  +----------+---+          예전 공간 (해제됨)
                           +---+---+---+
  first -----------------> | ? | ? | ? |   ← 댕글링 참조
                           +---+---+---+
```

`push` 전에 만들어 둔 `first = &v[0]`이 `push` 뒤에도 쓰인다면, `first`는 해제된 메모리를 가리키는 **댕글링 참조**가 됩니다. 대여 규칙("불변 참조가 살아 있는 동안 가변 대여 불가")이 바로 이 상황을 막습니다. `push`는 `&mut self`를 받는 메서드, 즉 벡터를 가변으로 빌리는 메서드이기 때문입니다.

용량이 넉넉해서 실제로는 옮겨지지 않는 경우에도 컴파일러는 똑같이 거부합니다. 실행 중의 용량은 컴파일할 때 알 수 없으므로, "요소를 빌린 동안에는 벡터를 바꾸지 않는다"는 규칙 하나로 모든 경우를 안전하게 만드는 것입니다.

[`03_vec_push_while_borrowed.rs`](examples/03_vec_push_while_borrowed.rs)에서 용량이 0 → 4 → 8 → 16으로 늘어나는 것을 볼 수 있습니다. 이 증가 규칙은 현재 구현의 동작일 뿐 표준 라이브러리가 보장하지 않습니다. 보장되는 것은 "`len == capacity`일 때 `push`하면 재할당한다", "`vec![...]`은 요소 수만큼의 공간을 요청한다" 같은 사실입니다.

이 제약에 걸렸을 때 쓸 수 있는 방법은 다음과 같습니다.

1. **값을 복사해 둡니다.** `let first = v[0];` 요소가 `i32`처럼 `Copy`이면 대여가 생기지 않습니다.
2. **참조를 다 쓴 뒤에 `push`합니다.** 참조의 마지막 사용이 `push`보다 앞에 오도록 순서를 바꿉니다(ch4의 "참조는 마지막 사용 지점까지 유효").
3. **인덱스만 기억합니다.** `usize` 인덱스는 벡터를 빌리지 않으므로, `push` 뒤에 `v[i]`로 다시 읽습니다.
4. **복제합니다.** `String`처럼 `Copy`가 아닌 요소는 `v[0].clone()`으로 독립된 사본을 만듭니다.

### 2.7 열거형으로 여러 종류의 값 담기

벡터는 한 가지 타입만 담을 수 있습니다. 컴파일러가 요소 하나에 필요한 메모리 크기를 알아야 하고, 요소에 어떤 연산을 해도 되는지 확인해야 하기 때문입니다. 여러 종류의 값을 한 벡터에 담고 싶으면 열거형(ch6)의 배리언트로 감싸 같은 타입으로 만듭니다.

```rust
enum Cell {
    Int(i32),
    Float(f64),
    Text(String),
}

let row = vec![Cell::Text(String::from("사과")), Cell::Int(3), Cell::Float(1200.5)];
```

꺼낼 때는 `match`로 배리언트마다 처리합니다. `match`는 모든 배리언트를 다뤄야 하므로, 나중에 배리언트를 추가하면 처리하지 않은 곳을 컴파일러가 알려 줍니다. 담을 타입의 종류를 컴파일할 때 다 알 수 없다면 트레이트 객체를 쓰며, ch17에서 다룹니다. 예제는 [`04_vec_enum.rs`](examples/04_vec_enum.rs)입니다.

### 2.8 `String`과 `&str`

| | `String` | `&str` (문자열 슬라이스) |
| --- | --- | --- |
| 소유 | 힙의 UTF-8 바이트를 **소유** | 어딘가의 UTF-8 바이트를 **빌려 봄** |
| 수정 | `mut`이면 덧붙이기·지우기 가능 | 불가 |
| 크기 | 실행 중에 늘어남 | 빌린 범위로 고정 |
| 만드는 법 | `String::from("..")`, `"..".to_string()`, `String::new()` | 문자열 리터럴 `".."`, `&s`, `&s[a..b]` |
| 비슷한 관계 | `Vec<T>` | `&[T]` (ch4의 배열 슬라이스) |

Rust 언어 자체에 있는 문자열 타입은 `str` 하나뿐이고, 보통 빌린 형태인 `&str`로 씁니다. 문자열 리터럴은 프로그램 바이너리에 들어 있는 `&str`입니다. `String`은 표준 라이브러리가 제공하는 타입으로, 늘어날 수 있고 소유권을 가집니다. 둘 다 UTF-8입니다.

`&String`은 `&str`이 필요한 자리에 자동으로 바뀌어 들어갑니다(역참조 강제 변환, ch14). 그래서 ch4에서 배운 대로 문자열을 읽기만 하는 함수는 `&str`을 받고, 문자열을 저장하거나 만들어 돌려줄 때는 `String`을 씁니다.

### 2.9 문자열 이어 붙이기: `push_str`, `push`, `+`, `format!`

```rust
let mut greeting = String::new();
let name = "지영";
greeting.push_str(name);           // &str을 덧붙입니다. name은 빌려줬을 뿐이라 계속 쓸 수 있습니다.
greeting.push_str("님, 반가워요");
greeting.push('!');                // char 하나를 덧붙입니다.
```

`+` 연산자는 `String`의 메서드를 부르는데, 그 모양을 단순하게 쓰면 다음과 같습니다(실제 정의는 ch10의 제네릭과 ch19의 연산자 오버로딩을 씁니다).

```rust
fn add(self, s: &str) -> String
```

이 시그니처에서 세 가지를 읽을 수 있습니다.

1. **왼쪽은 `self`입니다.** `&`가 없으므로 왼쪽 `String`의 소유권이 이동합니다. `let s3 = s1 + &s2;` 뒤에는 `s1`을 쓸 수 없습니다.
2. **오른쪽은 `&str`입니다.** 그래서 `&s2`처럼 빌려서 씁니다. `String` 두 개를 그대로 더하는 `s1 + s2`는 E0308(``expected `&str`, found `String` ``)입니다. `s2`는 빌려주기만 했으므로 계속 쓸 수 있습니다.
3. **복사가 적습니다.** 두 문자열을 통째로 복사해 새 문자열을 만드는 것처럼 보이지만, 실제로는 `s1`의 버퍼를 넘겨받아 그 뒤에 `s2`의 내용만 덧붙입니다. 왼쪽을 이동시키는 이유가 이것입니다.

여러 개를 이을 때는 `+`보다 `format!`이 읽기 쉽습니다. `format!`은 `println!`과 같은 서식 문자열을 쓰되, 출력하는 대신 `String`을 만들어 돌려줍니다. 인자를 빌리기만 하므로 어떤 변수의 소유권도 가져가지 않습니다.

```rust
let joined = rock + "-" + &paper + "-" + &scissors; // rock은 이동합니다.
let again = format!("{paper}-{scissors}-{joined}"); // 아무것도 이동하지 않습니다.
```

| 방법 | 덧붙이는 것 | 인자의 소유권 | 결과 |
| --- | --- | --- | --- |
| `s.push_str(t)` | `&str` | 빌림 | `s`가 바뀜 |
| `s.push(c)` | `char` | 복사(`char`는 `Copy`) | `s`가 바뀜 |
| `s1 + &s2` | `&str` | `s1`은 이동, `s2`는 빌림 | 새 `String` (`s1`의 버퍼 재사용) |
| `format!(..)` | 서식에 넣은 모든 값 | 모두 빌림 | 새 `String` |

예제는 [`06_string_build.rs`](examples/06_string_build.rs)입니다.

### 2.10 `s[0]`이 안 되는 이유: UTF-8

`String`의 `len()`은 글자 수가 아니라 **바이트 수**입니다. UTF-8은 글자마다 1~4바이트를 쓰기 때문입니다.

| 글자 | UTF-8 바이트 수 |
| --- | --- |
| `A` (영문, 숫자, 기본 기호) | 1 |
| `é` (악센트가 붙은 라틴 문자, 키릴 문자 등) | 2 |
| `한` (한글 음절, 한자, 가나 등) | 3 |
| `😀` (이모지 등) | 4 |

`String::from("안녕")`의 힙에는 바이트 6개가 들어 있습니다.

```
  s = String::from("안녕")
  +----------+---+         힙
  | ptr      | --|-------> +-----+-----+-----+-----+-----+-----+
  | len      | 6 |         | 236 | 149 | 136 | 235 | 133 | 149 |
  | capacity | 6 |         +-----+-----+-----+-----+-----+-----+
  +----------+---+           0     1     2     3     4     5     ← 바이트 인덱스
                           |<----- '안' ---->|<----- '녕' ---->|
```

여기서 `s[0]`이 허용된다면 돌려줄 수 있는 것은 `236`이라는 바이트뿐입니다. 이것은 '안'의 일부일 뿐 글자가 아닙니다. 사용자가 기대하는 '안'이 아닌 값을 조용히 돌려주면 찾기 어려운 버그가 되므로, Rust는 **문자열의 정수 인덱싱을 아예 컴파일 에러(E0277)로 막습니다**(4.2절).

이유가 하나 더 있습니다. 인덱싱은 보통 즉시(상수 시간에) 끝난다고 기대합니다. 하지만 UTF-8에서 n번째 글자를 찾으려면 앞에서부터 바이트를 해석해 나가야 하므로 그 기대를 지킬 수 없습니다.

#### 바이트, 문자, 자소 클러스터

문자열을 보는 단위는 세 가지입니다. 같은 '한'이라도 완성형(코드 포인트 1개)으로 적었는지, 조합형(초성 ㅎ + 중성 ㅏ + 종성 ㄴ, 코드 포인트 3개)으로 적었는지에 따라 결과가 달라집니다.

| 단위 | 얻는 방법 | 완성형 `"한"` | 조합형 `"\u{1112}\u{1161}\u{11AB}"` |
| --- | --- | --- | --- |
| 바이트 (`u8`) | `s.bytes()`, `s.as_bytes()`, `s.len()` | 3개: 237 149 156 | 9개: 225 132 146 225 133 161 225 134 171 |
| 문자 (`char`, 유니코드 스칼라 값) | `s.chars()` | 1개: U+D55C | 3개: U+1112 U+1161 U+11AB |
| 자소 클러스터 (grapheme cluster) | 표준 라이브러리에 없음 | 1개 | 1개 |

**자소 클러스터**는 사람이 글자 하나로 인식하는 단위입니다. 두 문자열은 화면에서 똑같이 '한'으로 보이는 것이 보통이지만, `char` 개수도 다르고 `==` 비교 결과도 `false`입니다. 외부에서 받은 텍스트에는 조합형이 섞여 있을 수 있다는 점을 기억해 두세요. 자소 클러스터 단위로 나누는 기능은 표준 라이브러리에 없고, 필요하면 crates.io의 크레이트를 씁니다. (한국어에서 '자소'는 ㄱ, ㅏ 같은 자모를 가리키기도 하지만, 여기서는 grapheme의 번역어로 "사람이 보는 한 글자"를 뜻합니다.)

#### 문자열을 안전하게 다루는 방법

| 하고 싶은 일 | 방법 | 결과 (`s = "안녕하세요"`) |
| --- | --- | --- |
| 바이트 수 | `s.len()` | `15` |
| 글자(`char`) 수 | `s.chars().count()` | `5` |
| 글자 단위 순회 | `for c in s.chars()` | `안`, `녕`, ... |
| 바이트 단위 순회 | `for b in s.bytes()` | `236`, `149`, ... |
| n번째 글자 | `s.chars().nth(n)` | `nth(2)` → `Some('하')`, `nth(9)` → `None` |
| 바이트 범위 자르기 | `&s[a..b]` | `&s[0..3]` → `"안"`, 경계가 아니면 **패닉** |
| 경계를 모를 때 자르기 | `s.get(a..b)` | `get(0..1)` → `None`, `get(0..3)` → `Some("안")` |
| 경계인지 확인 | `s.is_char_boundary(i)` | `is_char_boundary(1)` → `false` |
| 첫 글자를 뺀 나머지 | `&s[c.len_utf8()..]` (`c`는 첫 글자) | `"녕하세요"` |

`chars()`, `bytes()`는 값을 하나씩 내주는 반복자(iterator)를 돌려주고, `count()`, `nth()`는 반복자의 메서드입니다. 반복자 자체는 ch13에서 배우고, 지금은 `for` 문에 넘기거나 위 표처럼 메서드를 붙여 쓰는 것으로 충분합니다. `nth(n)`도 앞에서부터 세므로 문자열이 길수록 오래 걸립니다.

예제는 [`07_string_utf8.rs`](examples/07_string_utf8.rs)입니다.

### 2.11 해시 맵 `HashMap<K, V>`

해시 맵은 키(key)와 값(value)을 짝지어 저장하고, 번호 대신 키로 값을 찾습니다. 다른 언어의 딕셔너리(Python `dict`), 맵, 연관 배열과 같은 자료 구조입니다. 해시 함수로 키가 놓일 위치를 계산하므로 요소가 많아도 키로 빠르게 찾을 수 있습니다.

`HashMap`은 프렐류드(prelude, 모든 파일에 자동으로 가져와지는 항목들)에 없으므로 ch7에서 배운 `use`로 가져와야 합니다. 빠뜨리면 E0433(``cannot find type `HashMap` in this scope``)이 나고, 컴파일러가 `use std::collections::HashMap;`을 추가하라고 알려 줍니다. `vec!` 같은 생성 매크로도 없습니다.

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();
scores.insert(String::from("청팀"), 10);
scores.insert(String::from("백팀"), 50);

match scores.get("청팀") {        // 키 타입이 String이어도 &str로 찾을 수 있습니다.
    Some(score) => println!("{score}점"),
    None => println!("없는 팀"),
}
```

- 모든 키는 같은 타입, 모든 값은 같은 타입이어야 합니다. 데이터는 벡터처럼 힙에 있습니다.
- `get`은 `Option<&V>`를 돌려줍니다. 키가 없을 수 있기 때문입니다.
- `for (key, value) in &map`으로 순회합니다. 이때 **순서는 정해져 있지 않습니다.** 넣은 순서와 다르고, 같은 프로그램이라도 실행할 때마다 달라질 수 있습니다. 이 챕터의 예제는 출력을 일정하게 보여 주려고 순회 결과를 벡터에 모아 `sort()`로 정렬한 뒤 출력합니다.

#### 소유권: 넣으면 맵이 소유합니다

`i32`처럼 `Copy`인 값은 맵에 복사되어 들어가고, `String`처럼 소유권이 있는 값은 맵으로 **이동**합니다. `map.insert(field_name, field_value)` 뒤에 `field_name`을 쓰면 E0382입니다(4.4절). 원래 값을 계속 써야 하면 `clone()`한 사본을 넣거나, `&str` 같은 참조를 넣습니다. 참조를 넣으면 맵은 값을 빌리기만 하는 대신, 맵이 쓰이는 동안 원래 값이 살아 있어야 합니다. 이 조건을 컴파일러가 확인하는 방법(라이프타임)은 ch10에서 배웁니다.

#### 값 갱신하기: 세 가지 경우

키 하나에는 값이 하나만 있습니다. 이미 값이 있는 키에 새 값을 넣을 때 무엇을 원하느냐에 따라 쓰는 방법이 다릅니다.

| 원하는 동작 | 코드 | 설명 |
| --- | --- | --- |
| 새 값으로 덮어쓰기 | `map.insert(k, v)` | 원래 값을 `Option`으로 돌려줍니다(처음이면 `None`). |
| 키가 없을 때만 넣기 | `map.entry(k).or_insert(v)` | 이미 있으면 기존 값을 그대로 둡니다. |
| 기존 값을 바탕으로 바꾸기 | `*map.entry(k).or_insert(0) += 1` | `or_insert`가 돌려준 `&mut V`로 값을 고칩니다. |

`entry(k)`는 "그 키의 자리"를 나타내는 값을 돌려주고, `or_insert(기본값)`은 자리가 비어 있으면 기본값을 넣은 뒤, 어느 경우든 그 자리 값의 가변 참조 `&mut V`를 돌려줍니다. 단어 빈도수를 세는 코드가 이 방법의 대표적인 예입니다.

```rust
let mut counts = HashMap::new();
for word in text.split_whitespace() {     // 공백으로 나눈 조각(&str)을 하나씩
    let count = counts.entry(word).or_insert(0);
    *count += 1;
} // count의 가변 대여는 반복 한 번마다 끝나므로, 다음 반복에서 다시 entry를 부를 수 있습니다.
```

`get`으로 읽고 `insert`로 다시 쓰는 방법도 동작하지만 키를 두 번 찾아야 합니다. `entry`는 한 번 찾은 자리를 그대로 수정하므로 더 간결하고 효율적입니다. [`09_word_count.rs`](examples/09_word_count.rs)에서 두 방법의 결과가 같다는 것을 확인합니다.

#### 해시 함수

기본 해시 함수는 해시 테이블을 노린 서비스 거부(DoS) 공격에 강한 알고리즘(현재 SipHash 1-3)이고, 프로그램이 실행될 때마다 무작위 시드로 초기화됩니다. 순회 순서가 실행마다 달라지는 것도 이 때문입니다. 가장 빠른 해시 함수는 아니어서, 성능이 중요하면 다른 해시 함수로 바꿀 수 있습니다. 그 방법은 트레이트(ch10)를 배운 뒤에 쓸 수 있습니다.

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_vec_basics.rs`](examples/01_vec_basics.rs) | `Vec::new`, `vec!`, `push`, 인덱싱과 `get`의 차이 | 1 |
| [`02_vec_iterate.rs`](examples/02_vec_iterate.rs) | `&v`, `&mut v`, `v`로 순회하기 | 1 |
| [`03_vec_push_while_borrowed.rs`](examples/03_vec_push_while_borrowed.rs) | 길이와 용량, 참조를 쥔 채 `push`할 수 없는 이유와 해결 방법 | 2 |
| [`04_vec_enum.rs`](examples/04_vec_enum.rs) | 열거형으로 여러 종류의 값을 한 벡터에 담기 | 1 |
| [`05_vec_drop.rs`](examples/05_vec_drop.rs) | `push`·`pop`·대입·drop에서 요소의 소유권 | 2 |
| [`06_string_build.rs`](examples/06_string_build.rs) | `String`과 `&str`, `push_str`·`push`·`+`·`format!` | 3 |
| [`07_string_utf8.rs`](examples/07_string_utf8.rs) | 바이트·문자·자소 클러스터, 슬라이스 경계 | 4 |
| [`08_hashmap_basics.rs`](examples/08_hashmap_basics.rs) | `insert`·`get`·순회·덮어쓰기·`entry`, 소유권 | 5 |
| [`09_word_count.rs`](examples/09_word_count.rs) | `entry().or_insert()`로 단어 빈도수 세기 | 5 |
| [`10_median_mode.rs`](examples/10_median_mode.rs) | The Book 연습 과제 풀이: 중앙값과 최빈값 | 1, 5 |
| [`11_pig_latin.rs`](examples/11_pig_latin.rs) | The Book 연습 과제 풀이: 피그 라틴 | 3, 4 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 4개 (4절) | 2, 3, 4, 5 |

모든 예제는 패닉 없이 끝납니다. 패닉하는 동작은 같은 상황을 `get`처럼 패닉하지 않는 메서드로 관찰하고, 패닉하는 코드는 주석으로 남겨 두었습니다. 주석을 풀어 실행해 보면 4.5절의 메시지와 종료 코드 101을 직접 볼 수 있습니다.

### 01_vec_basics — 벡터 만들기와 읽기

6번 구역의 인덱스 5는 사용자 입력처럼 "실행해 봐야 아는 값"을 흉내 낸 것입니다. `get`을 썼기 때문에 패닉하지 않고 "없음"을 출력합니다.

```sh
cargo run -p p0-ch8-collections --example 01_vec_basics
```

```
1. 아무것도 넣지 않을 빈 벡터에는 요소 타입을 적어 줍니다
    empty = [], 길이 = 0
2. vec! 매크로는 초깃값으로 벡터를 만들고, 타입은 값에서 추론합니다
    numbers = [10, 20, 30], zeros = [0, 0, 0, 0]
3. push로 끝에 추가하려면 벡터가 mut여야 합니다
    push 전: 길이 = 0
    push 후: words = ["러스트", "벡터", "연습"], 길이 = 3
4. 인덱싱 v[i]는 요소를 바로 읽습니다 (0부터 셉니다)
    third = 30, first = 10
5. get(i)는 Option<&T>를 돌려줍니다
    get(1) = Some(20)
    get(99) = None (99번 요소는 없습니다)
6. 범위 밖일 수 있는 인덱스는 get으로 안전하게 처리합니다
    0번 요소: 10
    2번 요소: 30
    5번 요소: 없음 (길이는 3)
```

### 02_vec_iterate — 벡터 순회

1번은 읽기만, 2번은 그 자리에서 수정, 4번은 소유권까지 넘겨받습니다. 4번 구역 끝의 `// println!("{names:?}");` 주석을 풀면 E0382가 나고, 컴파일러가 `for name in &names`로 바꾸라고 제안합니다.

```sh
cargo run -p p0-ch8-collections --example 02_vec_iterate
```

```
1. for x in &v: 요소를 하나씩 빌려 읽습니다
    scores = [72, 85, 90], 합계 = 247
2. for x in &mut v: 요소를 하나씩 빌려 그 자리에서 고칩니다
    prices = [1500, 3000, 4500]
3. 인덱스가 함께 필요하면 iter().enumerate()를 씁니다
    1번: 김밥
    2번: 라면
    3번: 떡볶이
4. for x in v: 벡터의 소유권까지 넘겨 요소를 하나씩 꺼냅니다
    민수님, 안녕하세요
    지영님, 안녕하세요
    반복이 끝난 뒤 names는 더 이상 쓸 수 없습니다 (소유권이 이동함)
```

### 03_vec_push_while_borrowed — 용량과 재할당

1번은 용량이 바뀌는 순간만 출력합니다. 2번은 `vec!`으로 만든 벡터의 용량이 요소 수와 같아서, 첫 `push`부터 더 큰 공간이 필요하다는 것을 보여 줍니다. 3~6번은 2.6절의 해결 방법 네 가지입니다. 용량 값(4, 8, 16, 6)은 현재 표준 라이브러리 구현의 결과이며 버전에 따라 달라질 수 있습니다.

```sh
cargo run -p p0-ch8-collections --example 03_vec_push_while_borrowed
```

```
1. 벡터는 길이(len)와 별도로 용량(capacity)을 가집니다
    시작: len = 0, capacity = 0
    1번째 push: len = 1, capacity 0 → 4
    5번째 push: len = 5, capacity 4 → 8
    9번째 push: len = 9, capacity 8 → 16
2. 용량이 꽉 찬 벡터에 push하면 요소가 새 공간으로 옮겨질 수 있습니다
    push 전: len = 3, capacity = 3
    push 후: len = 4, capacity = 6
3. 해결 방법 ①: 참조 대신 값을 복사해 둡니다
    first = 10, numbers = [10, 20, 30, 40]
4. 해결 방법 ②: 참조를 다 쓴 뒤에 push합니다
    push 전 첫 요소: 사과
    fruits = ["사과", "배", "감"]
5. 해결 방법 ③: 위치(인덱스)만 기억했다가 push 뒤에 다시 읽습니다
    1번 요소: 배
6. 해결 방법 ④: String처럼 Copy가 아닌 요소는 clone으로 독립된 사본을 만듭니다
    copy = 사과, 길이 = 5
```

### 04_vec_enum — 열거형으로 여러 타입 담기

`Vec<Cell>` 하나에 글자·정수·실수 칸을 함께 담고, `match`로 꺼냅니다. 참조(`&Cell`)에 `match`를 쓰면 배리언트 안의 값도 참조로 바인딩되므로, 3번에서 `*n`, `*x`로 값을 꺼내 더합니다.

```sh
cargo run -p p0-ch8-collections --example 04_vec_enum
```

```
1. 서로 다른 종류의 값을 열거형으로 감싸 한 벡터에 담습니다
    row = [Text("사과"), Int(3), Float(1200.5)]
2. 꺼낼 때는 match로 배리언트마다 처리합니다
    글자: 사과 (6바이트)
    정수: 3
    실수: 1200.5
3. 숫자 칸만 골라 합계를 구합니다
    숫자 칸의 합계 = 1203.5
```

### 05_vec_drop — 벡터와 요소의 소유권

`[drop]` 줄이 찍히는 위치를 코드와 나란히 놓고 보세요. 2번에서 `pop`으로 꺼낸 "c"는 새 소유자 `last`의 스코프가 끝날 때, 4번의 x, y, z는 벡터 `inner`가 버려질 때 0번부터 차례로 drop됩니다(이 순서는 지금 구현의 동작이며 문서가 보장하지는 않습니다).

```sh
cargo run -p p0-ch8-collections --example 05_vec_drop
```

```
1. push하면 값의 소유권이 벡터로 이동합니다
    v에 3개가 들어 있습니다
2. pop은 마지막 요소를 꺼내 소유권을 돌려줍니다
    꺼낸 요소: c
    [drop] c
    남은 개수: 2
3. 요소 자리에 새 값을 대입하면 원래 요소는 그 자리에서 drop됩니다
    [drop] a
    0번 요소: a2
4. 벡터가 drop되면 남은 요소가 앞에서부터 차례로 drop됩니다
    inner 길이: 3
    [drop] x
    [drop] y
    [drop] z
5. main이 끝나면 v와 그 요소도 drop됩니다
    [drop] a2
    [drop] b
```

### 06_string_build — 문자열 만들기와 이어 붙이기

4번에서 `s1 + &s2` 뒤에도 `s2`는 출력되지만 `s1`은 쓰지 않는다는 점, 5번에서 `format!` 뒤에 모든 변수가 그대로 남아 있다는 점을 확인하세요.

```sh
cargo run -p p0-ch8-collections --example 06_string_build
```

```
1. String::from과 to_string은 같은 String을 만듭니다
    a = 안녕하세요, b = 안녕하세요, a == b: true
    n = 42, 길이 = 2
2. String은 소유하는 문자열, &str은 빌려 보는 문자열입니다
    whole = 러스트 문자열, part = 러스트, literal = 리터럴도 &str
3. push_str은 &str을, push는 char 하나를 덧붙입니다
    greeting = 지영님, 반가워요!, name = 지영
4. + 연산자는 왼쪽 String의 소유권을 가져갑니다
    s3 = 안녕, 세상!, s2 = 세상!
5. 여러 개를 이을 때 +는 읽기 어렵고, format!이 편합니다
    +로 이은 결과: 가위-바위-보
    format! 결과: 바위-보-가위-바위-보
    paper, scissors, joined 모두 그대로: 바위, 보, 가위-바위-보
```

### 07_string_utf8 — UTF-8과 글자 경계

6번 구역에는 패닉하는 `&korean[0..1]` 대신 `get(0..1)`을 써서 같은 상황을 `None`으로 관찰합니다. 7번 구역의 완성형과 조합형 '한'은 터미널과 글꼴에 따라 똑같이 보이거나 조금 다르게 보일 수 있지만, 바이트 수와 `char` 개수는 항상 출력처럼 나옵니다.

```sh
cargo run -p p0-ch8-collections --example 07_string_utf8
```

```
1. len()은 글자 수가 아니라 바이트 수입니다
    "hello": 5바이트, 5글자
    "안녕하세요": 15바이트, 5글자
2. UTF-8에서는 글자마다 차지하는 바이트 수가 다릅니다
    'A' → 1바이트
    'é' → 2바이트
    '한' → 3바이트
    '😀' → 4바이트
3. String 안에 실제로 들어 있는 것은 바이트(u8)입니다
    "한".as_bytes() = [237, 149, 156]
4. bytes()와 chars()로 원하는 단위를 골라 순회합니다
    bytes(): 236 149 136 235 133 149
    chars(): 안 녕
5. n번째 글자가 필요하면 chars().nth(n)을 씁니다
    korean.chars().nth(2) = Some('하')
    korean.chars().nth(9) = None
6. 슬라이스 범위는 바이트 단위이고, 글자 경계에 맞아야 합니다
    &korean[0..3] = 안
    &korean[3..9] = 녕하
    1번 바이트가 글자 경계인가? false
    korean.get(0..1) = None
    korean.get(0..3) = Some("안")
7. 사람이 보는 한 글자가 char 하나라는 보장도 없습니다
    화면에 보이는 모양: [한] [한]
    완성형: 3바이트, char 1개
    조합형: 9바이트, char 3개
    조합형의 char: U+1112 U+1161 U+11AB
    완성형 == 조합형? false
```

### 08_hashmap_basics — 해시 맵 기본

3번과 5번은 순회 결과를 `Vec<String>`에 모아 정렬한 뒤 출력하므로 항상 같은 순서로 나옵니다. 정렬하지 않고 바로 출력하면 실행할 때마다 순서가 달라질 수 있습니다. 4번은 `insert`가 덮어쓴 원래 값을 `Some(10)`으로 돌려주는 것을 보여 줍니다.

```sh
cargo run -p p0-ch8-collections --example 08_hashmap_basics
```

```
1. new로 만들고 insert로 (키, 값)을 넣습니다
    팀 수: 2
2. get은 키로 값을 찾아 Option<&V>로 돌려줍니다
    청팀: 10점
    홍팀: 없는 팀입니다
3. for (키, 값) in &맵 형태로 순회합니다 (순서는 정해져 있지 않습니다)
    ["백팀=50", "청팀=10"]
4. 같은 키로 다시 insert하면 값이 덮어써집니다
    원래 값: Some(10), 새 값: Some(25)
    팀 수: 2
5. entry().or_insert()는 키가 없을 때만 넣습니다
    ["백팀=50", "청팀=25", "홍팀=30"]
6. insert하면 String 키와 값의 소유권이 맵으로 이동합니다
    좋아하는 색: 파랑
7. 원래 String을 계속 써야 하면 clone한 사본이나 참조를 넣습니다
    city = 부산
    민수: Some("부산"), 지영: Some("부산")
```

### 09_word_count — 단어 빈도수

점심 메뉴 투표 문자열에서 메뉴별 표 수를 셉니다. 맵의 키는 `text`의 일부를 가리키는 `&str`이므로, 맵은 `text`를 빌리고 있습니다. (단어, 개수) 쌍을 벡터에 모아 정렬하면 튜플이 첫 요소부터 비교되어 가나다순으로 나옵니다.

```sh
cargo run -p p0-ch8-collections --example 09_word_count
```

```
1. entry().or_insert(0)으로 단어마다 개수를 셉니다
    김밥: 4표
    떡볶이: 2표
    라면: 2표
    순대: 1표
2. 가장 많이 나온 단어를 찾습니다
    1위: 김밥 (4표)
3. entry 없이 get과 insert로도 셀 수 있지만, 키를 두 번 찾게 됩니다
    두 방법의 결과가 같은가? true
```

### 10_median_mode — 중앙값과 최빈값 (The Book 연습 과제 풀이)

정수 목록의 중앙값(정렬했을 때 가운데 값)과 최빈값(가장 많이 나온 값)을 구합니다. 중앙값은 `to_vec()`으로 복사한 벡터를 `sort()`로 정렬해 구하고, 개수가 짝수면 가운데 두 값의 평균을 씁니다. 최빈값은 해시 맵으로 개수를 센 뒤, 공동 1위를 모두 모아 정렬해서 돌려줍니다. 맵의 순회 순서에 따라 결과가 바뀌지 않도록 하기 위해서입니다. 빈 목록은 중앙값이 없으므로 `Option`의 `None`으로 표현합니다.

```sh
cargo run -p p0-ch8-collections --example 10_median_mode
```

```
1. 개수가 홀수인 목록
    목록: [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5]
    중앙값: Some(4.0)
    최빈값: [5]
2. 개수가 짝수이고 최빈값이 둘인 목록
    목록: [10, 40, 20, 30, 40, 10]
    중앙값: Some(25.0)
    최빈값: [10, 40]
3. 빈 목록
    중앙값: None
    최빈값: []
```

### 11_pig_latin — 피그 라틴 (The Book 연습 과제 풀이)

영어 단어가 자음으로 시작하면 첫 자음을 끝으로 옮기고 "ay"를 붙이고(first → irst-fay), 모음으로 시작하면 "hay"를 붙입니다(apple → apple-hay). 첫 글자는 `chars().next()`로 꺼내는데, `next()`는 반복자에서 다음 값을 `Option`으로 꺼내는 메서드입니다(ch13). 나머지는 `&word[first.len_utf8()..]`로 잘라 글자 경계를 지킵니다. 알파벳으로 시작하지 않는 단어는 그대로 두므로, 한글이 섞여도 패닉하지 않습니다.

```sh
cargo run -p p0-ch8-collections --example 11_pig_latin
```

```
1. 단어 하나씩 바꿔 봅니다
    first → irst-fay
    apple → apple-hay
    string → tring-say
    Hello → ello-Hay
2. 문장 전체를 바꿉니다
    rust is fun and easy
    → ust-ray is-hay un-fay and-hay easy-hay
3. 영어가 아닌 단어가 섞여도 패닉하지 않습니다
    Hello 러스트 world
    → ello-Hay 러스트 orld-way
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch8-collections
rustc --edition 2024 compile_fail/e0502_push_while_borrowed.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 메시지를 읽는 법은 [ch4의 4절](../ch4-ownership/README.md#4-자주-하는-실수)과 같습니다.

### 4.1 요소를 빌린 채 `push` — E0502

파일: [`compile_fail/e0502_push_while_borrowed.rs`](compile_fail/e0502_push_while_borrowed.rs)

```
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
  --> compile_fail/e0502_push_while_borrowed.rs:13:5
   |
11 |     let first = &v[0];
   |                  - immutable borrow occurs here
12 |
13 |     v.push(6);
   |     ^^^^^^^^^ mutable borrow occurs here
14 |
15 |     println!("첫 번째 요소: {first}");
   |                              ----- immutable borrow later used here
```

- **원인**: `first`가 15번 줄에서 쓰이므로, 13번 줄의 `push`(가변 대여) 시점에 불변 대여가 아직 살아 있습니다. `push`가 재할당을 일으키면 `first`는 해제된 메모리를 가리키게 됩니다(2.6절).
- **해결**: 2.6절의 네 가지 방법 중 하나를 씁니다. 이 코드에서는 `let first = v[0];`으로 값을 복사하거나, `println!`을 `push` 위로 옮기는 것이 가장 간단합니다.
- **같은 에러, 다른 모양**: `for x in &v { v.push(..); }`처럼 순회 중에 같은 벡터에 `push`해도 E0502입니다. 이때는 `&v` 한 곳에 "immutable borrow occurs here"와 "immutable borrow later used here"가 함께 표시됩니다. 반복이 끝날 때까지 `v`를 빌리고 있기 때문입니다. 추가할 값을 다른 벡터에 모았다가 반복이 끝난 뒤 넣으세요.

### 4.2 문자열에 정수 인덱스 — E0277

파일: [`compile_fail/e0277_string_integer_index.rs`](compile_fail/e0277_string_integer_index.rs)

```
error[E0277]: the type `str` cannot be indexed by `{integer}`
  --> compile_fail/e0277_string_integer_index.rs:10:19
   |
10 |     let first = s[0];
   |                   ^ string indices are ranges of `usize`
   |
   = help: the trait `SliceIndex<str>` is not implemented for `{integer}`
   = note: you can use `.chars().nth()` or `.bytes().nth()`
           for more information, see chapter 8 in The Book: <https://doc.rust-lang.org/book/ch08-02-strings.html#indexing-into-strings>
```

(이 아래에 표준 라이브러리 소스 위치를 가리키는 `help:`, `note:` 줄이 더 나옵니다. 트레이트 이야기라 지금은 건너뛰어도 됩니다.)

- **원인**: 문자열은 정수 하나로 인덱싱할 수 없습니다. "string indices are ranges of `usize`"는 문자열에 쓸 수 있는 인덱스가 `0..3` 같은 **범위**뿐이라는 뜻입니다. 2.10절에서 본 것처럼 정수 인덱스가 돌려줄 수 있는 것은 글자가 아닌 바이트 하나이기 때문입니다.
- **해결**: 컴파일러의 `note:`대로 원하는 단위를 밝힙니다. 첫 글자가 필요하면 `s.chars().nth(0)`(결과는 `Some('안')`), 첫 바이트가 필요하면 `s.bytes().nth(0)` 또는 `s.as_bytes()[0]`, 첫 글자를 문자열로 원하면 범위 `&s[0..3]`이나 `s.get(0..3)`을 씁니다.

### 4.3 `+` 뒤에 왼쪽 `String` 사용 — E0382

파일: [`compile_fail/e0382_string_add_moves.rs`](compile_fail/e0382_string_add_moves.rs)

```
error[E0382]: borrow of moved value: `s1`
  --> compile_fail/e0382_string_add_moves.rs:14:16
   |
 8 |     let s1 = String::from("안녕, ");
   |         -- move occurs because `s1` has type `String`, which does not implement the `Copy` trait
...
11 |     let s3 = s1 + &s2;
   |              -- value moved here
...
14 |     println!("{s1}");
   |                ^^ value borrowed here after move
   |
help: consider cloning the value if the performance cost is acceptable
   |
11 |     let s3 = s1.clone() + &s2;
   |                ++++++++
```

- **원인**: `+`는 왼쪽 `String`을 `self`로 받으므로 `s1`의 소유권이 이동합니다(2.9절). "value moved here"가 `s1 + &s2`의 `s1`을 가리킵니다.
- **해결**: 원래 문자열도 계속 써야 하면 `format!("{s1}{s2}")`을 씁니다. `format!`은 아무것도 이동시키지 않습니다. `help:`의 `s1.clone() + &s2`도 동작하지만, 복제한 뒤 이어 붙이는 것이므로 `format!`보다 나을 것이 없습니다.
- **비슷한 실수**: `s1 + s2`처럼 오른쪽에 `&`를 빠뜨리면 E0308(``mismatched types``, ``expected `&str`, found `String` ``)이 나고, 컴파일러가 `&s2`로 고치라고 제안합니다.

### 4.4 맵에 넣은 뒤 원래 `String` 사용 — E0382

파일: [`compile_fail/e0382_hashmap_insert_moves.rs`](compile_fail/e0382_hashmap_insert_moves.rs)

```
error[E0382]: borrow of moved value: `field_name`
  --> compile_fail/e0382_hashmap_insert_moves.rs:16:16
   |
10 |     let field_name = String::from("좋아하는 색");
   |         ---------- move occurs because `field_name` has type `String`, which does not implement the `Copy` trait
...
14 |     map.insert(field_name, field_value);
   |                ---------- value moved here
15 |
16 |     println!("{field_name}: {:?}", map.get("좋아하는 색"));
   |                ^^^^^^^^^^ value borrowed here after move
   |
help: consider cloning the value if the performance cost is acceptable
   |
14 |     map.insert(field_name.clone(), field_value);
   |                          ++++++++
```

- **원인**: `insert`에 넘긴 `String`의 소유자는 이제 `map`입니다. 함수에 인자로 넘길 때 이동하는 것(ch4)과 똑같습니다.
- **해결**: 값은 맵에서 꺼내 씁니다(`map.get(..)`). 맵과 별개로 원래 변수도 필요하면 `help:`대로 `clone()`한 사본을 넣습니다. 맵이 원래 값보다 오래 쓰이지 않는다면 `&str` 참조를 키로 넣는 방법도 있습니다([`08_hashmap_basics.rs`](examples/08_hashmap_basics.rs) 7번 구역).

### 4.5 실행 중 패닉: 범위 밖 인덱스와 글자 경계

다음 두 실수는 컴파일은 되지만 실행하면 패닉하고 종료 코드 101로 끝납니다. 메시지 위의 `thread 'main' (숫자) panicked at 파일:줄:열:` 줄은 패닉 위치를 알려 주며, 괄호 안의 스레드 ID는 실행할 때마다 달라집니다.

| 코드 | 패닉 메시지 | 패닉하지 않는 대안 |
| --- | --- | --- |
| `numbers[5]` (길이 3인 벡터) | `index out of bounds: the len is 3 but the index is 5` | `numbers.get(5)` → `None` |
| `&korean[0..1]` (`"안녕하세요"`) | `end byte index 1 is not a char boundary; it is inside '안' (bytes 0..3 of string)` | `korean.get(0..1)` → `None`, `is_char_boundary(1)`로 미리 확인 |
| `&word[1..]` (`"러스트"`) | `start byte index 1 is not a char boundary; it is inside '러' (bytes 0..3 of string)` | `&word[c.len_utf8()..]` (`c`는 첫 글자) |

문자열 슬라이스 메시지는 어느 바이트가 어느 글자의 중간인지, 그 글자가 몇 번 바이트부터 몇 번 바이트까지인지 알려 줍니다. 범위의 시작이 문제면 `start`, 끝이 문제면 `end`로 시작합니다.

### 4.6 그 밖의 에러

| 상황 | 에러 | 해결 |
| --- | --- | --- |
| `let v = Vec::new();`에 아무것도 넣지 않음 | E0282 ``type annotations needed for `Vec<_>` `` | `let v: Vec<i32> = Vec::new();`처럼 타입을 적습니다. |
| `use` 없이 `HashMap::new()` | E0433 ``cannot find type `HashMap` in this scope`` | 파일 위에 `use std::collections::HashMap;`을 추가합니다. |
| `for x in v` 뒤에 `v` 사용 | E0382 ``borrow of moved value: `v` `` | 반복에 `&v`를 넘깁니다(2.4절). |

## 5. 연습 문제

### 연습 1. 글자 빈도수

문자열에서 공백을 뺀 각 글자(`char`)가 몇 번 나오는지 세어, 글자 순서대로 출력하는 프로그램을 작성하세요.

```
입력: "간장 공장 공장장"
출력:
간: 1
공: 2
장: 4
```

<details>
<summary>힌트</summary>

[`09_word_count.rs`](examples/09_word_count.rs)에서 `split_whitespace()`로 단어를 꺼내던 자리를 `for c in text.chars()`로 바꾸면 됩니다. 맵의 타입은 `HashMap<char, i32>`가 됩니다. 공백은 `if c == ' ' { continue; }`로 건너뜁니다. 출력 순서를 고정하려면 `(글자, 개수)` 쌍을 벡터에 모아 `sort()`합니다. `char`도 코드 포인트 순서로 정렬되며, 한글 음절은 가나다순으로 배치되어 있습니다.

</details>

### 연습 2. 순서를 지키며 중복 없애기

`fn dedup_keep_order(numbers: &[i32]) -> Vec<i32>`를 작성하세요. 처음 나온 순서를 유지하면서 두 번째 이후로 나오는 값을 빼야 합니다.

```rust
dedup_keep_order(&[3, 1, 3, 2, 1, 4]) // [3, 1, 2, 4]
dedup_keep_order(&[])                 // []
```

<details>
<summary>힌트</summary>

결과 벡터를 만들고 `for n in numbers`로 돌면서, 아직 넣지 않은 값만 `push(*n)`합니다. "이미 넣었는가"를 확인하는 방법은 두 가지입니다. (가) 결과 벡터에 `result.contains(n)`을 묻습니다. `contains`는 벡터를 처음부터 훑으므로 목록이 길면 느립니다. (나) 본 값을 기록하는 `HashMap<i32, bool>`을 따로 두고 `seen.contains_key(n)`으로 확인한 뒤 `seen.insert(*n, true)`로 기록합니다. 키로 바로 찾으므로 목록이 길어도 빠릅니다.

</details>

### 연습 3. 글자 단위로 뒤집기

`fn reverse_chars(s: &str) -> String`을 작성해 `"안녕하세요"`를 `"요세하녕안"`으로, `"Hello, 세상"`을 `"상세 ,olleH"`로 바꾸세요. 그런 다음 조합형 문자열 `"\u{1112}\u{1161}\u{11AB}\u{1100}\u{1173}\u{11AF}"`(화면에는 "한글"로 보임)을 넣어 보고, 결과가 왜 "글한"이 되지 않는지 설명해 보세요.

<details>
<summary>힌트</summary>

`for c in s.chars()`로 글자를 `Vec<char>`에 모은 뒤, 뒤에서부터 꺼내 새 `String`에 `push`합니다. 뒤에서부터 꺼내는 방법은 `for c in chars.iter().rev()`(ch3에서 범위에 썼던 `rev()`), 또는 `loop` 안에서 `match chars.pop()`으로 `None`이 나올 때 `break`하는 방법이 있습니다. 바이트 단위로 뒤집으면 UTF-8 순서가 깨지므로 반드시 `char` 단위여야 합니다. 조합형 문자열은 `char` 6개(ㅎ ㅏ ㄴ ㄱ ㅡ ㄹ)이므로, `char` 단위로 뒤집으면 ㄹ ㅡ ㄱ ㄴ ㅏ ㅎ 순서가 되어 글자가 제대로 조합되지 않습니다. `char`가 자소 클러스터와 같지 않다는 2.10절의 내용을 떠올려 보세요.

</details>

### 연습 4. 부서별 직원 명단

(부서, 이름) 목록으로 부서마다 직원 이름을 모으고, 부서 이름순으로, 부서 안에서는 직원 이름순으로 출력하세요. The Book 8장 연습 과제의 "부서별 직원 관리"를 입력 없이 단순하게 바꾼 것입니다.

```rust
let records = [("개발", "민수"), ("영업", "지영"), ("개발", "아라"), ("영업", "철수"), ("개발", "가온")];
```

```
개발: ["가온", "민수", "아라"]
영업: ["지영", "철수"]
```

<details>
<summary>힌트</summary>

맵의 타입은 `HashMap<&str, Vec<&str>>`입니다. `for (dept, name) in records`로 돌면서 `departments.entry(dept).or_insert(Vec::new()).push(name);`처럼 쓰면, 처음 보는 부서는 빈 벡터를 넣고 그 벡터의 가변 참조에 바로 `push`합니다. 출력할 때는 `departments.keys()`(키만 하나씩 내주는 메서드)로 부서 이름을 벡터에 모아 정렬하고, 부서마다 `departments.get_mut(dept)`로 직원 벡터의 가변 참조를 얻어 `sort()`한 뒤 출력합니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [8. Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html)
  - [8.1 Storing Lists of Values with Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html)
  - [8.2 Storing UTF-8 Encoded Text with Strings](https://doc.rust-lang.org/book/ch08-02-strings.html)
  - [8.3 Storing Keys with Associated Values in Hash Maps](https://doc.rust-lang.org/book/ch08-03-hash-maps.html): 끝부분 "Summary"에 이 챕터 10, 11번 예제의 원래 과제와 연습 4의 원래 과제가 있습니다.
- 표준 라이브러리 문서
  - [`std::collections`](https://doc.rust-lang.org/std/collections/index.html): 벡터·해시 맵 외의 컬렉션과 고르는 기준
  - [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html): "Guarantees" 절에 용량과 재할당에 대해 보장하는 것과 보장하지 않는 것이 정리되어 있습니다.
  - [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html): 기본 해시 함수와 순회 순서에 대한 설명
- Rust By Example: [Std library types](https://doc.rust-lang.org/rust-by-example/std.html) 장의 다음 절
  - [Vectors](https://doc.rust-lang.org/rust-by-example/std/vec.html): `push`, `pop`, `len`, 인덱싱, `iter_mut()`로 수정하기
  - [Strings](https://doc.rust-lang.org/rust-by-example/std/str.html): `String`과 `&str`, 이스케이프 문자와 원시 문자열 리터럴
  - [HashMap](https://doc.rust-lang.org/rust-by-example/std/hash.html): `insert`, `get`, `remove`, 순회
  - 예제에 `collect()`, `iter_mut()` 같은 반복자 메서드가 나옵니다(ch13). "벡터나 문자열을 만들어 주는 메서드" 정도로 보고 읽으면 됩니다.
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`05_vecs`](https://github.com/rust-lang/rustlings/tree/main/exercises/05_vecs): 벡터 만들기와 순회하며 새 벡터 채우기
  - [`09_strings`](https://github.com/rust-lang/rustlings/tree/main/exercises/09_strings): `String`과 `&str` 구분하고 만들기
  - [`11_hashmaps`](https://github.com/rust-lang/rustlings/tree/main/exercises/11_hashmaps): 해시 맵에 넣고 갱신하기
  - 연습 파일 아래쪽의 `#[cfg(test)]`, `#[test]`, `assert_eq!`은 정답을 확인하는 테스트(ch11)입니다. `hashmaps3.rs`의 `unwrap()`은 `Some`이면 값을 꺼내고 `None`이면 패닉하는 메서드입니다(ch9).
- 에러 설명: `rustc --explain E0502`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
