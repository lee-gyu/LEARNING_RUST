# ch4. 소유권 이해하기

소유권(ownership)은 Rust가 가비지 컬렉터 없이도 메모리를 안전하게 다루는 방법이고, 다른 언어에서 오신 분들이 Rust에서 가장 낯설게 느끼는 개념이기도 합니다. 이 챕터에서는 규칙을 외우는 데서 그치지 않고, 각 규칙이 **어떤 버그를 막으려고 생겼는지**를 예제로 확인합니다.

- 패키지: `p0-ch4-ownership`
- 기준 문서: [The Book 4장 Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- 확인한 툴체인: rustc 1.100.0, Edition 2024. 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 소유권 규칙 세 가지를 설명하고, 코드에서 값이 버려지는(drop) 지점을 짚을 수 있다.
2. 이동(move)·복사(copy)·복제(clone)의 차이를 설명하고, 대입·함수 인자·반환값에서 소유권이 어디로 옮겨 가는지 추적할 수 있다.
3. 불변 참조와 가변 참조로 값을 빌려주는 함수를 작성하고, 대여 규칙과 "참조는 마지막 사용 지점까지 유효하다"는 원칙을 설명할 수 있다.
4. 문자열 슬라이스(`&str`)와 배열 슬라이스(`&[T]`)를 만들어 쓰고, 함수 매개변수로 `&String`보다 `&str`을 쓰는 이유를 설명할 수 있다.
5. 대여 검사기가 내는 에러(E0382, E0499, E0502, E0596, E0106)를 읽고 원인을 찾아 고칠 수 있다.

## 2. 핵심 개념

### 2.1 왜 소유권이 필요한가

프로그램이 실행 중에 빌려 쓴 메모리는 언젠가 돌려줘야 합니다. 언어마다 이 문제를 푸는 방식이 다릅니다.

- **수동 관리 (C 등)**: 프로그래머가 `malloc`으로 받고 `free`로 돌려줍니다. 빠르지만 실수가 곧 버그입니다. 돌려주는 것을 잊으면 메모리 누수, 같은 메모리를 두 번 돌려주면 **이중 해제(double free)**, 돌려준 메모리를 계속 쓰면 **해제 후 사용(use after free)** 이 됩니다. 뒤의 두 가지는 프로그램을 망가뜨리거나 보안 취약점이 됩니다.
- **가비지 컬렉션 (Java, Python, Go, JavaScript 등)**: 런타임이 더 이상 쓰이지 않는 메모리를 찾아 대신 돌려줍니다. 안전하지만 실행 중에 추가 비용이 들고, 언제 정리될지 프로그래머가 정확히 알기 어렵습니다.
- **Rust의 소유권**: 컴파일러가 소유권 규칙을 검사해 **메모리를 돌려줄 시점을 컴파일할 때 정합니다.** 규칙을 어기는 코드는 컴파일되지 않고, 실행 중에는 추가 비용이 없습니다.

이 챕터의 규칙들은 각각 아래 버그 중 하나를 막습니다.

| 막으려는 버그 | 무슨 일이 생기나 | 막는 규칙 | 관련 절 |
| --- | --- | --- | --- |
| 이중 해제 | 같은 메모리를 두 번 해제 | 소유자는 한 번에 하나, 대입하면 이동 | 2.5 |
| 해제 후 사용 | 이미 해제된 메모리를 읽거나 씀 | 이동한 뒤에는 원래 변수를 쓸 수 없음 | 2.5 |
| 데이터 경쟁 | 한쪽이 바꾸는 동안 다른 쪽이 읽음 | 가변 참조 하나, 또는 불변 참조 여러 개 | 2.9 |
| 댕글링 참조 | 참조가 이미 사라진 값을 가리킴 | 참조는 항상 유효해야 함 | 2.11 |

### 2.2 스택과 힙 (이 챕터에 필요한 만큼만)

- **스택(stack)**: 함수가 호출될 때 지역 변수가 쌓이고, 함수가 끝나면 한꺼번에 사라지는 공간입니다. 크기가 컴파일할 때 정해진 값만 둘 수 있고, 매우 빠릅니다. `i32`, `f64`, `bool`, `char`와 이런 타입으로만 이뤄진 튜플·배열이 여기에 놓입니다.
- **힙(heap)**: 실행 중에 크기가 정해지거나 바뀌는 데이터를 두는 공간입니다. 할당자에게 공간을 요청하면 그 위치(포인터)를 돌려받습니다. 스택과 달리 저절로 사라지지 않으므로 **누군가 책임지고 돌려줘야 합니다.** 그 "누군가"를 정하는 것이 소유권입니다.

`String`은 힙 데이터를 소유하는 대표적인 타입입니다. `String::from("hello")`로 만들면 글자는 힙에 놓이고, 변수에는 힙을 가리키는 포인터·길이·용량 세 칸이 들어갑니다. 64비트 환경에서 이 세 칸은 24바이트입니다([`02_move_copy_clone.rs`](examples/02_move_copy_clone.rs)에서 확인합니다).

```
  s1 (스택에 있는 String 값)
  +----------+-----+            힙
  | ptr      |  ---|------->  +---+---+---+---+---+
  | len      |  5  |          | h | e | l | l | o |
  | capacity |  5  |          +---+---+---+---+---+
  +----------+-----+
```

`String`은 `push_str`로 글자를 덧붙일 수 있는 문자열입니다. 반면 코드에 직접 쓴 `"hello"` 같은 문자열 리터럴은 프로그램 안에 고정돼 있어 바꿀 수 없습니다(2.12절). `String`의 자세한 사용법은 ch8에서 다룹니다.

### 2.3 소유권 규칙

1. Rust의 각 값에는 **소유자(owner)** 가 있습니다.
2. 소유자는 한 번에 **하나**뿐입니다.
3. 소유자가 스코프를 벗어나면 값은 **버려집니다(drop)**.

이 세 줄이 이 챕터 전체의 바탕입니다. 이어지는 절은 모두 이 규칙에서 나옵니다.

### 2.4 스코프와 drop

변수가 선언된 블록(`{ }`)이 끝나면, Rust는 그 변수가 소유한 값을 정리하는 `drop`을 **자동으로** 호출합니다. `String`이라면 이때 힙 메모리가 해제됩니다. `free`를 직접 부를 필요가 없고, 부르는 것을 잊을 수도 없습니다. (C++의 RAII와 같은 방식입니다.)

```rust
{
    let s = String::from("hello"); // s가 힙 메모리의 소유자가 됩니다.
    println!("{s}");
} // s의 스코프가 끝납니다. Rust가 drop을 호출해 힙 메모리를 해제합니다.
```

`String`의 drop은 아무것도 출력하지 않아서 눈에 보이지 않습니다. 그래서 [`01_scope_and_drop.rs`](examples/01_scope_and_drop.rs)는 버려질 때 `[drop] 이름`을 출력하는 관찰용 타입 `Noisy`를 만들어 다음 사실을 확인합니다.

- 블록이 끝나는 `}`에서 값이 drop됩니다.
- 한 스코프의 값들은 **선언의 역순**으로 drop됩니다. 나중에 만든 값이 먼저 만든 값에 기대고 있을 수 있으므로 역순으로 정리하는 것이 안전합니다.
- 변수에 새 값을 대입하면 **이전 값은 그 자리에서** drop됩니다. 이전 값의 소유자가 사라졌기 때문입니다.

`Noisy`를 만드는 데 쓴 구조체(struct)는 ch5에서, `Drop` 트레이트는 ch14에서 배웁니다. 지금은 "버려질 때 메시지를 출력하는 값"으로만 보면 됩니다.

### 2.5 이동(move)

```rust
let s1 = String::from("hello");
let s2 = s1;
```

이 대입에서 복사되는 것은 스택에 있는 세 칸(포인터, 길이, 용량)뿐이고, 힙의 글자는 복사되지 않습니다. 그러면 `s1`과 `s2`가 같은 힙 메모리를 가리키게 되는데, 둘 다 유효하다면 스코프가 끝날 때 **두 변수가 같은 메모리를 각각 해제하는 이중 해제**가 일어납니다.

Rust는 이 문제를 이렇게 풉니다. **`let s2 = s1;` 이후 `s1`은 더 이상 유효하지 않습니다.** 소유권이 `s2`로 옮겨 갔다는 뜻에서 이것을 **이동(move)** 이라고 부릅니다. 소유자가 하나뿐이니 해제도 한 번만 일어납니다. `s1`을 다시 쓰려 하면 컴파일러가 막으므로(E0382, 4.1절) 해제 후 사용도 일어날 수 없습니다.

```
  s1 (이동 후 사용 불가)
  +----------+-----+
  | ptr      |  ---|----+
  | len      |  5  |    |       힙
  | capacity |  5  |    |     +---+---+---+---+---+
  +----------+-----+    +---> | h | e | l | l | o |
  s2                    |     +---+---+---+---+---+
  +----------+-----+    |
  | ptr      |  ---|----+
  | len      |  5  |
  | capacity |  5  |
  +----------+-----+
```

[`02_move_copy_clone.rs`](examples/02_move_copy_clone.rs)는 이동 전후의 힙 주소를 비교해, 이동이 힙 데이터를 건드리지 않고 주인만 바꾼다는 것을 `true`로 확인합니다.

다른 언어와 비교하면 차이가 분명합니다. Python이나 Java에서 `s2 = s1`은 같은 객체를 가리키는 이름이 하나 더 생기는 것이고, 가비지 컬렉터가 둘 다 안 쓰일 때까지 기다렸다가 정리합니다. C++의 `std::string s2 = s1;`은 기본적으로 힙 데이터까지 복사합니다. Rust는 둘 다 아니고 **이동**합니다. Rust는 힙 데이터를 자동으로 복사하지 않습니다. 대입할 때 복사되는 것은 언제나 스택에 있는 부분뿐이고, 힙 데이터까지 복사하는 비싼 작업은 다음 절의 `clone()`처럼 코드에 드러납니다.

이동해 나간 변수라도 `mut`로 선언했다면 새 값을 대입해 다시 쓸 수 있습니다. 새 값의 소유자가 되었기 때문입니다.

### 2.6 복제(clone)와 복사(copy)

힙 데이터까지 정말 두 벌이 필요하면 `clone()`을 **명시적으로** 호출합니다.

```rust
let s1 = String::from("hello");
let s2 = s1.clone(); // 새 힙 공간을 할당하고 글자를 복사합니다.
println!("s1 = {s1}, s2 = {s2}"); // 둘 다 쓸 수 있습니다.
```

`clone()`은 데이터 크기만큼 비용이 듭니다. 코드에서 `clone`이 보이면 "여기서 비싼 일이 일어날 수 있다"는 신호로 읽으면 됩니다.

그런데 정수는 대입해도 원래 변수를 계속 쓸 수 있습니다.

```rust
let x = 5;
let y = x;
println!("x = {x}, y = {y}"); // 문제없습니다.
```

`i32`는 스택에만 있는 4바이트 값이라서, 통째로 복사하는 비용이 이동과 똑같고 정리할 힙 메모리도 없습니다. 원래 변수를 무효로 만들 이유가 없는 것입니다. 이런 타입은 **`Copy` 트레이트**를 가지고 있어서 대입하면 이동 대신 **복사(copy)** 됩니다. (트레이트(trait)는 "타입이 가진 능력"을 나타내는 표시로, ch10에서 배웁니다.)

`Copy`인 타입:

- 모든 정수 타입(`i32`, `u64` 등), 모든 부동소수점 타입(`f64` 등), `bool`, `char`
- `Copy` 타입으로만 이뤄진 튜플과 배열. `(i32, f64)`, `[i32; 3]`은 `Copy`이지만 `(i32, String)`은 아닙니다.

`String`처럼 힙 메모리나 다른 자원을 소유해서 정리 작업이 필요한 타입은 `Copy`가 될 수 없습니다. 복사본마다 정리를 하면 이중 해제가 되기 때문입니다.

| | 이동 (move) | 복사 (copy) | 복제 (clone) |
| --- | --- | --- | --- |
| 일어나는 때 | `Copy`가 아닌 값을 대입·전달 | `Copy` 타입을 대입·전달 | `.clone()`을 직접 호출 |
| 복사되는 것 | 스택 부분만 | 값 전체(원래 스택에만 있음) | 스택 부분과 힙 데이터 모두 |
| 원래 변수 | 사용 불가 | 계속 사용 | 계속 사용 |
| 비용 | 작음 | 작음 | 데이터 크기에 비례 |

### 2.7 함수와 소유권

함수에 인자를 넘기는 것도 대입과 같습니다. `String`을 넘기면 소유권이 매개변수로 **이동**하고, 정수를 넘기면 **복사**됩니다. 반환값도 마찬가지로 소유권을 호출한 쪽으로 넘겨줍니다.

```rust
fn takes_ownership(some_string: String) {
    println!("{some_string}");
} // 여기서 some_string이 drop되어 힙 메모리가 해제됩니다.

fn gives_ownership() -> String {
    String::from("yours") // 소유권이 호출한 쪽으로 넘어갑니다.
}

let s = String::from("hello");
takes_ownership(s);
// println!("{s}"); // E0382: s는 이미 함수로 이동했습니다.
```

[`03_ownership_and_functions.rs`](examples/03_ownership_and_functions.rs)에서는 `Noisy` 값을 함수로 넘기면 `[drop]`이 **그 함수가 끝날 때**, 즉 `main`의 다음 줄보다 먼저 찍히는 것을 볼 수 있습니다. 새 소유자인 매개변수의 스코프가 함수 본문이기 때문입니다.

그런데 길이만 알고 싶을 때도 `String`을 넘기면 소유권이 넘어가 버려서, 원래 값을 튜플로 다시 돌려받아야 합니다.

```rust
fn calculate_length(s: String) -> (String, usize) {
    let length = s.len();
    (s, length) // 받은 String을 돌려줘야 호출한 쪽이 다시 쓸 수 있습니다.
}
```

값을 잠깐 쓰기만 하는데 매번 소유권을 주고받는 것은 번거롭습니다. 이것을 해결하는 것이 참조입니다.

### 2.8 참조와 대여

**참조(reference)** 는 값을 소유하지 않고 가리키기만 하는 값입니다. `&s1`처럼 만들고, 타입은 `&String`처럼 씁니다. 참조를 만드는 일을 **대여(borrow)** 라고 부릅니다. 빌린 것이므로 쓰고 나면 돌려줘야 하고, 빌린 쪽이 버릴 수도 없습니다.

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
} // s는 참조일 뿐 소유자가 아니므로, 여기서 String이 해제되지 않습니다.

