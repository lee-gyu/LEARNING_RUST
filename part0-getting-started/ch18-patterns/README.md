# ch18. 패턴과 매칭

ch6에서 `match`, `if let`, `let...else`로 열거형을 다루면서 패턴을 처음 썼습니다. 사실 패턴은 그보다 훨씬 많은 곳에 있었습니다. `let x = 5;`의 `x`도, `for (i, v) in ...`의 `(i, v)`도 패턴입니다. 이 챕터에서는 패턴이 쓰이는 자리를 모두 모아 보고, 자리마다 "맞지 않을 수도 있는 패턴"을 허용하는지(반박 가능성)를 정리한 뒤, 패턴 문법 전체를 하나씩 익힙니다. ch6이 이 챕터로 미룬 기본 바인딩 모드와 `ref`, `..`, `|`, 범위 패턴, 매치 가드, `@` 바인딩, `while let`도 여기서 다룹니다.

이 저장소의 `ch18`은 The Book의 **19장**에 해당합니다. Book 14장(Cargo와 crates.io)을 Part 3으로 옮겼기 때문에 저장소 챕터 번호가 Book 장 번호보다 하나 작습니다. 이 문서에서 "N장"은 The Book의 장을, `chN`은 이 저장소의 챕터를 가리킵니다.

- 패키지: `p0-ch18-patterns`
- 기준 문서: [The Book 19장 Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html)
- 확인한 툴체인: rustc 1.100.0, Edition 2024. 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 패턴이 쓰이는 일곱 자리(`match` 갈래, `let`, `if let` 사슬, `while let`, `for`, 함수·클로저 매개변수, `let...else`)를 나열하고, `while let`과 let 체인(`if let ... && 조건`)으로 코드를 작성할 수 있다.
2. 반박 가능한 패턴과 반박 불가능한 패턴을 구분하고, 자리마다 어느 쪽을 요구하는지와 잘못 썼을 때 나오는 E0005 에러와 `irrefutable_let_patterns` 경고를 읽고 고칠 수 있다.
3. 리터럴, 이름 있는 변수, 상수, `|`, 범위 패턴으로 값을 고르고, 구조체·열거형·중첩 구조·튜플·배열·슬라이스를 구조 분해할 수 있다. 패턴 안의 변수 이름이 바깥 변수를 가리는 함정을 설명할 수 있다.
4. 참조를 패턴과 맞출 때 바인딩이 참조가 되는 규칙(기본 바인딩 모드)과 `&` 패턴, `ref`/`ref mut`를 설명하고, `_`·`_x`·`..`로 값을 무시할 때 이동과 drop 시점이 어떻게 달라지는지 설명할 수 있다.
5. 매치 가드와 `@` 바인딩으로 패턴만으로는 표현할 수 없는 조건을 작성하고, 가드가 완전성 검사에 들어가지 않는다는 점과 `|`와 함께 쓸 때의 우선순위를 설명할 수 있다.

## 2. 핵심 개념

### 2.1 패턴은 "모양 검사"와 "값 꺼내기"를 한 번에 합니다

패턴(pattern)은 값의 모양을 적은 것입니다. 패턴을 값과 맞춰 보면 두 가지 일이 동시에 일어납니다.

- **검사**: 값이 그 모양인지 확인합니다. `Some(n)`은 "`Some`인가?", `(0, y)`는 "첫째 원소가 0인 튜플인가?"를 묻습니다.
- **바인딩**: 모양이 맞으면 패턴 안의 이름(`n`, `y`)에 값의 해당 부분을 묶습니다.

ch6에서 본 것처럼, 검사와 꺼내기가 한 문법으로 붙어 있기 때문에 "검사하지 않고 꺼내는" 실수를 할 수 없습니다. 이 챕터에서 배우는 문법은 모두 이 두 가지 일을 더 정교하게 하는 방법입니다.

### 2.2 패턴이 쓰이는 일곱 자리

| 자리 | 모양 | 예 ([`01_pattern_places.rs`](examples/01_pattern_places.rs)) |
| --- | --- | --- |
| `match` 갈래 | `match 값 { 패턴 => 식, ... }` | `Some(n) => format!("{n}칸 이동")` |
| `let` 문 | `let 패턴 = 식;` | `let (a, b, c) = (1, 2, 3);` |
| `if let` 사슬 | `if let 패턴 = 식 { ... } else if ... else { ... }` | `if let Some(code) = coupon { ... }` |
| `while let` | `while let 패턴 = 식 { ... }` | `while let Some(top) = stack.pop() { ... }` |
| `for` | `for 패턴 in 반복자 { ... }` | `for (index, fruit) in fruits.iter().enumerate() { ... }` |
| 함수·클로저 매개변수 | `fn f(패턴: 타입)`, `\|패턴\| 식` | `fn print_coordinates(&(x, y): &(i32, i32))` |
| `let...else` | `let 패턴 = 식 else { 흐름 끊기 };` | `let Some((name, age)) = line.split_once(':') else { ... };` |

#### `let`의 왼쪽은 처음부터 패턴이었습니다

`let x = 5;`의 `x`는 "무엇이든 맞고, 맞은 값을 `x`에 묶는다"는 가장 단순한 패턴입니다. 그래서 `let (a, b, c) = (1, 2, 3);`처럼 왼쪽에 튜플 패턴을 쓰면 세 변수를 한 번에 만들 수 있습니다. 이때 패턴과 값의 모양이 같아야 합니다. 원소 개수가 다르면 타입이 다른 것이므로 E0308이 납니다.

```
error[E0308]: mismatched types
...
2 |     let (x, y) = (1, 2, 3);
  |         ^^^^^^   --------- this expression has type `({integer}, {integer}, {integer})`
  |         |
  |         expected a tuple with 3 elements, found one with 2 elements
```

#### `if let`, `else if`, `else if let` 사슬

ch6에서 본 `if let`은 `else if`, `else if let`과 섞어 쓸 수 있고, 각 조건이 **서로 다른 값**을 검사해도 됩니다. 값 하나를 여러 패턴과 맞춰 보는 `match`로는 이런 모양을 만들 수 없습니다.

```rust
fn discount(coupon: Option<&str>, is_member_day: bool, age_text: &str) -> String {
    if let Some(code) = coupon {
        format!("쿠폰 {code} 할인")
    } else if is_member_day {
        String::from("회원의 날 10% 할인")
    } else if let Ok(age) = age_text.parse::<u32>() {
        if age >= 65 { String::from("경로 우대 할인") } else { String::from("할인 없음") }
    } else {
        String::from("나이를 확인할 수 없음")
    }
}
```

`if let`은 `match`와 달리 **완전성 검사를 하지 않습니다.** 마지막 `else`를 빠뜨려도 컴파일러는 알려 주지 않습니다.

#### let 체인 (Edition 2024)

위 코드에서 `age >= 65` 비교가 한 단계 안쪽에 들어간 이유는, `if let Ok(age)`가 묶은 `age`를 그 블록 안에서만 쓸 수 있기 때문입니다. Edition 2024부터는 `if`와 `while`의 조건에서 `let` 패턴과 `bool` 조건을 `&&`로 이을 수 있습니다. 이것을 let 체인(let chains)이라고 부릅니다.

```rust
fn senior_discount(age_text: &str) -> &'static str {
    if let Ok(age) = age_text.parse::<u32>()
        && age >= 65
    {
        "경로 우대 할인"
    } else {
        "할인 없음"
    }
}
```

- 앞의 `let`이 묶은 `age`를 `&&` 뒤의 조건에서 바로 쓸 수 있습니다.
- 사슬의 고리 하나라도 실패하면(숫자가 아니거나 65 미만이면) `else`로 갑니다.
- `if let Some(x) = a && let Some(y) = b && x < y`처럼 `let`을 여러 번 이을 수도 있습니다.
- `&&`로만 이을 수 있습니다. `||`를 쓰면 `` `||` operators are not supported in let chain conditions `` 에러가 납니다.
- Edition 2021 이하에서는 `let chains are only allowed in Rust 2024 or later` 에러가 납니다. 에디션별 차이는 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md)에서 다룹니다.

The Book 19.1절에는 "`if let Ok(age) = age && age > 30`처럼 두 조건을 합칠 수 없다"는 설명이 있지만, 이 저장소의 툴체인(Edition 2024)에서는 위처럼 합칠 수 있습니다.

#### `while let`: 패턴이 맞는 동안 반복

`while let`은 `if let`을 패턴이 맞는 동안 되풀이합니다. 패턴이 맞지 않는 순간 반복이 끝납니다.

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("꺼낸 값: {top}"); // 3, 2, 1 순서
}
```

`Vec::pop`은 꺼낼 원소가 있으면 `Some(원소)`, 비었으면 `None`을 돌려주므로 "스택이 빌 때까지 꺼내기"가 이 한 줄로 끝납니다. ch15의 채널에서도 `while let Ok(value) = rx.recv() { ... }`처럼 모든 송신자가 drop되어 `recv`가 `Err`를 돌려줄 때까지 받을 수 있습니다. `loop` 안에서 `match`로 `None`일 때 `break`하던 코드를 볼 때마다 `while let`으로 줄일 수 있는지 살펴보세요. Clippy도 그런 `loop`를 보면 `while let`을 제안합니다.

#### `for`, 함수와 클로저의 매개변수

`for` 바로 뒤도 패턴입니다. `enumerate()`가 내놓는 `(인덱스, 원소)` 튜플을 `(index, fruit)` 패턴이 받아서 바로 풉니다.

함수 매개변수의 이름도 패턴이므로, 튜플을 받는 자리에서 곧바로 풀 수 있습니다. 클로저 매개변수도 같습니다.

```rust
fn print_coordinates(&(x, y): &(i32, i32)) {
    println!("현재 위치: ({x}, {y})");
}