let s1 = String::from("hello");
let len = calculate_length(&s1); // 호출하는 쪽에도 &가 보입니다.
println!("'{s1}'의 길이는 {len}입니다"); // s1을 계속 쓸 수 있습니다.
```

```
  r = &s1          s1
  +-----+          +----------+-----+            힙
  | ptr |--------> | ptr      |  ---|------->  +---+---+---+---+---+
  +-----+          | len      |  5  |          | h | e | l | l | o |
                   | capacity |  5  |          +---+---+---+---+---+
                   +----------+-----+
```

참조는 기본적으로 **불변**이라서 빌린 값을 읽을 수만 있습니다. 바꾸려면 **가변 참조** `&mut`가 필요하고, 세 곳을 모두 맞춰야 합니다.

```rust
fn change(some_string: &mut String) {   // ③ 매개변수 타입이 &mut String
    some_string.push_str(", world");
}

let mut s = String::from("hello");     // ① 변수 자체가 mut
change(&mut s);                        // ② 넘길 때 &mut
```

참조가 가리키는 정수 자체를 바꿀 때는 **역참조 연산자 `*`** 로 참조를 따라가야 합니다(`*n += 1`). 반면 `some_string.push_str(...)`처럼 메서드를 호출할 때는 컴파일러가 참조를 자동으로 따라가 주므로 `*`를 쓰지 않습니다. 예제는 [`04_borrowing.rs`](examples/04_borrowing.rs)에 있습니다.

### 2.9 대여 규칙

1. 어느 시점에든 **가변 참조 하나** 또는 **불변 참조 여러 개** 중 한쪽만 존재할 수 있습니다.
2. 참조는 **항상 유효**해야 합니다.

첫 번째 규칙은 **데이터 경쟁(data race)** 을 막습니다. 데이터 경쟁은 다음 세 조건이 동시에 맞을 때 생깁니다.

- 둘 이상의 포인터가 같은 데이터에 동시에 접근한다.
- 그중 적어도 하나가 데이터를 쓴다.
- 접근을 동기화하는 장치가 없다.

데이터 경쟁은 실행할 때마다 결과가 달라질 수 있어 찾기가 매우 어렵습니다. Rust는 이런 코드를 아예 컴파일하지 않습니다. 스레드는 ch15에서 다루지만, 이 규칙은 스레드가 하나여도 의미가 있습니다. 예를 들어 `String`에 글자를 덧붙이다 용량이 모자라면 더 큰 힙 공간을 새로 할당해 글자를 옮기고 예전 공간을 해제할 수 있습니다. 그 순간 예전 공간을 가리키던 참조는 해제된 메모리를 가리키게 됩니다. "누군가 읽는 중이면 아무도 바꾸지 못한다"는 규칙이 이런 상황을 원천적으로 막습니다.

불변 참조는 여러 개여도 괜찮습니다. 모두 읽기만 하므로 서로의 결과에 영향을 줄 수 없기 때문입니다.

### 2.10 참조는 마지막으로 사용되는 곳까지 유효합니다

"동시에"를 판단하는 기준은 변수의 스코프(중괄호)가 아닙니다. **참조의 유효 범위는 참조를 만든 곳에서 시작해 그 참조가 마지막으로 사용되는 곳에서 끝납니다.** 그래서 아래 코드는 컴파일되고 실행됩니다.

```rust
let mut s = String::from("hello");

let r1 = &s;
let r2 = &s;
println!("r1 = {r1}, r2 = {r2}"); // r1, r2의 마지막 사용. 불변 대여가 여기서 끝납니다.

let r3 = &mut s; // 겹치는 불변 대여가 없으므로 괜찮습니다.
r3.push_str(", world");
println!("r3 = {r3}");
```

변수 `r1`, `r2`라는 이름은 블록이 끝날 때까지 남아 있지만, 대여는 마지막 사용과 함께 이미 끝났습니다. 대여 검사기(borrow checker)가 이렇게 중괄호가 아니라 실제 사용 지점을 따지는 방식을 NLL(Non-Lexical Lifetimes)이라고 부릅니다. 예전의 대여 검사기는 참조 변수의 스코프 끝까지 대여가 이어진다고 보았기 때문에, 오래된 자료에서 "에러가 난다"고 소개한 코드가 지금은 컴파일되는 경우가 있습니다.

반대로 `r3`를 만든 뒤에 `r1`을 한 번 더 사용하면, 불변 대여가 `r3`의 구간까지 늘어나 겹치므로 E0502가 납니다. [`05_borrow_scope.rs`](examples/05_borrow_scope.rs)는 이 원칙을 실행되는 코드로 보여 줍니다.

대여 규칙에 걸렸을 때 쓸 수 있는 해결책은 대개 다음 중 하나입니다.

1. **순서 바꾸기**: 참조의 마지막 사용이 수정보다 먼저 오도록 줄 순서를 바꿉니다.
2. **필요한 값만 복사해 두기**: 참조 대신 `s.len()` 같은 `Copy` 값을 받아 두면 대여가 바로 끝납니다.
3. **스코프 나누기**: 중괄호로 블록을 만들어 참조의 구간을 눈에 보이게 제한합니다.
4. **복제하기**: 정말로 원본과 독립된 값이 필요할 때 `clone()`을 씁니다.

### 2.11 댕글링 참조

**댕글링 참조(dangling reference)** 는 이미 해제된 메모리를 가리키는 참조입니다. C에서 함수 안의 지역 변수 주소를 반환하는 것이 대표적인 예입니다. Rust에서 같은 일을 시도하면 컴파일되지 않습니다.

```rust
fn dangle() -> &String {
    let s = String::from("hello");
    &s
} // s가 여기서 drop됩니다. 반환한 참조는 해제된 메모리를 가리키게 됩니다.
```

컴파일러는 E0106으로 거부합니다(4.5절). 에러 메시지에 나오는 라이프타임(lifetime)은 ch10에서 배우고, 지금은 "함수 안에서 만든 값의 참조는 반환할 수 없다"고 기억하면 됩니다. 해결책은 참조 대신 값 자체를 반환해 소유권을 넘기는 것입니다.

```rust
fn no_dangle() -> String {
    let s = String::from("hello");
    s // 소유권이 호출한 쪽으로 이동하므로 아무것도 해제되지 않습니다.
}
```

### 2.12 슬라이스

#### 인덱스를 돌려주면 생기는 문제

"문자열에서 첫 단어를 찾는 함수"를 만든다고 해 봅시다. 참조만 배운 상태라면 첫 공백의 위치(숫자)를 돌려주게 됩니다. 그런데 그 숫자는 원래 문자열과 아무 연결이 없습니다.

```rust
let mut text = String::from("hello world");
let end = first_word_end(&text); // 5
text.clear();                    // text는 빈 문자열이 되었지만
println!("{end}");               // end는 여전히 5. 컴파일러는 이 값이 낡았다는 것을 모릅니다.
```

이 상태에서 `&text[..end]`를 시도하면 실행 중에 패닉(panic)이 납니다. 패닉은 프로그램이 에러 메시지를 출력하고 즉시 멈추는 것으로, ch9에서 자세히 다룹니다.

```
end byte index 5 is out of bounds for string of length 0
```

#### 문자열 슬라이스 `&str`

**슬라이스(slice)** 는 컬렉션의 연속된 일부를 가리키는 참조입니다. 문자열 슬라이스의 타입은 `&str`이고, 범위 문법으로 만듭니다.

```rust
let s = String::from("hello world");
let hello = &s[0..5];  // 0번 바이트부터 5번 바이트 앞까지
let world = &s[6..11];
```

| 쓰는 법 | 같은 뜻 | 결과 |
| --- | --- | --- |
| `&s[0..5]` | `&s[..5]` | `"hello"` |
| `&s[6..11]` | `&s[6..]` | `"world"` |
| `&s[0..11]` | `&s[..]` | `"hello world"` |

슬라이스는 **(시작 위치의 포인터, 길이)** 두 칸만 가진 참조라서 글자를 복사하지 않습니다.

```
  s (String)                   힙
  +----------+----+           +-+-+-+-+-+-+-+-+-+-+-+
  | ptr      | ---|---------> |h|e|l|l|o| |w|o|r|l|d|
  | len      | 11 |           +-+-+-+-+-+-+-+-+-+-+-+
  | capacity | 11 |                        ^
  +----------+----+                        |
                                           |
  world = &s[6..11]                        |
  +-----+----+                             |
  | ptr | ---|-----------------------------+
  | len |  5 |
  +-----+----+