let areas: Vec<i32> = rects.iter().map(|&(w, h)| w * h).collect();
```

`&(x, y)`의 `&`는 "참조를 벗기고 안의 튜플을 푼다"는 패턴으로 2.6절에서 자세히 다룹니다.

### 2.3 반박 가능성: 실패할 수 있는 패턴인가

ch6에서 일반 `let`에 `Some(port)`를 쓰면 E0005가 나는 것을 봤습니다. 이 규칙을 일반화하면 다음과 같습니다.

- **반박 불가능한 패턴(irrefutable pattern)**: 그 타입의 어떤 값이 와도 맞는 패턴입니다. `x`, `(a, b)`, `(x, _)`, 필드마다 변수를 두는 구조체 패턴 `Point { x, y }`, 필드 하나짜리 튜플 구조체 `Meters(m)`이 그렇습니다.
- **반박 가능한 패턴(refutable pattern)**: 맞지 않는 값이 있는 패턴입니다. `Some(x)`(`None`과 맞지 않음), `Ok(n)`, `(y, 0)`(둘째 값이 0이 아닌 튜플과 맞지 않음), `1..=5`가 그렇습니다.

판단 기준은 ch6에서 배운 `match`의 완전성 검사와 같습니다. **그 패턴 하나만 갈래로 둔 `match`가 완전하면 반박 불가능**합니다. 실제로 `let (y, 0) = pair;`라고 쓰면, 컴파일러는 `match`의 E0004처럼 덮지 못한 값을 계산해서 알려 줍니다.

```
error[E0005]: refutable pattern in local binding
...
3 |     let (y, 0) = pair;
  |         ^^^^^^ patterns `(_, i32::MIN..=-1_i32)` and `(_, 1_i32..=i32::MAX)` not covered