```

`first_word`가 슬라이스를 돌려주면, 반환값이 원래 문자열을 **빌리고 있다**는 사실을 컴파일러가 압니다. 그래서 슬라이스가 쓰이는 동안 `clear()`로 원본을 비우려 하면 E0502로 막힙니다(4.6절). 인덱스 방식에서는 실행 중에 터지던 버그가, 슬라이스 방식에서는 컴파일 에러가 됩니다.

#### 문자열 리터럴은 `&str`

```rust
let literal: &str = "Hello, Rust";
```

문자열 리터럴은 프로그램 바이너리 안에 들어 있는 글자를 가리키는 슬라이스입니다. 불변 참조이므로 바꿀 수 없습니다.

#### 매개변수는 `&String`보다 `&str`

```rust
fn first_word(s: &str) -> &str { /* ... */ }

first_word(&my_string);      // &String도 받습니다. &str로 자동 변환됩니다.
first_word(&my_string[6..]); // String의 일부
first_word("good morning");  // 문자열 리터럴
```

`&String`은 "어딘가에 있는 `String` 값 자체"의 참조라서, 넘길 수 있는 것이 `String`뿐입니다. `&str`은 "어딘가에 있는 연속된 글자들"의 참조라서 `String` 전체, `String`의 일부, 문자열 리터럴을 모두 받을 수 있습니다. 함수가 문자열을 읽기만 한다면 `&str`이 더 많은 경우에 쓸 수 있는 매개변수입니다. `&String`을 넘겼을 때 `&str`로 바뀌는 것은 역참조 강제 변환(deref coercion)이라는 기능 덕분이며, ch14에서 다룹니다.

Rust 코드 검사 도구인 Clippy도 `&String` 매개변수를 보면 `&str`을 쓰라고 경고합니다(`clippy::ptr_arg`). [`04_borrowing.rs`](examples/04_borrowing.rs)는 아직 슬라이스를 배우기 전이라 일부러 `&String`을 쓰고, 그 경고를 `#[expect(clippy::ptr_arg)]`로 표시해 두었습니다.

#### 한글을 자를 때 주의할 점

문자열 슬라이스의 범위는 **글자가 아니라 바이트** 단위입니다. Rust의 문자열은 UTF-8로 저장되고, 한글 한 글자는 3바이트입니다. 그래서 `"안녕 세상".len()`은 4글자가 아니라 13(4글자 x 3바이트 + 공백 1바이트)이고, 첫 글자 `안`은 `&s[0..3]`입니다. `&s[0..1]`처럼 글자 중간을 자르는 코드는 컴파일은 되지만 실행하면 다음과 같이 패닉이 납니다.

```
end byte index 1 is not a char boundary; it is inside '안' (bytes 0..3 of string)
```

`s.is_char_boundary(i)`로 `i`번 바이트가 글자 경계인지 미리 확인할 수 있습니다. 한편 공백(`b' '`)은 1바이트이고 UTF-8에서 다른 글자의 일부로 나타나지 않으므로, 공백 위치에서 자르는 `first_word`는 한글에도 안전합니다. 글자 단위로 다루는 방법은 ch8에서 자세히 배웁니다.

#### 배열 슬라이스 `&[T]`

배열에도 같은 문법이 적용됩니다. `i32` 배열의 슬라이스 타입은 `&[i32]`입니다.

```rust
let a = [1, 2, 3, 4, 5];
let middle = &a[1..3]; // [2, 3]

fn sum(numbers: &[i32]) -> i32 { /* ... */ }
sum(&a);      // 배열 전체. &[i32; 5]가 &[i32]로 자동 변환됩니다.
sum(&a[..2]); // 배열의 일부
```

배열 타입 `[i32; 5]`는 길이가 타입의 일부여서, 매개변수를 `[i32; 5]`로 쓰면 길이가 5인 배열만 받습니다. 슬라이스 `&[i32]`는 길이가 타입에 없으므로 길이가 다른 배열도 함수 하나로 다룰 수 있습니다. 원소를 바꿔야 하면 가변 슬라이스 `&mut [i32]`를 씁니다. 슬라이스도 참조이므로 대여 규칙을 똑같이 따릅니다. 예제는 [`07_array_slices.rs`](examples/07_array_slices.rs)에 있습니다.

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_scope_and_drop.rs`](examples/01_scope_and_drop.rs) | 스코프가 끝날 때 drop되는 시점, 역순 drop, 재대입 시 drop | 1 |
| [`02_move_copy_clone.rs`](examples/02_move_copy_clone.rs) | 이동·복사·복제의 차이, 힙 주소 비교 | 2 |
| [`03_ownership_and_functions.rs`](examples/03_ownership_and_functions.rs) | 함수 인자와 반환값에서의 소유권 이동 | 1, 2 |
| [`04_borrowing.rs`](examples/04_borrowing.rs) | 불변 참조와 가변 참조, 역참조 `*` | 3 |
| [`05_borrow_scope.rs`](examples/05_borrow_scope.rs) | 참조가 마지막 사용 지점까지만 유효하다는 원칙(NLL) | 3 |
| [`06_string_slices.rs`](examples/06_string_slices.rs) | 문자열 슬라이스, `&str` 매개변수, 한글 바이트 경계 | 4 |
| [`07_array_slices.rs`](examples/07_array_slices.rs) | 배열 슬라이스 `&[T]`와 가변 슬라이스 `&mut [T]` | 4 |
| [`compile_fail/`](compile_fail/) | 대여 검사기가 거부하는 대표 코드 6개 (4절) | 5 |

### 01_scope_and_drop — 스코프와 drop

관찰용 타입 `Noisy`가 언제 버려지는지 출력합니다. `[drop]` 줄이 찍히는 위치가 각 블록의 `}`와 어떻게 대응하는지 코드와 나란히 놓고 보세요.

```sh
cargo run -p p0-ch4-ownership --example 01_scope_and_drop
```

```
1. 블록이 끝나면 그 안에서 만든 값이 drop됩니다
    [생성] a
    블록 안: a 사용 중
    [drop] a
    블록 밖: a라는 이름은 더 이상 존재하지 않습니다
2. 한 스코프의 여러 값은 선언의 역순으로 drop됩니다
    [생성] first
    [생성] second
    first, second 사용 중
    [drop] second
    [drop] first
3. 변수에 새 값을 대입하면 이전 값은 그 자리에서 drop됩니다
    [생성] old
    x = old
    [생성] new
    [drop] old
    x = new
4. main이 끝날 때 남아 있는 값이 drop됩니다
    [drop] new
```

### 02_move_copy_clone — 이동, 복사, 복제

`String`의 크기가 24바이트(포인터·길이·용량)라는 것, 이동 후에는 같은 힙 주소를 가리키고(`true`) 복제 후에는 다른 힙 주소를 가리킨다는 것(`false`)을 확인합니다. 힙 주소 값 자체는 실행할 때마다 달라지므로 출력하지 않고 비교 결과만 출력합니다. 크기 24바이트는 64비트 환경 기준입니다.

```sh
cargo run -p p0-ch4-ownership --example 02_move_copy_clone
```

```
1. 정수는 대입하면 복사됩니다 (Copy 타입)
    x = 5, y = 5
2. String 값 자체는 (포인터, 길이, 용량) 세 칸짜리 작은 묶음입니다
    String의 크기: 24바이트
3. String은 대입하면 소유권이 이동합니다
    s2 = hello
    s2가 s1과 같은 힙 데이터를 가리키는가? true
4. clone()은 힙 데이터까지 통째로 복제합니다
    s2 = hello, s3 = hello
    s3가 s2와 같은 힙 데이터를 가리키는가? false
5. 값이 이동해 나간 변수도 새 값을 대입하면 다시 쓸 수 있습니다
    s4 = second, s5 = first
6. Copy 타입으로만 이뤄진 튜플·배열은 통째로 복사됩니다
    t1 = (1, 2.5, 'a', true), t2 = (1, 2.5, 'a', true)
    a1 = [1, 2, 3], a2 = [1, 2, 3]
7. String이 하나라도 섞인 튜플은 이동합니다
    pair2 = (1, "one")
```

예제 안에 주석 처리된 `println!("{s1}");`, `println!("{}", pair1.0);`의 주석을 풀어 보면 E0382가 어떻게 나는지 직접 볼 수 있습니다.

### 03_ownership_and_functions — 함수와 소유권

3번 구역에서 `[drop] n`이 `main: consume 호출이 끝남`보다 먼저 찍히는 것이 핵심입니다. 5번 구역은 소유권을 넘겼다가 튜플로 돌려받는 번거로운 방식을 보여 주며, 다음 예제의 참조로 이어집니다.

```sh
cargo run -p p0-ch4-ownership --example 03_ownership_and_functions
```

```
1. String을 인자로 넘기면 소유권이 함수로 이동합니다
    takes_ownership: hello
2. Copy 타입은 인자로 넘겨도 원래 변수를 계속 쓸 수 있습니다
    makes_copy: 5
    main: x는 여전히 5
3. 함수로 이동한 값은 그 함수가 끝날 때 drop됩니다
    consume: n 받음
    [drop] n
    main: consume 호출이 끝남