```

#### 자리마다 요구하는 것이 다릅니다

| 자리 | 요구 | 반박 가능한 패턴을 쓰면 | 반박 불가능한 패턴을 쓰면 |
| --- | --- | --- | --- |
| `let`, `for`, 함수·클로저 매개변수 | 반박 불가능 | E0005 에러 | 정상 |
| `if let`, `while let`, `let...else` | 반박 가능해야 의미가 있음 | 정상 | 경고 |
| `match` 갈래 | 갈래 전체가 모든 값을 덮어야 함 | 정상(덮지 못한 값이 있으면 E0004) | 마지막 갈래면 정상, 중간이면 아래 갈래에 `unreachable pattern` 경고 |

이유는 하나입니다. **패턴이 맞지 않을 때 할 일을 적을 곳이 있는가**입니다.

- `let x = ...;` 다음 줄은 `x`가 있다고 믿고 실행됩니다. 함수 본문도 매개변수에 값이 있다고 믿고, `for` 본문도 원소가 패턴과 맞았다고 믿습니다. 맞지 않을 때 갈 곳이 없으므로, 처음부터 맞지 않을 수 있는 패턴을 거부합니다.

  ```
  error[E0005]: refutable pattern in `for` loop binding
  error[E0005]: refutable pattern in function argument
  error[E0005]: refutable pattern in closure argument
  ```

- `if let`, `while let`, `let...else`는 거꾸로 "맞지 않을 수도 있으니 그때는 `else`로 가라(반복을 끝내라)"라는 문법입니다. 언제나 맞는 패턴을 쓰면 `else`가 절대 실행되지 않으므로 컴파일러가 경고합니다. 에러가 아니라 경고인 것은, 코드가 틀린 것은 아니고 쓸모없을 뿐이기 때문입니다.

  ```
  warning: irrefutable `if let` pattern
  ...
     |
  20 |     if let Meters(m) = distance {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^
     |
     = note: this pattern will always match, so the `if let` is useless
     = help: consider replacing the `if let` with a `let`
     = note: `#[warn(irrefutable_let_patterns)]` on by default
  ```

  `while let`이면 `` irrefutable `while let` pattern ``(반복이 끝나지 않음), `let...else`면 `` unreachable `else` clause ``로 같은 경고가 나옵니다. [`02_refutability.rs`](examples/02_refutability.rs)의 `always_matches` 함수는 이 경고가 나는 코드를 일부러 쓰고, 의도한 경고라는 뜻으로 `#[expect(irrefutable_let_patterns)]`를 달아 두었습니다. 그 줄을 지우고 빌드하면 위 경고가 나옵니다.

- `match`는 앞쪽 갈래에 반박 가능한 패턴을 두고, 마지막 갈래에 반박 불가능한 패턴(`other`, `_`)을 두어 나머지를 받는 것이 일반적인 모양입니다.

### 2.4 값을 고르는 패턴: 리터럴, 변수, 상수, `|`, 범위

#### 리터럴

`1`, `'a'`, `"start"`, `true` 같은 리터럴은 값이 같을 때만 맞습니다. 여러 값을 튜플로 묶어 리터럴과 맞추면 조건 조합을 표처럼 적을 수 있습니다.

```rust
fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => String::from("FizzBuzz"),
        (0, _) => String::from("Fizz"),
        (_, 0) => String::from("Buzz"),
        _ => n.to_string(),
    }
}
```

갈래는 위에서부터 맞춰 보므로 `(0, 0)`이 맨 위에 있어야 합니다. `(0, _)`와 `(_, 0)` 아래로 내리면 15가 `(0, _)`에 먼저 걸려 `Fizz`가 되고, `(0, 0)` 갈래에는 어떤 값도 도달할 수 없으므로 컴파일러가 `unreachable pattern` 경고를 냅니다.

#### 이름 있는 변수와 섀도잉 함정

패턴 안에 쓴 이름은 **언제나 새 변수**입니다. 바깥에 같은 이름의 변수가 있어도 그 값과 비교하지 않습니다.

```rust
let x = Some(5);
let y = 10;
match x {
    Some(50) => println!("50을 받았습니다"),
    Some(y) => println!("맞았습니다, y = {y}"), // y = 5 (새 변수)
    _ => println!("기본 갈래, x = {x:?}"),
}
println!("match 뒤: x = {x:?}, y = {y}"); // y = 10
```

`Some(y)`의 `y`는 "바깥 `y`(10)와 같은 `Some`"이 아니라 "아무 `Some`이나 받아 안의 값을 새 `y`에 묶는다"입니다. 그래서 `Some(5)`와 맞고, 갈래 안에서만 바깥 `y`를 가립니다(ch3의 섀도잉). 갈래가 끝나면 바깥 `y`가 다시 보입니다.

이 함정은 "변수에 담긴 값과 같은지" 비교하려고 할 때 자주 빠집니다. 다음 코드는 `n`이 무엇이든 첫 갈래로 가서, 7인데도 `맞음 7`을 출력합니다. 다행히 컴파일러가 경고로 짚어 줍니다.

```rust
let expected = 3;
let n = 7;
match n {
    expected => println!("맞음 {expected}"), // 바깥 expected(3)와 비교하지 않고, n을 새 expected에 묶음
    _ => println!("틀림"),
}
```

```
warning: unreachable pattern
...
5 |         expected => println!("맞음 {expected}"),
  |         -------- matches any value
6 |         _ => println!("틀림"),
  |         ^ no value can reach this
  |
note: there is a binding of the same name; if you meant to pattern match against the value of that binding, that is a feature of constants that is not available for `let` bindings
```

`note:`가 해결 방법 두 가지를 알려 줍니다. 비교할 값이 고정돼 있다면 **상수**를 쓰고, 실행 중에 정해지는 값이라면 2.8절의 **매치 가드**를 씁니다.

#### 상수

패턴 안의 이름이 `const`로 정의한 상수(ch3)이면, 새 변수를 만들지 않고 그 상수의 값과 비교합니다. 상수 이름은 대문자로 쓰는 관례가 있어서 변수와 쉽게 구분됩니다.

```rust
const LUCKY: u32 = 7;

match n {
    LUCKY => println!("{n}: 행운의 숫자!"),
    _ => println!("{n}: 보통 숫자"),
}
```

#### `|`: 여러 패턴 중 하나

`|`는 "이 패턴 또는 저 패턴"입니다. 다른 패턴의 안쪽에도 쓸 수 있어서, `Some(6 | 7)`은 `Some(6) | Some(7)`과 같습니다.

```rust
let kind = match day {
    Some(6 | 7) => "주말",
    Some(1..=5) => "평일",
    Some(_) => "없는 요일",
    None => "요일 정보 없음",
};
```

#### 범위 패턴

`시작..=끝`은 양 끝을 포함한 범위입니다. 정수와 `char`에 쓸 수 있습니다.

```rust
fn classify(c: char) -> &'static str {
    match c {
        '가'..='힣' => "한글 음절",            // 현대 한글 음절 전체(U+AC00~U+D7A3)
        'a'..='z' | 'A'..='Z' => "영문자",
        '0'..='9' => "숫자",
        _ => "기타",
    }
}
```

컴파일러는 범위가 덮는 구간을 계산해 완전성을 검사합니다. 그래서 `u8`의 0~255를 범위로 빠짐없이 나누면 `_` 없이도 완전한 `match`가 됩니다. 하나라도 빠지면 ch6의 E0004처럼 빠진 구간을 알려 줍니다.

```rust
match n {
    0..=9 => 1,
    10..=99 => 2,
    100..=u8::MAX => 3,
}
```

끝을 포함하지 않는 범위 `시작..끝`과 한쪽이 열린 범위 `시작..`, `..=끝`도 패턴에 쓸 수 있습니다. 성적 구간처럼 경계가 이어지는 경우에 `80..90`, `90..`으로 쓰면 89와 90 사이에 빈틈이 생길 걱정이 없습니다.

```rust
match score {
    90.. => 'A',
    80..90 => 'B',
    70..80 => 'C',
    0..70 => 'F',
}
```

시작이 끝보다 큰 빈 범위(`5..=1`)는 E0030(`lower bound for range pattern must be less than or equal to upper bound`)으로 거부됩니다. 전체 코드는 [`03_literals_ranges.rs`](examples/03_literals_ranges.rs)에 있습니다.

### 2.5 구조 분해: 만들 때의 모양 그대로 풀기

구조 분해(destructuring)는 값을 만들 때 쓴 문법을 패턴 자리에 그대로 써서 값을 부분으로 나누는 것입니다.

#### 구조체

```rust
struct Point { x: i32, y: i32 }

let p = Point { x: 3, y: -7 };
let Point { x: a, y: b } = p; // 필드 x를 변수 a에, 필드 y를 변수 b에
let Point { x, y } = p;       // 변수 이름이 필드 이름과 같으면 줄여 씀
```

- `필드: 변수` 순서입니다. 값을 만들 때의 `Point { x: 3 }`이 "필드 x에 3을 넣는다"였듯이, 패턴의 `Point { x: a }`는 "필드 x의 값을 a에 꺼낸다"입니다.
- `Point { x, y }`는 ch5에서 배운 필드 초기화 축약의 반대 방향입니다.
- 구조체 패턴은 **모든 필드를 적어야** 합니다. 필요 없는 필드는 `..`로 넘깁니다(2.7절). 빠뜨리면 E0027입니다(4.4절).

필드 자리에 리터럴을 쓰면 "이 필드는 이 값이어야 한다"는 조건이 됩니다.

```rust
match p {
    Point { x: 0, y: 0 } => String::from("원점"),
    Point { x, y: 0 } => format!("x축 위 (x = {x})"),
    Point { x: 0, y } => format!("y축 위 (y = {y})"),
    Point { x, y } => format!("어느 축에도 없음 ({x}, {y})"),
}
```

#### 열거형과 중첩 구조

열거형은 ch6에서 본 대로 배리언트의 모양(데이터 없음, 튜플형, 구조체형)에 맞춰 패턴을 씁니다. 배리언트 안에 다른 열거형이 있으면 패턴도 중첩해서 **한 번에 안쪽까지** 풉니다.

```rust
enum Color { Rgb(u8, u8, u8), Hsv(u16, u8, u8) }
enum Message { Quit, Move { x: i32, y: i32 }, Write(String), ChangeColor(Color) }

match msg {
    Message::Quit => String::from("종료"),
    Message::Move { x, y } => format!("이동: ({x}, {y})"),
    Message::Write(text) => format!("글자 쓰기: {text}"),
    Message::ChangeColor(Color::Rgb(r, g, b)) => format!("색 변경: RGB({r}, {g}, {b})"),
    Message::ChangeColor(Color::Hsv(h, s, v)) => format!("색 변경: HSV({h}, {s}, {v})"),
}
```

`match`를 두 번 겹쳐 쓰지 않고, `ChangeColor`이면서 `Rgb`인 경우를 한 갈래로 적었습니다. 완전성 검사도 중첩된 안쪽까지 이루어지므로, `Hsv` 갈래를 빠뜨리면 ``pattern `Message::ChangeColor(Color::Hsv(_, _, _))` not covered``처럼 알려 줍니다.

튜플형 배리언트의 원소 개수를 틀리면 E0023이 납니다. 아래는 원소가 둘인 `Msg::Move(i32, i32)`를 `Msg::Move(x)`로 맞춘 경우로, 남는 자리를 `_`로 채우라고 알려 줍니다.

```
error[E0023]: this pattern has 1 field, but the corresponding tuple variant has 2 fields
...
help: use `_` to explicitly ignore each field
  |
4 |     match m { Msg::Move(x, _) => println!("{x}") }
  |                          +++
```

#### 튜플과 구조체 섞기

패턴은 얼마든지 섞고 중첩할 수 있습니다.

```rust
let ((feet, inches), Point { x, y }) = ((5, 11), Point { x: 3, y: -10 });
```

전체 코드는 [`04_destructuring.rs`](examples/04_destructuring.rs)에 있습니다.

### 2.6 바인딩 모드: 가져갈까, 빌릴까

패턴의 바인딩도 대입과 같은 소유권 규칙을 따릅니다(ch6 2.9절). 이 절에서는 그 규칙을 정확히 정리합니다.

#### 값을 맞추면 이동하거나 복사합니다

```rust
let User { name, age } = user; // name: String(이동), age: u32(복사)
```

`name` 필드가 빠져나갔으므로 `user`는 부분 이동된 값이 되어 통째로 쓸 수 없습니다(`` borrow of partially moved value: `user` ``, E0382).

#### 참조를 맞추면 바인딩이 참조가 됩니다 (기본 바인딩 모드)

```rust
let User { name, age } = &user;     // name: &String, age: &u32
let User { name, age } = &mut user; // name: &mut String, age: &mut u32
```

오른쪽은 `&User`인데 패턴 `User { .. }`에는 `&`가 없습니다. 이렇게 **참조 값을 참조가 아닌 패턴**(구조체, 튜플, 열거형, 슬라이스 패턴)과 맞추면, 컴파일러는 참조를 따라가서 안쪽을 맞춘 뒤 그 안의 바인딩을 모두 참조로 만듭니다. 이 동작을 기본 바인딩 모드(default binding mode)라고 합니다. 아무것도 이동하지 않으므로 `user`는 그대로 남습니다.

이 규칙은 깊이 중첩된 곳에도, `if let`과 `for`에도 똑같이 적용됩니다.

```rust
if let Some(s) = &nickname { /* s: &String */ }
for (who, score) in &scores { /* who: &String, score: &i32 */ }
```

ch6에서 `&self` 메서드의 `match self` 안에서 바인딩이 참조였던 것도 이 규칙 때문입니다. [`05_binding_modes.rs`](examples/05_binding_modes.rs)는 `std::any::type_name`(타입 이름을 문자열로 알려 주는 표준 라이브러리 함수)으로 각 바인딩의 실제 타입을 출력해 보여 줍니다.

바인딩이 참조이므로 값을 비교하거나 바꿀 때는 `*`로 따라가야 하는 경우가 있습니다. `*age += 1`이 그 예이고, 연습 2에서 한 번 더 만납니다.

#### `&` 패턴: 참조를 벗겨 값을 꺼냅니다

반대로 패턴에 `&`를 쓰면 "참조를 벗기고 안의 값과 맞춘다"는 뜻입니다.

```rust
let numbers = [1, 2, 3];
for &n in &numbers { /* n: i32 */ }
let doubled: Vec<i32> = numbers.iter().map(|&n| n * 2).collect();
```

벗긴 값은 바인딩으로 옮겨지므로 `Copy` 타입에만 쓸 수 있습니다. `String`처럼 `Copy`가 아닌 값에 쓰면 "빌린 것에서 값을 꺼내 가져가려 한다"는 E0507이 납니다.

```
error[E0507]: cannot move out of a shared reference
...
9 |     for &x in &v { println!("{x}"); }
  |          -    ^^
  |          |
  |          data moved here because `x` has type `String`, which does not implement the `Copy` trait
```

#### `ref`와 `ref mut`

`ref`는 "이 바인딩은 값을 옮기지 말고 참조로 빌려라"라고 바인딩 하나에 직접 붙이는 키워드입니다. `ref mut`은 가변 참조로 빌립니다.

```rust
match nickname {                 // 값을 맞추지만
    Some(ref s) => println!("{s}"), // s만 &String으로 빌림
    None => println!("별명 없음"),
}
println!("{nickname:?}");          // 이동이 없었으므로 쓸 수 있음

if let Some(ref mut lv) = level { *lv += 1; }
```

`match nickname { Some(ref s) => ... }`는 `match &nickname { Some(s) => ... }`와 같은 효과입니다. 기본 바인딩 모드가 생기기 전에 쓰인 Rust 코드에는 `&` 패턴과 `ref`가 아주 많이 나옵니다. 위의 `let User { name, age } = &user;`를 옛 방식대로 모두 적으면 `let &User { ref name, ref age } = &user;`입니다. 지금은 대부분 기본 바인딩 모드로 충분하지만, ch6과 이 챕터의 E0382 에러에서 컴파일러가 `help:`로 `ref`를 제안하기 때문에 읽을 줄은 알아야 합니다.

`ref`가 지금도 쓸모 있는 곳은 **한 패턴 안에서 어떤 필드는 빌리고 어떤 필드는 복사하고 싶을 때**입니다. `&user`를 맞추면 `age`까지 `&u32`가 되지만, `ref`를 쓰면 원하는 필드만 빌릴 수 있습니다.

```rust
let User { ref name, age } = user; // name: &String, age: u32
```

#### Edition 2024: 암묵적으로 빌리는 패턴 안에서는 `ref`를 쓸 수 없습니다

기본 바인딩 모드로 이미 참조가 되는 자리에 `ref`를 또 붙이면 Edition 2024에서는 에러입니다. `mut`과 `&`도 마찬가지입니다.

```rust
match &nickname {
    Some(ref s) => println!("{s}"), // 에러
    None => {}
}
```

```
error: cannot explicitly borrow within an implicitly-borrowing pattern
...
4 |         Some(ref s) => println!("{s}"),
  |              ^^^ explicit `ref` binding modifier not allowed when implicitly borrowing
...
help: remove the unnecessary binding modifier
```

`&nickname`을 `Some(..)` 패턴과 맞추는 순간 안쪽 바인딩은 이미 참조이므로 `ref`는 쓸모가 없습니다. `help:`대로 `ref`를 지우면 됩니다. 옛날 코드를 옮겨 올 때 자주 만나는 에러입니다. 에디션 2024에서 바뀐 바인딩 모드 규칙의 상세는 Part 9 `ch6-patterns`에서 다룹니다.

| 맞추는 값 | 패턴 | 바인딩 |
| --- | --- | --- |
| `user` (값) | `User { name, age }` | `String`(이동), `u32`(복사) |
| `&user` | `User { name, age }` | `&String`, `&u32` |
| `&mut user` | `User { name, age }` | `&mut String`, `&mut u32` |
| `user` (값) | `User { ref name, age }` | `&String`, `u32`(복사) |
| `&numbers`의 원소 `&i32` | `&n` | `i32`(복사) |

### 2.7 값 무시하기: `_`, 중첩된 `_`, `_x`, `..`

#### `_`: 값 하나를 통째로 무시

ch6에서 `match`의 마지막 갈래에 쓴 `_`는 어느 패턴 자리에든 쓸 수 있습니다. 함수 매개변수에 쓰면 "이 인자는 받지만 쓰지 않는다"는 뜻이 됩니다. 함수 모양이 정해져 있는데(다른 코드와 약속했거나, ch10의 트레이트 메서드를 구현할 때) 그 값이 필요 없을 때 씁니다.

```rust
fn on_click(_: (i32, i32), label: &str) { /* 클릭 위치는 쓰지 않음 */ }
```

#### 중첩된 `_`: 일부만 무시

```rust
match (setting, new_setting) {
    (Some(_), Some(_)) => println!("이미 설정된 값을 덮어쓸 수 없습니다"),
    _ => setting = new_setting,
}
let (first, _, third, _, fifth) = numbers;
```

`Some(_)`는 "안의 값이 무엇이든 `Some`이기만 하면"이라는 뜻입니다. 안의 값이 필요 없고 배리언트만 중요할 때 씁니다.

#### `_x`: 묶기는 하지만 경고만 끔

이름을 `_`로 시작하면 "쓰지 않는 변수" 경고(`warning: unused variable`)가 나지 않습니다. 작성 중인 코드에서 잠시 변수를 남겨 둘 때 씁니다. 겉보기는 `_`와 비슷하지만 **`_x`는 보통 변수**라서 값을 묶습니다. 이 차이가 두 군데서 드러납니다.

1. **이동**: `_`는 아무것도 묶지 않으므로 값이 이동하지 않습니다. `_x`는 값을 묶으므로 `Copy`가 아닌 값이면 이동합니다.

   ```rust
   let pair = (String::from("안녕"), 3);
   let (_, count) = pair;       // 이동 없음. pair를 계속 쓸 수 있음
   let (_greeting, count) = pair; // String이 _greeting으로 이동 → 뒤에서 pair를 쓰면 E0382 (4.5절)
   ```

2. **drop 시점**: `let _ = 식;`에서 새로 만든 값은 어디에도 묶이지 않으므로 **그 문장이 끝나자마자** drop됩니다. `let _guard = 식;`은 변수가 스코프를 벗어날 때 drop됩니다.

   ```rust
   {
       let _ = Noisy("_에 넘긴 값");          // 여기서 바로 drop
       let _guard = Noisy("_guard에 묶은 값"); // 블록 끝에서 drop
       println!("블록의 마지막 줄");
   }
   ```

   ch15의 `MutexGuard`처럼 살아 있는 동안 효과가 있는 값(잠금)을 `_`에 넘기면 잠금이 곧바로 풀립니다. 표준 라이브러리의 잠금은 컴파일러가 `let_underscore_lock` 에러(`non-binding let on a synchronization lock`)로 막아 주지만, 직접 만든 타입은 아무 경고 없이 drop됩니다.

| 패턴 | 값을 묶는가 | `Copy`가 아닌 값의 이동 | 새로 만든 값의 drop 시점 | "쓰지 않음" 경고 |
| --- | --- | --- | --- | --- |
| `_` | 아니요 | 없음 | 그 문장 끝 | 없음 |
| `_x` | 예 | 있음 | `_x`의 스코프 끝 | 없음 |
| `x` | 예 | 있음 | `x`의 스코프 끝 | 쓰지 않으면 경고 |

#### `..`: 나머지를 한꺼번에 무시

```rust
let Point3 { x, .. } = p;        // 구조체: x만 꺼내고 나머지 필드는 무시
let (first, .., last) = numbers; // 튜플: 처음과 마지막만
match point {
    Point3 { x: 0, .. } => println!("x가 0인 점"),
    Point3 { y, z, .. } => println!("x가 0이 아닌 점 (y = {y}, z = {z})"),
}
```

필드가 많은 구조체에서 `_`를 필드마다 쓰지 않아도 되고, 나중에 필드가 늘어나도 패턴을 고칠 필요가 없습니다. 다만 그 말은 새 필드를 놓쳐도 컴파일러가 알려 주지 않는다는 뜻이기도 합니다. ch6에서 열거형에 `_`를 조심해서 쓴 것과 같은 이유로, 모든 필드를 확인해야 하는 곳에서는 `..`보다 필드를 모두 적는 편이 안전합니다.

`..`는 **한 패턴에 한 번만** 쓸 수 있습니다(4.6절). 전체 코드는 [`06_ignoring.rs`](examples/06_ignoring.rs)에 있습니다.

### 2.8 매치 가드: 패턴 뒤에 붙이는 조건

매치 가드(match guard)는 `match` 갈래의 패턴 뒤에 붙이는 `if 조건`입니다. **패턴이 맞고 조건도 참일 때만** 그 갈래가 선택되고, 조건이 거짓이면 다음 갈래로 넘어갑니다. ch9에서 Rustlings를 안내하며 미뤄 둔 `x if x < 0 =>` 같은 갈래가 이것입니다.

```rust
match num {
    Some(x) if x % 2 == 0 => println!("{x}: 짝수"),
    Some(x) => println!("{x}: 홀수"), // Some(5)는 첫 갈래의 가드가 거짓이라 여기로 옴
    None => println!("값 없음"),
}
```

"짝수인가"는 패턴으로 표현할 수 없습니다. 가드는 패턴 안에서 묶인 변수(`x`)와 바깥 변수를 모두 쓸 수 있는 임의의 `bool` 식이라서, 패턴이 할 수 없는 조건을 맡습니다.

#### 바깥 변수와 비교하기

2.4절의 섀도잉 함정은 가드로 해결합니다. 패턴에는 새 이름을 쓰고, 가드에서 바깥 변수와 비교합니다. 가드는 패턴이 아니므로 새 변수를 만들지 않습니다.

```rust
let target = 10;
match x {
    Some(n) if n == target => println!("{n}: 목표값과 같습니다"),
    Some(n) => println!("{n}: 목표값이 아닙니다"),
    None => println!("값 없음"),
}
```

#### `|`와 가드의 우선순위

가드는 `|`로 묶인 패턴 **전체**에 걸립니다.

```rust
let x = 4;
let y = false;
match x {
    4 | 6 | 8 if y => println!("yes"),
    _ => println!("no"),
}
// no
```

이 갈래는 `(4 | 6 | 8) if y`로 읽힙니다. `4 | 6 | (8 if y)`였다면 `x`가 4이므로 `yes`가 출력됐을 것입니다.

#### 가드는 완전성 검사에 들어가지 않습니다

컴파일러는 가드 조건이 참일지 거짓일지 따지지 않습니다. 가드가 붙은 갈래는 "맞지 않을 수도 있는 갈래"로 보기 때문에, 사람이 보기에 모든 경우를 덮어도 가드만으로는 완전성 검사를 통과할 수 없습니다.

```rust
match temp {
    t if t < 0 => "영하",
    t if t >= 0 => "영상", // E0004 (4.2절)
}
```

그래서 가드를 쓴 `match`에는 마지막에 가드 없는 갈래가 필요합니다. 가능하면 범위 패턴처럼 컴파일러가 이해할 수 있는 패턴으로 조건을 적는 편이 좋습니다. `t if t < 0` 대신 `..0`이나 `i32::MIN..=-1`을 쓰면 완전성 검사가 그대로 작동합니다.

#### `if let`과 가드

가드는 `match` 갈래에만 붙일 수 있고, `if let`이나 `while let`에는 붙일 수 없습니다. 대신 Edition 2024의 let 체인으로 같은 일을 합니다.

```rust
if let Some(x) = num
    && x % 2 == 0
{
    println!("{x}: 짝수만 처리합니다");
}
```

반대로 가드 자리에 `if let`을 쓸 수도 있습니다. 가드 안의 패턴이 맞을 때만 그 갈래로 오고, 묶인 변수를 갈래 안에서 씁니다. The Book에는 나오지 않지만 이 저장소의 툴체인(rustc 1.98.1)에서 동작을 확인했습니다.

```rust
match input {
    Some(text) if let Ok(n) = text.parse::<i32>() => println!("{text} → {n}"),
    Some(text) => println!("{text} → 숫자가 아닙니다"),
    None => println!("입력 없음"),
}
```

전체 코드는 [`07_match_guards.rs`](examples/07_match_guards.rs)에 있습니다.

### 2.9 `@` 바인딩: 검사하면서 묶기

범위 패턴 `13..=18`은 값이 범위 안인지 검사하지만, 갈래 안에서는 실제 값이 무엇이었는지 알 수 없습니다. `이름 @ 패턴`은 값이 패턴과 맞는지 검사하면서 **동시에** 그 값을 이름에 묶습니다.

```rust
match age {
    0..=6 => String::from("무료"),                    // 값이 필요 없으면 범위만
    child @ 7..=12 => format!("어린이 요금 ({child}세)"),
    teen @ 13..=18 => format!("청소년 요금 ({teen}세)"),
    senior @ 65.. => format!("경로 요금 ({senior}세)"),
    adult => format!("일반 요금 ({adult}세)"),
}
```

가드로 `n if (13..=18).contains(&n)`처럼 쓸 수도 있지만, 그러면 완전성 검사에서 빠집니다. `@`는 패턴이므로 완전성 검사에 그대로 들어갑니다.

- **`|`와 함께 쓸 때는 괄호가 필요합니다.** `n @ 1 | 4 | 9`는 `(n @ 1) | 4 | 9`로 읽혀서, 4나 9와 맞았을 때 `n`이 묶이지 않으므로 E0408이 납니다(4.3절과 같은 에러). `square @ (1 | 4 | 9)`처럼 씁니다.
- **값 전체와 안쪽을 함께 묶을 수 있습니다.** `whole @ Point { x: 0, y }`는 `Point` 전체를 `whole`에 옮기고 필드 `y`(i32)를 복사합니다. 안쪽 바인딩이 `String`처럼 `Copy`가 아니면 같은 값을 두 번 가져가는 셈이라 E0382가 납니다. 이때는 참조를 맞춥니다(`match &opt { whole @ Some(inner) => ... }`).

### 2.10 배열과 슬라이스 패턴

배열과 슬라이스도 `[원소, 원소, ...]` 모양의 패턴으로 풉니다.

- **배열**은 길이가 타입에 들어 있으므로(`[i32; 3]`) 길이가 맞는 패턴은 반박 불가능합니다. `let [r, g, b] = [255, 128, 0];`처럼 `let`에 바로 씁니다.
- **슬라이스**(`&[T]`)는 길이가 실행할 때 정해지므로, 길이별로 갈래를 나눌 수 있습니다. `..`는 "몇 개든 건너뛰기"이고, `이름 @ ..`는 건너뛴 부분을 슬라이스로 묶습니다.

```rust
fn describe(items: &[i32]) -> String {
    match items {
        [] => String::from("비어 있음"),
        [one] => format!("하나뿐: {one}"),
        [first, second] => format!("둘: {first}, {second}"),
        [first, .., last] => format!("처음 {first}, 마지막 {last} (모두 {}개)", items.len()),
    }
}

fn sum(items: &[i32]) -> i32 {
    match items {
        [] => 0,
        [head, rest @ ..] => head + sum(rest), // rest: &[i32]
    }
}
```

`describe`의 네 갈래는 길이 0, 1, 2, 3 이상을 모두 덮으므로 `_`가 필요 없습니다. 슬라이스를 참조로 맞추므로 `one`, `first` 같은 바인딩은 `&i32`입니다(기본 바인딩 모드).

문자열을 공백으로 나눈 `Vec<&str>`을 `as_slice()`로 슬라이스로 만들면, 리터럴과 바인딩을 섞어 "명령 이름과 인자 개수"를 한 번에 검사할 수 있습니다.

```rust
match words.as_slice() {
    ["add", item] => format!("'{item}' 추가"),
    ["remove", index] => format!("{index}번 삭제"),
    ["list"] => String::from("목록 보기"),
    ["add" | "remove", ..] => String::from("인자 개수가 틀렸습니다"),
    [command, args @ ..] => format!("모르는 명령 {command}, 인자 {args:?}"),
    [] => String::from("빈 줄"),
}
```

전체 코드는 [`08_at_and_slices.rs`](examples/08_at_and_slices.rs)에 있습니다.

### 2.11 정리: 패턴 문법 한눈에 보기

| 문법 | 예 | 뜻 | 절 |
| --- | --- | --- | --- |
| 리터럴 | `1`, `'a'`, `"stop"` | 값이 같으면 맞음 | 2.4 |
| 이름 | `x` | 무엇이든 맞고 `x`에 묶음(바깥 `x`와 비교하지 않음) | 2.4 |
| 상수 | `LUCKY` | 상수의 값과 같으면 맞음 | 2.4 |
| 여러 패턴 | `1 \| 2`, `Some(6 \| 7)` | 하나라도 맞으면 맞음. 모든 쪽이 같은 이름을 묶어야 함 | 2.4 |
| 범위 | `1..=5`, `80..90`, `90..`, `'a'..='z'` | 범위 안이면 맞음(정수, `char`) | 2.4 |
| 구조체 | `Point { x, y: 0 }` | 필드별로 맞춤. 모든 필드를 적거나 `..` | 2.5 |
| 열거형 | `Some(n)`, `Message::Move { x, y }` | 배리언트가 같고 안쪽이 맞으면 맞음 | 2.5 |
| 튜플 | `(a, 0, _)` | 원소별로 맞춤. 개수가 같아야 함 | 2.5 |
| 참조 | `&n`, `&(x, y)` | 참조를 벗기고 안쪽과 맞춤 | 2.6 |
| `ref`, `ref mut` | `Some(ref s)` | 바인딩을 참조로 만듦 | 2.6 |
| 무시 | `_`, `Some(_)` | 무엇이든 맞고 묶지 않음 | 2.7 |
| 나머지 | `Point { x, .. }`, `(a, .., z)`, `[h, ..]` | 나머지를 무시. 한 패턴에 한 번 | 2.7 |
| 가드 | `Some(x) if x > 0` | 패턴이 맞고 조건이 참일 때(`match`만) | 2.8 |
| `@` 바인딩 | `n @ 1..=9`, `rest @ ..` | 검사하면서 그 값을 묶음 | 2.9 |
| 배열·슬라이스 | `[r, g, b]`, `[first, .., last]` | 원소 위치별로 맞춤 | 2.10 |

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_pattern_places.rs`](examples/01_pattern_places.rs) | 패턴이 쓰이는 일곱 자리와 let 체인 | 1 |
| [`02_refutability.rs`](examples/02_refutability.rs) | 반박 가능한·반박 불가능한 패턴, 자리별 요구, `if let`의 경고 | 2 |
| [`03_literals_ranges.rs`](examples/03_literals_ranges.rs) | 리터럴, 이름 있는 변수의 섀도잉 함정, 상수, `\|`, 범위 패턴 | 3 |
| [`04_destructuring.rs`](examples/04_destructuring.rs) | 구조체·열거형·중첩 열거형·튜플 구조 분해 | 3 |
| [`05_binding_modes.rs`](examples/05_binding_modes.rs) | 기본 바인딩 모드, `&` 패턴, `ref`/`ref mut` | 4 |
| [`06_ignoring.rs`](examples/06_ignoring.rs) | `_`, 중첩된 `_`, `_x`와 `_`의 차이(이동·drop 시점), `..` | 4 |
| [`07_match_guards.rs`](examples/07_match_guards.rs) | 매치 가드, `\|`와의 우선순위, 완전성 검사, let 체인, 가드 안의 `if let` | 5 |
| [`08_at_and_slices.rs`](examples/08_at_and_slices.rs) | `@` 바인딩, 배열·슬라이스 패턴, `rest @ ..` | 3, 5 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 6개 (4절) | 2~5 |

### 01_pattern_places — 패턴이 쓰이는 자리

`match` 갈래, `let`, `if let` 사슬, let 체인, `while let`, `for`, 함수·클로저 매개변수, `let...else`를 차례로 보여 줍니다. 3번 구역과 4번 구역은 같은 나이 판정을 중첩 `if`와 let 체인으로 각각 쓴 것이라 나란히 비교해 보세요. 4번 구역에서 `"모름"`은 사슬의 첫 고리(`parse`)에서 실패해 `else`로 갑니다.

```sh
cargo run -p p0-ch18-patterns --example 01_pattern_places
```

```
1. match 갈래: => 왼쪽이 패턴입니다
    Some(3) → 3칸 이동
    None → 아직 던지지 않음
2. let: 등호 왼쪽도 패턴입니다
    x = 5, a = 1, b = 2, c = 3
3. if let / else if / else if let / else 사슬
    쿠폰 WELCOME 할인
    회원의 날 10% 할인
    경로 우대 할인
    할인 없음
    나이를 확인할 수 없음
4. let 체인: let 패턴과 조건을 &&로 잇기 (Edition 2024)
    70: 경로 우대 할인
    30: 할인 없음
    모름: 할인 없음
5. while let: 패턴이 맞는 동안 반복합니다
    꺼낸 값: 3
    꺼낸 값: 2
    꺼낸 값: 1
    남은 스택: []
6. for: for 바로 뒤가 패턴입니다
    0번: 사과
    1번: 배
    2번: 감
7. 함수와 클로저의 매개변수
    현재 위치: (3, 5)
    넓이: [12, 30]
8. let...else
    민지님은 20살입니다
    형식이 잘못됐습니다: 이름만
```

### 02_refutability — 반박 가능성

`let`, 함수 매개변수에 반박 불가능한 패턴을, `if let`에 반박 가능한 패턴을 씁니다. 3번 구역은 같은 튜플 모양이라도 `(x, _)`는 반박 불가능하고 `(y, 0)`은 반박 가능하다는 것을 보여 줍니다. 5번 구역의 `always_matches`는 경고를 직접 확인하기 위해 반박 불가능한 패턴을 `if let`에 일부러 쓴 함수로, `#[expect(irrefutable_let_patterns)]`를 지우고 빌드하면 2.3절의 경고가 나옵니다.

```sh
cargo run -p p0-ch18-patterns --example 02_refutability
```

```
1. 반박 불가능한 패턴: 어떤 값이 와도 맞습니다
    w = 3, h = 4, m = 2.5
    2.5m = 250cm
2. 반박 가능한 패턴: 맞지 않는 값이 있습니다
    42 → 숫자 42
    사십이 → 숫자가 아님
3. 같은 모양이라도 리터럴이 들어가면 반박 가능해집니다
    x = 1, 둘째 값이 0인 쌍 (y = 1)
    x = 2, 둘째 값이 0이 아닌 쌍
4. match는 갈래들을 합쳐서 모든 값을 덮어야 합니다
    200: 성공
    404: 찾을 수 없음
    500: 기타 상태 500
5. 반박 불가능한 패턴을 if let에 쓰면 else는 절대 실행되지 않습니다
    언제나 이 블록이 실행됩니다: 7m
```

### 03_literals_ranges — 리터럴, 변수, 상수, `|`, 범위

2번 구역이 섀도잉 함정입니다. 바깥 `y`는 10이지만 `Some(y)` 갈래 안의 `y`는 5입니다. 6번과 7번 구역의 `digits`, `grade` 함수는 범위 패턴만으로 모든 값을 덮어 `_` 갈래가 없습니다. 갈래 하나를 지우고 빌드해서 컴파일러가 알려 주는 빠진 구간을 확인해 보세요.

```sh
cargo run -p p0-ch18-patterns --example 03_literals_ranges
```

```
1. 리터럴 패턴: 값이 같으면 맞습니다
    start: 시작합니다
    stop: 멈춥니다
    jump: 모르는 명령입니다
    1 2 Fizz 4 Buzz Fizz 7 8 Fizz Buzz 11 Fizz 13 14 FizzBuzz
2. 이름 있는 변수는 무엇이든 맞고, 같은 이름의 바깥 변수를 가립니다
    맞았습니다, y = 5
    match 뒤: x = Some(5), y = 10
3. 상수는 패턴 안에서 값으로 비교됩니다
    3: 보통 숫자
    7: 행운의 숫자!
4. | 로 여러 패턴 중 하나와 맞추기
    Some(1): 평일
    Some(6): 주말
    Some(7): 주말
    Some(9): 없는 요일
    None: 요일 정보 없음
5. ..= 범위 패턴 (정수와 char)
    '한': 한글 음절
    'Q': 영문자
    '7': 숫자
    '!': 기타
6. 범위로 모든 값을 덮으면 _ 없이도 완전합니다
    7: 1자리
    42: 2자리
    255: 3자리
7. 끝을 포함하지 않는 범위(a..b)와 끝이 열린 범위(a..)
    95점: A
    90점: A
    89점: B
    72점: C
    30점: F
```

### 04_destructuring — 구조 분해

구조체를 다른 이름으로·같은 이름으로 풀기, 구조체 패턴에 리터럴 섞기, 중첩된 열거형을 한 갈래에서 풀기, 튜플과 구조체를 섞은 값을 한 번에 풀기를 보여 줍니다. 1번 구역에서 `p`를 두 번 구조 분해해도 `p`가 남아 있는 것은 `i32` 필드가 복사되기 때문입니다.

```sh
cargo run -p p0-ch18-patterns --example 04_destructuring
```

```
1. 구조체 구조 분해: 필드 이름으로 꺼냅니다
    a = 3, b = -7 / x = 3, y = -7
    p는 그대로: (3, -7)
2. 구조체 패턴에 리터럴 섞기
    원점
    x축 위 (x = 5)
    y축 위 (y = -2)
    어느 축에도 없음 (1, 1)
3. 열거형과 중첩된 열거형 구조 분해
    이동: (10, 20)
    글자 쓰기: 안녕
    색 변경: RGB(255, 128, 0)
    색 변경: HSV(200, 50, 80)
    종료
4. 튜플과 구조체를 섞은 값도 한 번에 풉니다
    키 5피트 11인치, 위치 (3, -10)
```

### 05_binding_modes — 기본 바인딩 모드와 `ref`

`type_of` 함수가 `std::any::type_name`으로 바인딩의 실제 타입을 출력합니다. `type_name`은 `String`을 `alloc::string::String`처럼 전체 경로로 적기 때문에, 예제에서는 읽기 쉽도록 경로 부분을 지웠습니다. 1번 구역에서는 `String`과 `u32`, 2·3번 구역에서는 같은 패턴이 `&`, `&mut`를 맞추자 `&String`, `&mut String`으로 바뀌는 것을 확인하세요. 6번 구역 마지막 줄은 `ref`로 `name`만 빌리고 `age`는 복사한 결과입니다.

```sh
cargo run -p p0-ch18-patterns --example 05_binding_modes
```

```
1. 값을 패턴과 맞추면 바인딩이 값을 가져갑니다
    name: String = 민지
    age: u32 = 20
2. 참조를 참조가 아닌 패턴과 맞추면 바인딩이 참조가 됩니다
    name: &String, age: &u32
    user는 그대로: User { name: "준호", age: 31 }
3. 가변 참조를 맞추면 바인딩은 가변 참조가 됩니다
    name: &mut String, age: &mut u32
    고친 뒤: User { name: "준호(관리자)", age: 32 }
4. 안쪽 깊은 곳의 바인딩도, for와 if let에서도 같은 규칙입니다
    if let의 s: &String
    가은: 90점 (&String, &i32)
    나래: 85점 (&String, &i32)
5. & 패턴: 참조를 벗겨 안의 값을 꺼냅니다 (Copy 값만)
    n = 1 (i32)
    n = 2 (i32)
    n = 3 (i32)
    [2, 4, 6]
6. ref: 값을 맞추면서 바인딩만 참조로 만듭니다
    s: &String = 러스타
    nickname은 그대로: Some("러스타")
    level: Some(2)
    name: &String, age: u32
    user는 그대로: User { name: "준호(관리자)", age: 32 }
```

### 06_ignoring — 값 무시하기

5번 구역이 `_`와 `_x`의 drop 시점 차이입니다. ch14에서 만든 것과 같은 방식으로 `Drop`을 구현한 `Noisy`가 drop될 때 메시지를 출력합니다. `_`에 넘긴 값은 "블록의 마지막 줄"보다 **먼저** drop되고, `_guard`에 묶은 값은 블록이 끝날 때 drop됩니다.

```sh
cargo run -p p0-ch18-patterns --example 06_ignoring
```

```
1. _: 값 전체를 무시합니다
    '확인' 버튼이 눌렸습니다
2. 중첩된 _: 값의 일부만 무시합니다
    이미 설정된 값을 덮어쓸 수 없습니다
    setting = Some(5)
    2, 8, 32
3. _로 시작하는 이름: 묶기는 하지만 '쓰지 않는 변수' 경고를 끕니다
    경고 없이 컴파일됐습니다
4. _와 _x의 차이 (1): _는 값을 옮기지 않습니다
    count = 3, pair는 그대로: ("안녕", 3)
5. _와 _x의 차이 (2): drop 시점
    [drop] _에 넘긴 값
    블록의 마지막 줄
    [drop] _guard에 묶은 값
6. ..: 나머지를 한꺼번에 무시합니다
    x = 1
    x가 0인 점
    x가 0이 아닌 점 (y = 2, z = 3)
    처음 2, 마지막 32
```

### 07_match_guards — 매치 가드

1번 구역의 `Some(5)`는 첫 갈래의 패턴과 맞지만 가드가 거짓이라 둘째 갈래로 갑니다. 3번 구역은 `|`와 가드의 우선순위 때문에 `no`가 출력됩니다. 4번 구역의 `sign` 함수에서 마지막 `_` 갈래를 `x if x > 0`으로 바꾸면 E0004가 납니다(4.2절).

```sh
cargo run -p p0-ch18-patterns --example 07_match_guards
```

```
1. 가드: 패턴이 맞아도 조건이 거짓이면 다음 갈래로 넘어갑니다
    4: 짝수
    5: 홀수
    값 없음
2. 가드로 바깥 변수와 비교하기 (03의 섀도잉 함정 해결)
    5: 목표값이 아닙니다
    10: 목표값과 같습니다
3. |와 가드: 가드는 | 로 묶인 패턴 전체에 걸립니다
    no
4. 가드가 있으면 마지막에 가드 없는 갈래가 필요합니다
    -3: 음수
    0: 0
    8: 양수
5. if let에는 가드가 없지만, let 체인으로 같은 일을 합니다 (Edition 2024)
    4: 짝수만 처리합니다
    Some(5): 건너뜁니다
6. 가드 자리에 if let 쓰기
    42 → 42
    사십이 → 숫자가 아닙니다
    입력 없음
```

### 08_at_and_slices — `@` 바인딩과 슬라이스 패턴

1번 구역에서 5세는 `@` 없는 범위 갈래(`0..=6`)로 가므로 결과에 나이가 들어가지 않고, 나머지는 `@`로 묶은 나이가 결과에 들어갑니다. 5번 구역은 같은 배열을 길이가 다른 슬라이스(`&data[..0]` 등)로 잘라 `describe`의 갈래가 길이별로 갈리는 것을 보여 줍니다. 7번 구역에서 `"add"`는 `["add", item]`(원소 2개)과 맞지 않아 `["add" | "remove", ..]`로 갑니다.

```sh
cargo run -p p0-ch18-patterns --example 08_at_and_slices
```

```
1. @ 바인딩: 검사하면서 묶기
    5세: 무료
    10세: 어린이 요금 (10세)
    16세: 청소년 요금 (16세)
    40세: 일반 요금 (40세)
    70세: 경로 요금 (70세)
2. @와 |: 괄호로 묶어야 | 전체가 한 이름에 묶입니다
    1: 한 자리 제곱수
    4: 한 자리 제곱수
    5: 제곱수가 아님
    9: 한 자리 제곱수
3. @로 값 전체를 묶으면서 안쪽 패턴도 검사하기
    Point { x: 0, y: 7 }: y축 위, 높이 7
    (2, 3): y축 밖
4. 배열 패턴: 길이가 정해져 있으므로 반박 불가능합니다
    r = 255, g = 128, b = 0
5. 슬라이스 패턴: 길이별로 나누기
    [] → 비어 있음
    [10] → 하나뿐: 10
    [10, 20] → 둘: 10, 20
    [10, 20, 30, 40] → 처음 10, 마지막 40 (모두 4개)
6. rest @ ..: 나머지를 슬라이스로 묶기
    sum([10, 20, 30, 40]) = 100
7. 문자열 슬라이스 패턴으로 명령어 해석하기
    "add 우유" → '우유' 추가
    "remove 3" → 3번 삭제
    "list" → 목록 보기
    "add" → 인자 개수가 틀렸습니다
    "move a b c" → 모르는 명령 move, 인자 ["a", "b", "c"]
    "" → 빈 줄
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch18-patterns
rustc --edition 2024 compile_fail/e0005_refutable_for_pattern.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 코드의 자세한 설명은 `rustc --explain E0005`처럼 확인할 수 있습니다.

### 4.1 `for`에 반박 가능한 패턴 사용 — E0005

파일: [`compile_fail/e0005_refutable_for_pattern.rs`](compile_fail/e0005_refutable_for_pattern.rs)

측정에 실패한 칸이 `None`인 센서 값 벡터를 `for Some(r) in readings`로 순회하려 한 코드입니다.

```
error[E0005]: refutable pattern in `for` loop binding
  --> compile_fail/e0005_refutable_for_pattern.rs:14:9
   |
14 |     for Some(r) in readings {
   |         ^^^^^^^ pattern `None` not covered
   |
   = note: the matched value is of type `Option<i32>`
```

- **읽는 법**: ch6의 `let`에서 본 E0005와 같은 에러인데, 첫 줄이 `` in `for` loop binding ``으로 자리를 알려 줍니다. 함수 매개변수라면 `in function argument`, 클로저라면 `in closure argument`입니다. `` pattern `None` not covered ``는 이 패턴이 `None`을 처리하지 못한다는 뜻입니다.
- **원인**: `for`의 패턴은 반박 불가능해야 합니다. 원소가 `None`일 때 `r`에 넣을 값이 없는데, `for`에는 그 경우에 할 일을 적을 자리가 없습니다.
- **해결**: 원소를 변수 하나로 받고, 반복문 안에서 `if let`이나 `match`로 나눕니다.

  ```rust
  for reading in readings {
      if let Some(r) = reading {
          println!("측정값: {r}");
      }
  }
  ```

### 4.2 가드만으로 모든 경우를 덮으려 함 — E0004

파일: [`compile_fail/e0004_guard_not_exhaustive.rs`](compile_fail/e0004_guard_not_exhaustive.rs)

```
error[E0004]: non-exhaustive patterns: `i32::MIN..=i32::MAX` not covered
  --> compile_fail/e0004_guard_not_exhaustive.rs:11:11
   |
11 |     match temp {
   |           ^^^^ pattern `i32::MIN..=i32::MAX` not covered
   |
   = note: the matched value is of type `i32`
   = note: match arms with guards don't count towards exhaustivity
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
13 ~         t if t >= 0 => "영상",
14 ~         i32::MIN..=i32::MAX => todo!(),
   |
```

- **읽는 법**: 덮지 못한 값이 `i32::MIN..=i32::MAX`, 즉 **모든 정수**라고 나옵니다. 두 번째 `note:`가 이유입니다. "가드가 붙은 갈래는 완전성 계산에 넣지 않는다." 그래서 이 `match`는 컴파일러가 보기에 아무 값도 확실히 덮지 못합니다.
- **원인**: 사람이 보기에는 `t < 0`과 `t >= 0`이 모든 정수를 덮지만, 컴파일러는 가드 안의 조건을 따지지 않습니다.
- **해결**: 마지막 갈래의 가드를 지우고 `_ => "영상"`처럼 가드 없는 패턴으로 나머지를 받습니다. 조건을 범위 패턴으로 바꾸는 방법도 있습니다. `..0 => "영하", 0.. => "영상"`이라고 쓰면 가드가 없으므로 컴파일러가 완전성을 확인할 수 있습니다.

### 4.3 `|`의 양쪽이 다른 변수를 묶음 — E0408

파일: [`compile_fail/e0408_or_pattern_binding.rs`](compile_fail/e0408_or_pattern_binding.rs)

원의 반지름과 정사각형의 한 변을 "크기"로 묶어 한 갈래에서 처리하려고, 필드 이름 축약을 그대로 쓴 코드입니다.

```
error[E0408]: variable `side` is not bound in all patterns
  --> compile_fail/e0408_or_pattern_binding.rs:21:9
   |
21 |         Shape::Circle { radius } | Shape::Square { side } => radius,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^                   ---- variable not in all patterns
   |         |
   |         pattern doesn't bind `side`

error[E0408]: variable `radius` is not bound in all patterns
...
error[E0381]: used binding `radius` is possibly-uninitialized
...
```

- **읽는 법**: `side`는 오른쪽 패턴에만, `radius`는 왼쪽 패턴에만 있다고 양쪽 모두에 대해 알려 줍니다. 뒤따르는 E0381(`possibly-uninitialized`)은 "오른쪽이 맞으면 `radius`에 값이 없다"는 같은 문제를 다른 각도에서 본 것이라, E0408을 고치면 함께 사라집니다.
- **원인**: `|`로 묶은 패턴은 어느 쪽이 맞든 갈래 안의 코드가 똑같이 실행됩니다. 그러려면 어느 쪽이 맞아도 같은 변수가 같은 타입으로 준비돼 있어야 합니다.
- **해결**: 2.5절의 `필드: 변수` 문법으로 양쪽 이름을 맞춥니다.

  ```rust
  Shape::Circle { radius: size } | Shape::Square { side: size } => size,
  ```

  `@`와 `|`를 섞은 `n @ 1 | 4 | 9`도 같은 E0408이 납니다. `(n @ 1) | 4 | 9`로 읽히기 때문이며, `n @ (1 | 4 | 9)`로 고칩니다(2.9절). 한 패턴 안에서 같은 이름을 두 번 묶는 `let (a, a) = (1, 2);`는 반대 방향의 실수로, E0416(`` identifier `a` is bound more than once in the same pattern ``)이 납니다.

### 4.4 구조체 패턴에서 필드를 빠뜨림 — E0027

파일: [`compile_fail/e0027_missing_struct_field.rs`](compile_fail/e0027_missing_struct_field.rs)

`Config`에 `verbose` 필드를 새로 추가했는데, 구조 분해하는 곳을 고치지 않은 상황입니다.

```
error[E0027]: pattern does not mention field `verbose`
  --> compile_fail/e0027_missing_struct_field.rs:23:9
   |
23 |     let Config { host, port } = config;
   |         ^^^^^^^^^^^^^^^^^^^^^ missing field `verbose`
   |
help: include the missing field in the pattern
   |
23 |     let Config { host, port, verbose } = config;
   |                            +++++++++
help: if you don't care about this missing field, you can explicitly ignore it
   |
23 |     let Config { host, port, verbose: _ } = config;
   |                            ++++++++++++
help: or always ignore missing fields here
   |
23 |     let Config { host, port, .. } = config;
   |                            ++++
```

- **읽는 법**: `help:`가 세 가지 해결책을 순서대로 제안합니다. 필드를 쓰거나(`verbose`), 이 필드만 무시하거나(`verbose: _`), 빠진 필드를 모두 무시합니다(`..`).
- **원인**: 구조체 패턴은 모든 필드를 적어야 합니다. 필드를 빠뜨린 것이 실수인지 의도인지 컴파일러가 알 수 없으므로 조용히 넘어가지 않습니다. 이 덕분에 구조체에 필드를 추가하면 그 구조체를 푸는 곳을 컴파일러가 모두 찾아 줍니다.
- **해결**: 새 필드를 이 자리에서도 다뤄야 하면 패턴에 적습니다. 정말 필요 없다면 `verbose: _`로 그 필드만 무시하는 편이, 앞으로 추가될 필드까지 모두 무시하는 `..`보다 다음 변경에 안전합니다.

### 4.5 `_x`로 값을 묶은 뒤 원본 사용 — E0382

파일: [`compile_fail/e0382_underscore_name_moves.rs`](compile_fail/e0382_underscore_name_moves.rs)

```
error[E0382]: borrow of partially moved value: `pair`
  --> compile_fail/e0382_underscore_name_moves.rs:16:16
   |
13 |     let (_greeting, count) = pair;
   |          --------- value partially moved here
...
16 |     println!("{pair:?}");
   |                ^^^^ value borrowed here after partial move
   |
   = note: partial move occurs because `pair.0` has type `String`, which does not implement the `Copy` trait
help: borrow this binding in the pattern to avoid moving the value
   |
13 |     let (ref _greeting, count) = pair;
   |          +++
```

- **읽는 법**: `value partially moved here`가 `_greeting`을 가리킵니다. `_`로 시작하는 이름도 값을 묶는 보통 변수라서 `pair.0`(`String`)이 그리로 이동했습니다.
- **원인**: `_greeting`의 밑줄은 "쓰지 않는 변수" 경고만 끌 뿐, `_`처럼 "묶지 않음"을 뜻하지 않습니다(2.7절).
- **해결**: 값이 필요 없다면 `let (_, count) = pair;`처럼 `_`를 씁니다. 값을 빌려서 쓰고 싶다면 `help:`대로 `ref _greeting`을 쓰거나 `&pair`를 맞춥니다.

### 4.6 한 패턴에 `..`를 두 번 사용 — 에러 코드 없음

파일: [`compile_fail/double_rest_pattern.rs`](compile_fail/double_rest_pattern.rs)

```
error: `..` can only be used once per tuple pattern
  --> compile_fail/double_rest_pattern.rs:12:22
   |
12 |     let (.., second, ..) = numbers;
   |          --          ^^ can only be used once per tuple pattern
   |          |
   |          previously used here
```

- **읽는 법**: 에러 코드가 없는 문법 에러입니다. 두 번째 `..`를 가리키며 첫 번째 `..`의 위치(`previously used here`)를 함께 보여 줍니다.
- **원인**: `..`는 "몇 개인지 모르는 나머지"입니다. 둘이 있으면 앞의 `..`가 몇 개를 건너뛰는지, 즉 `second`가 몇 번째 원소인지 정할 수 없습니다.
- **해결**: `..`는 한 번만 쓰고 위치를 정할 원소는 `_`로 하나씩 적습니다. 두 번째 값이 필요하면 `let (_, second, ..) = numbers;`입니다.

## 5. 연습 문제

### 연습 1. `_` 없이 사분면 판별하기

점 `(x, y)`가 원점, x축 위, y축 위, 1~4사분면 중 어디에 있는지 돌려주는 `fn quadrant(point: (i32, i32)) -> &'static str`을 작성하세요. 조건은 두 가지입니다. 매치 가드를 쓰지 않고, `_` 하나만으로 된 갈래(포괄 패턴)도 쓰지 않습니다. 그래도 컴파일러가 완전하다고 인정해야 합니다.

```
(0, 0) → 원점
(0, 5) → y축 위
(-3, 0) → x축 위
(2, 3) → 1사분면
(-2, 3) → 2사분면
(-2, -3) → 3사분면
(2, -3) → 4사분면
```

<details>
<summary>힌트</summary>

튜플 패턴 안에 리터럴과 범위를 넣습니다. `(0, 0)`, `(0, _)`, `(_, 0)`을 먼저 두고, 사분면은 `(1.., 1..)`, `(..=-1, 1..)`, `(..=-1, ..=-1)`, `(1.., ..=-1)`로 씁니다. `..=-1`은 "-1 이하"입니다. 완성한 뒤 사분면 갈래 하나를 지워 보면, 컴파일러가 E0004로 `` `(1_i32..=i32::MAX, i32::MIN..=-1_i32)` not covered ``처럼 빠진 영역을 정확히 계산해 알려 줍니다. 같은 내용을 가드(`(x, y) if x > 0 && y > 0`)로 쓰면 이 도움을 받을 수 없습니다.

</details>

### 연습 2. 이동 없이 매칭하기

다음 코드는 E0382로 컴파일되지 않습니다. 마지막 `println!`을 그대로 둔 채 두 가지 방법으로 고치세요. (가) `match` 대상에 `&`를 붙이는 방법, (나) 패턴에 `ref`를 쓰는 방법입니다. 두 방법에서 가드의 `pages > 300`이 어떻게 달라져야 하는지도 설명해 보세요.

```rust
#[derive(Debug)]
struct Book {
    title: String,
    pages: u32,
}

fn main() {
    let book = Book { title: String::from("러스트 입문"), pages: 320 };
    match book {
        Book { title, pages } if pages > 300 => println!("두꺼운 책: {title} ({pages}쪽)"),
        Book { title, .. } => println!("얇은 책: {title}"),
    }
    println!("{book:?}");
}
```

<details>
<summary>힌트</summary>

(가) `match &book`으로 바꾸면 기본 바인딩 모드에 따라 `title`은 `&String`, `pages`는 `&u32`가 됩니다. 그러면 `pages > 300`이 `&u32`와 정수의 비교가 되어 E0308(`` expected `&u32`, found integer ``)이 나므로 `*pages > 300`으로 고쳐야 합니다. (나) `Book { ref title, pages }`, `Book { ref title, .. }`로 쓰면 `title`만 빌리고 `pages`는 `u32`로 복사되므로 가드를 고칠 필요가 없습니다. 컴파일러의 `help:`가 제안하는 것이 (나)입니다. 두 방법 모두 `두꺼운 책: 러스트 입문 (320쪽)`과 `Book { title: "러스트 입문", pages: 320 }`을 출력합니다.

</details>

### 연습 3. 명령어 해석기 확장하기

[`08_at_and_slices.rs`](examples/08_at_and_slices.rs)의 7번 구역을 바탕으로 `fn run(line: &str) -> String`을 만들고 다음 규칙을 지키세요.

- `add`는 품목을 하나 이상 받습니다. 품목이 없으면 잘못된 명령입니다.
- `remove`는 인자가 숫자일 때만 삭제하고, 숫자가 아니면 그렇다고 알려 줍니다.
- `rename 옛이름 새이름`을 추가합니다.

```
"add 우유 빵" → 2개 추가: 우유, 빵
"add" → 잘못된 명령: add
"remove 3" → 3번 삭제
"remove 셋" → 번호가 숫자가 아닙니다: 셋
"rename 우유 두유" → 우유 → 두유
"list" → 목록 보기
"rename 우유" → 잘못된 명령: rename
"" → 빈 줄
```

<details>
<summary>힌트</summary>

`["add", items @ ..] if !items.is_empty()`처럼 `@ ..`와 가드를 함께 쓰면 품목이 하나 이상일 때만 맞습니다. `items`는 `&[&str]`이므로 `items.len()`, `items.join(", ")`를 쓸 수 있습니다. 가드 없이 `["add", first, rest @ ..]`로 "최소 하나"를 패턴에 담을 수도 있습니다. `remove`는 2.8절의 가드 안 `if let`으로 `["remove", index] if let Ok(n) = index.parse::<usize>() => ...`를 쓰고, 그 아래에 가드 없는 `["remove", index] => ...` 갈래를 둡니다. 맞지 않은 나머지는 `[command, ..] => format!("잘못된 명령: {command}")`과 `[] => ...`로 받습니다.

</details>

### 연습 4. 괄호 짝 검사

문자열의 괄호 `()`, `[]`, `{}`가 올바르게 짝지어져 있는지 검사하는 `fn is_balanced(text: &str) -> bool`을 작성하세요. 괄호가 아닌 문자는 무시합니다.

```
"(a[b]{c})": true
"(]": false
"((": false
"{[()]}": true
"x)": false
```

<details>
<summary>힌트</summary>

`Vec<char>`를 스택으로 씁니다. `for c in text.chars()` 안에서 `match c`로 나눕니다. 여는 괄호는 `'(' | '[' | '{' => stack.push(c)`로 쌓고, 닫는 괄호 `')' | ']' | '}'`가 나오면 `match (stack.pop(), c)`로 "꺼낸 여는 괄호와 지금 닫는 괄호"의 쌍을 한 번에 검사합니다. 맞는 쌍은 `(Some('('), ')') | (Some('['), ']') | (Some('{'), '}') => {}` 한 갈래로 묶고, 나머지(`_`)는 `return false`입니다. 스택이 비어 있으면 `pop`이 `None`을 돌려주므로 `"x)"`도 자연스럽게 `_`로 갑니다. 반복이 끝난 뒤 `stack.is_empty()`를 돌려주면 `"(("`처럼 닫히지 않은 경우도 잡힙니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [19. Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html)
  - [19.1 All the Places Patterns Can Be Used](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html): 패턴이 쓰이는 자리. "`if let Ok(age) = age && age > 30`으로 합칠 수 없다"는 설명은 Edition 2024의 let 체인 이전 기준입니다(2.2절).
  - [19.2 Refutability: Whether a Pattern Might Fail to Match](https://doc.rust-lang.org/book/ch19-02-refutability.html): 반박 가능성
  - [19.3 Pattern Syntax](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html): 패턴 문법 전체. 슬라이스 패턴과 `ref`는 이 절에 없고, 이 챕터의 2.6절과 2.10절에서 보충했습니다.
- Rust By Example: [Flow of Control > match](https://doc.rust-lang.org/rust-by-example/flow_control/match.html)와 그 하위 절
  - [Destructuring](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring.html): [tuples](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_tuple.html), [arrays/slices](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_slice.html), [enums](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_enum.html), [pointers/ref](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_pointers.html), [structs](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_structures.html). pointers/ref 절의 `let ref _is_a_reference = 3;`은 Clippy가 `clippy::toplevel_ref_arg`로 경고하는 옛 스타일이며, `let _is_a_reference = &3;`과 같습니다.
  - [Guards](https://doc.rust-lang.org/rust-by-example/flow_control/match/guard.html): 매치 가드와 완전성 검사. 4.2절과 같은 상황을 다룹니다.
  - [Binding](https://doc.rust-lang.org/rust-by-example/flow_control/match/binding.html): `@` 바인딩, `n @ (1 | 7 | 15 | 13)`, `Some(n @ 42)`. 첫 예제의 `n @ (1 | 7 | 15 | 13)` 갈래는 앞의 두 범위 갈래가 이미 모든 값을 덮기 때문에, 직접 컴파일하면 `unreachable pattern` 경고가 나옵니다.
  - [while let](https://doc.rust-lang.org/rust-by-example/flow_control/while_let.html): `loop` + `match`를 `while let`으로 줄이기
  - [if let](https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html), [let-else](https://doc.rust-lang.org/rust-by-example/flow_control/let_else.html)는 ch6에서 안내한 절입니다.
- Rustlings: 패턴만 다루는 주제는 없습니다. 이 챕터의 내용으로 풀 수 있는 문제는 다음과 같습니다(설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다).
  - [`12_options`](https://github.com/rust-lang/rustlings/tree/main/exercises/12_options): `options2.rs`는 `if let`과, `Vec::pop`이 감싼 `Option`을 한 번 더 푸는 중첩 패턴의 `while let`(`Some(Some(..))`)을 쓰는 문제입니다. `options3.rs`는 `match`에서 값이 이동하는 문제로, `&`를 붙이거나 `ref`를 쓰면 풀립니다(2.6절).
  - [`08_enums`](https://github.com/rust-lang/rustlings/tree/main/exercises/08_enums)의 `enums3.rs`: 구조체형·튜플형 배리언트를 구조 분해해 메서드에 넘기는 `match`를 작성합니다.
- Edition Guide
  - [let chains in if and while](https://doc.rust-lang.org/edition-guide/rust-2024/let-chains.html): let 체인이 Edition 2024에만 있는 이유
  - [Match ergonomics reservations](https://doc.rust-lang.org/edition-guide/rust-2024/match-ergonomics.html): Edition 2024에서 암묵적으로 빌리는 패턴 안의 `ref`, `mut`, `&`를 금지한 이유
- The Reference: [Patterns](https://doc.rust-lang.org/reference/patterns.html): 패턴 문법의 정식 정의와 [바인딩 모드](https://doc.rust-lang.org/reference/patterns.html#binding-modes). Part 9 `ch6-patterns`의 기준 문서입니다.
- 표준 라이브러리: [`std::any::type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html): 05 예제에서 타입 이름을 출력하는 데 쓴 함수. 출력 형식은 보장되지 않으므로 디버깅과 학습용으로만 씁니다.
- 에러 설명: `rustc --explain E0408`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