4. 반환값으로 소유권을 넘겨받을 수 있습니다
    s1 = yours, s3 = hello
5. 소유권을 넘겼다가 돌려받는 방식은 번거롭습니다
    'hello'의 길이는 5입니다
```

### 04_borrowing — 참조와 대여

03번의 `calculate_length`를 참조로 바꾸고, 가변 참조로 `String`과 정수를 수정합니다.

```sh
cargo run -p p0-ch4-ownership --example 04_borrowing
```

```
1. 불변 참조(&)로 빌려주면 소유권은 그대로 남습니다
    'hello'의 길이는 5입니다
2. 불변 참조는 동시에 여러 개 있어도 됩니다
    r1 = hello, r2 = hello, s1 = hello
3. 가변 참조(&mut)로 빌려주면 빌린 쪽이 값을 바꿀 수 있습니다
    s2 = hello, world
4. 가변 참조는 겹치지 않게 차례로 만들면 몇 번이든 쓸 수 있습니다
    count = 2
5. 참조를 담은 변수도 일반 값처럼 출력하고 메서드를 호출할 수 있습니다
    r3 = hello, world, 길이 = 12
    *rc + 10 = 12
```

### 05_borrow_scope — 참조의 유효 범위

모든 코드가 컴파일되고 실행됩니다. 주석에 표시한 "마지막 사용" 지점을 기준으로, 불변 대여와 가변 대여가 한 번도 겹치지 않는다는 것을 확인하세요. 1번 구역의 마지막 `println!` 아래에 `println!("{r1}");`을 추가하면 E0502가 납니다.

```sh
cargo run -p p0-ch4-ownership --example 05_borrow_scope
```

```
1. 불변 참조의 마지막 사용 뒤에는 가변 참조를 만들 수 있습니다
    r1 = hello, r2 = hello
    r3 = hello, world
2. 가변 참조가 끝나면 원래 변수로 다시 읽고 쓸 수 있습니다
    s = hello, world
    s = hello, world!
3. 가변 참조도 사용 구간이 겹치지 않으면 차례로 여러 번 만들 수 있습니다
    s = hello, world!??
4. 중괄호로 스코프를 나눠 대여 구간을 눈에 보이게 할 수도 있습니다
    r4 = new
5. 참조를 오래 들고 있을 필요가 없으면, 필요한 값만 복사해 둡니다
    길이: 3 → 9
```

### 06_string_slices — 문자열 슬라이스

범위 문법, 슬라이스가 원본과 같은 메모리를 가리킨다는 사실, 인덱스 방식의 문제, `&str` 매개변수, 한글의 바이트 경계를 차례로 보여 줍니다. `first_word`에 쓰인 `bytes.iter().enumerate()`는 (인덱스, 원소의 참조) 쌍을 차례로 꺼내는 반복자이고(ch13), `&item`은 참조를 벗겨 바이트 값을 꺼내는 패턴입니다(ch18).

```sh
cargo run -p p0-ch4-ownership --example 06_string_slices
```

```
1. 범위 문법으로 문자열의 일부를 빌립니다
    hello = 'hello', world = 'world'
    [..5] = 'hello', [6..] = 'world', [..] = 'hello world'
2. 슬라이스는 (시작 주소, 길이)만 가진 참조라서 복사 없이 만들어집니다
    world는 s의 6번 바이트를 가리키는가? true
    world의 길이: 5
3. 숫자(인덱스)를 돌려주면 원본과의 연결이 끊어집니다
    text = '', end = 5 (이제 의미 없는 값)
4. 슬라이스를 돌려주는 first_word
    첫 단어: 'hello'
5. 문자열 리터럴의 타입이 바로 &str입니다
    첫 단어: 'Hello,'
6. &str 매개변수는 String, String의 일부, 리터럴을 모두 받습니다
    &my_string        → 'hello'
    &my_string[..]    → 'hello'
    &my_string[6..]   → 'world'
    "good morning"    → 'good'
7. 인덱스는 글자가 아니라 바이트 단위입니다 (한글 주의)
    "안녕 세상"의 len(): 13
    &hangul[0..3] = '안'
    1번 바이트는 글자 경계인가? false
    첫 단어: '안녕'
```

### 07_array_slices — 배열 슬라이스

길이가 다른 배열(`[i32; 5]`, `[i32; 3]`)을 `&[i32]`를 받는 함수 하나로 처리하고, `&mut [i32]`로 배열의 뒷부분만 바꿉니다.

```sh
cargo run -p p0-ch4-ownership --example 07_array_slices
```

```
1. 범위 문법으로 배열의 일부를 빌립니다
    middle = [2, 3], 길이 = 2
2. &[i32] 매개변수는 배열 전체, 배열의 일부, 길이가 다른 배열을 모두 받습니다
    sum(&a)      = 15
    sum(&a[..2]) = 3
    sum(&a[3..]) = 9
    sum(&b)      = 60
3. 가변 슬라이스로 배열의 일부만 바꿉니다
    c = [1, 2, 0, 0, 0]
4. 슬라이스도 참조이므로 대여 규칙을 따릅니다
    view = [7, 8, 9]
    d = [70, 8, 9]
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch4-ownership
rustc --edition 2024 compile_fail/e0382_use_after_move.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요.

**에러 메시지 읽는 법**: `error[E0382]`의 코드는 `rustc --explain E0382`로 설명을 볼 수 있습니다. `-->` 줄은 파일:줄:열 위치입니다. 코드 아래의 `^^^`는 에러가 난 곳, `---`는 그 에러와 관련된 곳이고, 옆의 설명을 위에서 아래로 읽으면 "어디서 빌렸고 → 어디서 충돌했고 → 왜 아직 빌리는 중인지"가 이야기처럼 이어집니다. `help:`는 컴파일러가 제안하는 수정 방법입니다.

### 4.1 이동한 값을 사용 — E0382

파일: [`compile_fail/e0382_use_after_move.rs`](compile_fail/e0382_use_after_move.rs)

```
error[E0382]: borrow of moved value: `s1`
  --> compile_fail/e0382_use_after_move.rs:11:16
   |
 8 |     let s1 = String::from("hello");
   |         -- move occurs because `s1` has type `String`, which does not implement the `Copy` trait
 9 |     let s2 = s1;
   |              -- value moved here
10 |
11 |     println!("{s1}, world!");
   |                ^^ value borrowed here after move
```

- **원인**: 9번 줄에서 소유권이 `s2`로 이동했는데 11번 줄에서 `s1`을 사용했습니다. 메시지가 `String`이 `Copy`가 아니라서 이동이 일어났다는 것까지 알려 줍니다.
- **해결**: 두 변수가 모두 필요하면 `let s2 = s1.clone();`으로 복제합니다(컴파일러의 `help:`도 이것을 제안합니다). 읽기만 하면 된다면 `let s2 = &s1;`처럼 빌리는 쪽이 더 값쌉니다.
- 참고: `println!`처럼 값을 빌리는 곳에서 쓰면 "borrow of moved value", 함수 인자로 다시 넘기는 것처럼 값을 옮기는 곳에서 쓰면 "use of moved value"로 표시됩니다. 둘 다 같은 E0382입니다.

### 4.2 가변 참조를 동시에 두 개 — E0499

파일: [`compile_fail/e0499_two_mutable_borrows.rs`](compile_fail/e0499_two_mutable_borrows.rs)

```
error[E0499]: cannot borrow `s` as mutable more than once at a time
  --> compile_fail/e0499_two_mutable_borrows.rs:11:14
   |
10 |     let r1 = &mut s;
   |              ------ first mutable borrow occurs here
11 |     let r2 = &mut s;
   |              ^^^^^^ second mutable borrow occurs here
12 |
13 |     println!("{r1}, {r2}");
   |                -- first borrow later used here
```

- **원인**: `r1`이 13번 줄에서 다시 쓰이므로, 11번 줄에서 `r2`를 만들 때 `r1`의 가변 대여가 아직 살아 있습니다. 같은 값을 바꿀 수 있는 통로가 동시에 둘이 되면 데이터 경쟁이 생길 수 있습니다.
- **해결**: 두 가변 참조의 사용 구간이 겹치지 않게 합니다. `r1`을 다 쓴 뒤에 `r2`를 만들면 됩니다.

```rust
let r1 = &mut s;
println!("{r1}"); // r1의 마지막 사용
let r2 = &mut s;
println!("{r2}");
```

### 4.3 불변 참조가 살아 있는데 가변 참조 — E0502

파일: [`compile_fail/e0502_mutable_while_immutable.rs`](compile_fail/e0502_mutable_while_immutable.rs)

```
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
  --> compile_fail/e0502_mutable_while_immutable.rs:12:14
   |
10 |     let r1 = &s;
   |              -- immutable borrow occurs here
11 |     let r2 = &s;
12 |     let r3 = &mut s;
   |              ^^^^^^ mutable borrow occurs here
13 |
14 |     println!("{r1}, {r2}, and {r3}");
   |                -- immutable borrow later used here
```

- **원인**: 마지막 줄의 "immutable borrow later used here"가 핵심입니다. `r1`이 14번 줄에서 쓰이므로, `r3`를 만드는 12번 줄에서 불변 대여가 아직 끝나지 않았습니다.
- **해결**: 불변 참조를 다 쓴 뒤에 가변 참조를 만듭니다. [`05_borrow_scope.rs`](examples/05_borrow_scope.rs)의 1번 구역이 바로 고친 모습입니다.

### 4.4 불변 참조로 값을 수정 — E0596

파일: [`compile_fail/e0596_modify_through_shared_ref.rs`](compile_fail/e0596_modify_through_shared_ref.rs)

```
error[E0596]: cannot borrow `*some_string` as mutable, as it is behind a `&` reference
  --> compile_fail/e0596_modify_through_shared_ref.rs:15:5
   |
15 |     some_string.push_str(", world");
   |     ^^^^^^^^^^^ `some_string` is a `&` reference, so it cannot be borrowed as mutable
   |
help: consider changing this to be a mutable reference
   |
14 | fn change(some_string: &mut String) {
   |                         +++
```

- **원인**: `change`는 불변 참조 `&String`을 받았는데, `push_str`은 값을 바꾸는 메서드라 가변 참조가 필요합니다.
- **해결**: 2.8절의 세 곳을 모두 고칩니다. `let mut s`, `change(&mut s)`, `fn change(some_string: &mut String)`. `help:`가 알려 주는 매개변수만 고치면 다음 에러가 차례로 나타납니다. 호출부가 아직 `&s`이면 E0308(`types differ in mutability`)이, 변수가 `mut`가 아니면 E0596(``cannot borrow `s` as mutable, as it is not declared as mutable``)이 납니다.

### 4.5 함수 안에서 만든 값의 참조를 반환 — E0106

파일: [`compile_fail/e0106_dangling_reference.rs`](compile_fail/e0106_dangling_reference.rs)

```
error[E0106]: missing lifetime specifier
  --> compile_fail/e0106_dangling_reference.rs:13:16
   |
13 | fn dangle() -> &String {
   |                ^ expected named lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
...
help: instead, you are more likely to want to return an owned value
   |
13 - fn dangle() -> &String {
13 + fn dangle() -> String {
```

- **원인**: 첫 줄은 라이프타임 이야기라 낯설지만, `help:` 줄이 핵심을 말해 줍니다. "반환 타입에 빌린 값이 있는데, 빌려 올 대상이 없다." 매개변수로 받은 것이 없으니, 반환한 참조가 가리킬 수 있는 것은 함수 안의 `s`뿐인데 `s`는 함수가 끝날 때 drop됩니다.
- **해결**: 마지막 `help:`대로 `String`을 반환해 소유권을 넘깁니다(2.11절). 중간에 `'static`을 쓰라는 제안도 나오지만, 메시지 스스로 "흔하지 않다"고 말하듯 이 경우의 해결책이 아닙니다.

### 4.6 슬라이스가 살아 있는데 원본을 수정 — E0502

파일: [`compile_fail/e0502_clear_while_slice_alive.rs`](compile_fail/e0502_clear_while_slice_alive.rs)

```
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
  --> compile_fail/e0502_clear_while_slice_alive.rs:25:5
   |
23 |     let word = first_word(&s);
   |                           -- immutable borrow occurs here
24 |
25 |     s.clear();
   |     ^^^^^^^^^ mutable borrow occurs here
26 |
27 |     println!("첫 단어: {word}");
   |                         ---- immutable borrow later used here
```

- **원인**: `word`는 `s`의 일부를 가리키는 슬라이스이므로 `s`를 빌리고 있습니다. `clear()`는 `s`를 바꾸므로 가변 대여가 필요한데, `word`가 27번 줄에서 쓰이기 때문에 불변 대여가 아직 살아 있습니다. 이 규칙이 없다면 `word`는 지워진 글자를 가리키게 됩니다.
- **해결**: `word`를 다 쓴 뒤에 `clear()`를 호출하도록 순서를 바꿉니다. 2.12절의 인덱스 방식은 같은 실수를 실행 중에야 드러냈지만, 슬라이스 방식은 컴파일할 때 잡아 줍니다.

## 5. 연습 문제

### 연습 1. 이동 에러를 두 가지 방법으로 고치기

아래 코드는 E0382로 컴파일되지 않습니다. 원인을 설명하고, (가) `clone()`을 쓰는 방법과 (나) 참조를 쓰는 방법으로 각각 고쳐 보세요. 두 방법은 메모리 사용 면에서 어떻게 다른가요?

```rust
fn print_twice(s: String) {
    println!("{s} {s}");
}

fn main() {
    let name = String::from("러스트");
    print_twice(name);
    print_twice(name);
}
```

<details>
<summary>힌트</summary>

첫 번째 `print_twice(name)`에서 `name`의 소유권이 매개변수로 이동합니다. (가)는 첫 호출에 `name.clone()`을 넘기면 됩니다. 이때 힙 데이터가 한 벌 더 만들어집니다. (나)는 매개변수 타입을 `&str`로 바꾸고 `print_twice(&name)`으로 호출합니다. 힙 데이터는 한 벌뿐이고 함수는 빌려 쓰기만 합니다. 두 방법 모두 `러스트 러스트`가 두 줄 출력됩니다.

</details>

### 연습 2. drop 순서 예측하기

[`01_scope_and_drop.rs`](examples/01_scope_and_drop.rs)의 `main` 함수를 아래 코드로 바꾸고, 실행하기 **전에** 출력을 예측해 보세요. 특히 `[drop] p`가 어디에 찍힐지 생각해 보세요.

```rust
fn main() {
    let p = noisy("p");
    {
        let q = noisy("q");
        let r = p; // p의 값이 r로 이동합니다.
        println!("    블록 끝: {}, {}", q.name, r.name);
    }
    println!("    블록 밖");
}
```

<details>
<summary>힌트</summary>

`p`라는 변수는 `main` 끝까지 살아 있지만, 그 값의 소유자는 블록 안의 `r`로 바뀌었습니다. 값은 **현재 소유자**의 스코프가 끝날 때 drop됩니다. 또 블록 안에서는 나중에 선언된 `r`이 `q`보다 먼저 drop됩니다. 이동해 나간 `p`는 `main` 끝에서 아무것도 drop하지 않습니다. 예측한 뒤 `cargo run -p p0-ch4-ownership --example 01_scope_and_drop`으로 확인하세요.

</details>

### 연습 3. 줄 순서만 바꿔 대여 에러 고치기

아래 코드는 컴파일되지 않습니다. 에러 코드와 원인을 확인한 뒤, (가) 줄 순서만 바꿔서, (나) `first`에 참조 대신 값을 담아서 각각 고쳐 보세요.

```rust
fn main() {
    let mut scores = [70, 85, 90];
    let first = &scores[0];
    scores[2] = 100;
    println!("첫 점수: {first}");
    println!("전체: {scores:?}");
}
```

<details>
<summary>힌트</summary>

에러는 E0506(``cannot assign to `scores[_]` because it is borrowed``)입니다. 배열의 원소 하나를 빌려도 컴파일러는 배열 전체를 빌린 것으로 봅니다. `first`가 `println!`에서 쓰이기 때문에 `scores[2] = 100`을 실행하는 시점에 대여가 아직 살아 있습니다. (가)는 `println!("첫 점수: {first}");`를 대입보다 위로 옮깁니다. (나)는 `i32`가 `Copy` 타입이라는 점을 이용해 `let first = scores[0];`으로 값을 복사해 둡니다.

</details>

### 연습 4. 두 번째 단어 찾기

`fn second_word(s: &str) -> &str`를 작성하세요. 공백으로 구분된 두 번째 단어를 돌려주고, 단어가 하나뿐이면 빈 문자열 `""`을 돌려줍니다. 다음 호출로 확인하세요.

```rust
second_word("hello world rust") // "world"
second_word("hello")            // ""
second_word("안녕 세상 만세")     // "세상"
```

<details>
<summary>힌트</summary>

[`06_string_slices.rs`](examples/06_string_slices.rs)의 `first_word`를 재사용할 수 있습니다. `first_word(s)`의 길이가 `s.len()`과 같으면 공백이 없는 것입니다. 그렇지 않으면 첫 단어 바로 뒤 공백(1바이트)을 건너뛴 나머지 `&s[first.len() + 1..]`에 다시 `first_word`를 적용하면 됩니다. 공백 위치를 기준으로 자르므로 한글이 섞여도 글자 경계 문제가 생기지 않습니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [4. Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
  - [4.1 What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
  - [4.2 References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
  - [4.3 The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html)
- Rust By Example: [Scoping rules](https://doc.rust-lang.org/rust-by-example/scope.html) 장의 다음 절
  - [RAII](https://doc.rust-lang.org/rust-by-example/scope/raii.html): 스코프가 끝날 때 자원이 해제되는 원리와 `Drop`
  - [Ownership and moves](https://doc.rust-lang.org/rust-by-example/scope/move.html): 대입과 함수 인자에서의 이동
  - [Borrowing](https://doc.rust-lang.org/rust-by-example/scope/borrow.html), [Mutability](https://doc.rust-lang.org/rust-by-example/scope/borrow/mut.html), [Aliasing](https://doc.rust-lang.org/rust-by-example/scope/borrow/alias.html): 대여, `&mut`, 대여 규칙
  - 슬라이스는 [Primitives > Arrays and Slices](https://doc.rust-lang.org/rust-by-example/primitives/array.html)에 있습니다.
  - 예제 중 일부는 `Box`(ch14)와 구조체(ch5)를 씁니다. `Box<i32>`는 `String`처럼 힙 데이터를 소유하는 타입이라고 보고 읽으면 됩니다.
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`06_move_semantics`](https://github.com/rust-lang/rustlings/tree/main/exercises/06_move_semantics): 이동과 대여 연습. `Vec<i32>`(ch8)와 테스트(ch11)가 나오는데, `Vec<i32>`는 `String`처럼 힙 데이터를 소유하는 타입으로 보고 풀면 됩니다.
  - [`04_primitive_types`](https://github.com/rust-lang/rustlings/tree/main/exercises/04_primitive_types)의 `primitive_types4.rs`: 배열에서 슬라이스 만들기
- 에러 설명: `rustc --explain E0382`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
