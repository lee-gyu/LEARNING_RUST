# ch3. 일반적인 프로그래밍 개념

변수, 타입, 함수, 제어 흐름은 거의 모든 프로그래밍 언어에 있는 개념입니다. 이 챕터에서는 이 개념들이 Rust에서 어떻게 동작하는지 정리합니다. ch2 숫자 맞히기 게임에서 `let`, `mut`, `loop`를 맛보았다면, 여기서는 그 규칙을 처음부터 다시 짚고 규칙이 왜 그렇게 정해졌는지 살펴봅니다.

- 패키지 이름: `p0-ch3-common-concepts`
- 기준 문서: The Book 3장 (3.1 ~ 3.5)
- 예제 실행: `cargo run -p p0-ch3-common-concepts --example <예제 이름>` (저장소 안 어느 디렉터리에서 실행해도 됩니다)

## 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `let`, `let mut`, `const`, 섀도잉의 차이를 설명하고, 상황에 맞는 것을 골라 쓸 수 있다.
2. 스칼라 타입 네 가지(정수, 부동소수점 수, 불리언, 문자)와 복합 타입 두 가지(튜플, 배열)로 값을 선언하고 사용할 수 있다.
3. 정수 오버플로와 배열 범위 밖 접근이 컴파일할 때와 실행할 때 각각 어떻게 처리되는지 설명하고, 패닉하지 않는 대안 메서드(`checked_*`, `wrapping_*`, `get` 등)를 골라 쓸 수 있다.
4. 매개변수와 반환 타입이 있는 함수를 작성하고, 구문과 표현식의 차이와 세미콜론의 의미를 설명할 수 있다.
5. `if`, `loop`, `while`, `for`로 제어 흐름을 작성하고, `break`로 값을 돌려주거나 루프 레이블로 바깥 루프를 제어할 수 있다.

## 핵심 개념

### 1. 변수와 가변성

`let`은 값에 이름을 묶는 바인딩(binding)을 만듭니다. 이렇게 만든 변수는 기본적으로 **불변(immutable)** 입니다.

```rust
let x = 5;
x = 6; // 컴파일 에러 E0384: cannot assign twice to immutable variable `x`
```

불변이 기본인 이유는 버그를 막기 위해서입니다. 코드의 한 부분은 "이 값은 그대로다"라고 가정하는데 다른 부분이 그 값을 바꾸면, 문제가 생긴 곳과 원인이 있는 곳이 서로 멀리 떨어져 있어 찾기 어렵습니다. Rust는 "바뀌지 않는다"는 약속을 컴파일러가 확인하게 해서 이런 버그를 컴파일 단계에서 막습니다.

값이 바뀌어야 한다면 `mut`를 붙여 **가변(mutable)** 변수로 만듭니다. `mut`는 코드를 읽는 사람에게 "이 값은 뒤에서 바뀐다"고 알리는 표시이기도 합니다.

```rust
let mut count = 0;
count = 1;
count += 1; // count = count + 1 과 같습니다.
```

#### 상수

**상수(constant)** 는 `const`로 선언합니다. 불변 변수와 비슷해 보이지만 다음이 다릅니다.

| | 불변 변수 `let` | 상수 `const` |
| --- | --- | --- |
| `mut` | 붙이면 가변이 됨 | 붙일 수 없음 (`const mut`는 에러) |
| 타입 표기 | 대부분 생략 (컴파일러가 추론) | 반드시 적어야 함 |
| 선언 위치 | 함수 안 | 전역을 포함한 모든 스코프 |
| 값이 정해지는 때 | 실행 중 | 컴파일할 때 (상수 표현식만 가능) |
| 이름 관례 | `snake_case` | `UPPER_SNAKE_CASE` |

```rust
const SECONDS_PER_MINUTE: u32 = 60;
const THREE_HOURS_IN_SECONDS: u32 = 60 * SECONDS_PER_MINUTE * 3; // 컴파일할 때 10800으로 계산됨
```

상수의 값은 컴파일할 때 계산되므로 `10800`처럼 결과만 적는 대신 계산식을 적어 의미를 드러낼 수 있습니다. 반대로 실행해 봐야 알 수 있는 값은 상수에 넣을 수 없습니다. 실행 중에 바뀌는 변수로 상수를 만들려고 하면 다음 에러가 납니다.

```text
error[E0435]: attempt to use a non-constant value in a constant
```

여러 곳에서 쓰는 고정값(최대 점수, 단위 변환 계수 등)에 이름을 붙여 한곳에서 관리하고 싶을 때 상수를 씁니다. → [`examples/01_variables.rs`](examples/01_variables.rs)

### 2. 섀도잉

`let`을 다시 써서 같은 이름의 변수를 선언하면 **섀도잉(shadowing)** 이 일어납니다. 새 변수가 이전 변수를 가리므로, 그 이름으로는 새 변수만 보입니다. 안쪽 스코프(scope), 즉 중괄호 블록 안에서 섀도잉했다면 블록이 끝날 때 새 변수도 사라지고 이전 변수가 다시 보입니다.

```rust
let x = 5;
let x = x + 1;       // 새 변수 x = 6
{
    let x = x * 2;   // 블록 안에서만 x = 12
}
// 여기서 x는 다시 6
```

섀도잉은 "값을 바꾸는 것"이 아니라 "새 변수를 만드는 것"이라는 점에서 `mut`와 다릅니다.

| | 섀도잉 (`let x = ...`를 다시 씀) | `mut` 변수에 재대입 (`x = ...`) |
| --- | --- | --- |
| 일어나는 일 | 같은 이름의 새 변수를 만듦 | 기존 변수의 값을 바꿈 |
| 타입 변경 | 가능 | 불가능 (E0308) |
| 그 뒤의 가변성 | 새 변수를 `let`으로 만들면 불변 | 계속 가변 |
| 안쪽 블록에서 했을 때 | 블록이 끝나면 효과가 사라짐 | 바깥 변수에 그대로 남음 |

타입을 바꿀 수 있다는 점 덕분에, ch2에서 본 것처럼 문자열로 입력받은 값을 숫자로 바꾼 뒤 같은 이름으로 계속 쓸 수 있습니다.

```rust
let spaces = "   ";        // &str (문자열)
let spaces = spaces.len(); // usize (숫자) — 섀도잉이라 가능
```

→ [`examples/02_shadowing.rs`](examples/02_shadowing.rs)

### 3. 데이터 타입

Rust는 **정적 타입(static typing)** 언어입니다. 모든 변수의 타입이 컴파일할 때 정해져 있어야 합니다. 보통은 컴파일러가 값과 쓰임새를 보고 타입을 **추론(inference)** 하지만, 여러 타입이 가능하면 직접 **타입 표기(type annotation)** 를 해야 합니다. 예를 들어 `parse`는 여러 숫자 타입으로 변환할 수 있어서 어느 타입인지 알려 줘야 합니다.

```rust
let guess: u32 = "42".parse().expect("숫자가 아닙니다");
// `: u32`를 빼면 error[E0284]: type annotations needed
```

#### 스칼라 타입

**스칼라 타입(scalar type)** 은 값 하나를 나타내는 타입이며 네 가지가 있습니다.

**정수.** `i`로 시작하면 부호 있는 정수, `u`로 시작하면 부호 없는 정수이고, 뒤의 숫자는 비트 수입니다.

| 크기 | 부호 있음 | 부호 없음 |
| --- | --- | --- |
| 8비트 | `i8` (-128 ~ 127) | `u8` (0 ~ 255) |
| 16비트 | `i16` | `u16` |
| 32비트 | `i32` | `u32` |
| 64비트 | `i64` | `u64` |
| 128비트 | `i128` | `u128` |
| 아키텍처에 따라 다름 | `isize` | `usize` |

`n`비트 부호 있는 정수는 -2<sup>n-1</sup> ~ 2<sup>n-1</sup>-1, 부호 없는 정수는 0 ~ 2<sup>n</sup>-1을 담습니다. 타입을 적지 않으면 `i32`가 기본이고, `usize`는 배열 인덱스처럼 "몇 번째"나 "몇 개"를 셀 때 씁니다. 64비트 컴퓨터에서 `isize`/`usize`는 64비트입니다.

정수 리터럴은 `98_222`(밑줄로 자릿수 구분), `0xff`(16진수), `0o77`(8진수), `0b1111_0000`(2진수), `b'A'`(바이트, `u8` 전용)처럼 쓸 수 있고, `57u8`처럼 타입을 뒤에 붙일 수도 있습니다.

**부동소수점 수.** `f32`와 `f64`가 있고 기본은 `f64`입니다. 현대 CPU에서 둘의 속도가 거의 같고 `f64`가 더 정밀하기 때문입니다. IEEE-754 표준을 따르므로 `0.1 + 0.2`가 `0.30000000000000004`가 되는 것처럼 십진수를 정확히 나타내지 못하는 경우가 있습니다.

**수치 연산.** `+`, `-`, `*`, `/`, `%`를 씁니다. 정수끼리 나누면 결과도 정수이고 소수점 아래는 0 쪽으로 버려집니다(`7 / 2`는 `3`, `-5 / 3`은 `-1`). Rust는 정수와 실수를 자동으로 섞어 계산하지 않습니다. `1 + 2.5`는 `error[E0277]: cannot add a float to an integer`입니다. 타입을 바꾸려면 `as`로 직접 변환합니다. 숨은 변환이 없으므로 정밀도가 바뀌는 지점이 코드에 드러납니다.

```rust
let average = total_score as f64 / count as f64;
```

**불리언.** `bool` 타입이며 값은 `true`와 `false`뿐입니다. `10 > 5` 같은 비교 연산의 결과가 `bool`입니다.

**문자.** `char` 타입이며 작은따옴표로 씁니다(큰따옴표는 문자열입니다). `char`는 **유니코드 스칼라 값** 하나를 4바이트로 담으므로, 영문자뿐 아니라 `'한'`, `'ℤ'`, `'😻'`도 `char` 하나입니다.

→ [`examples/03_scalar_types.rs`](examples/03_scalar_types.rs)

#### 복합 타입

**복합 타입(compound type)** 은 여러 값을 하나로 묶은 타입입니다. 두 가지 모두 **길이가 고정**되어 있어 만든 뒤에 늘리거나 줄일 수 없습니다.

**튜플(tuple)** 은 서로 다른 타입의 값을 묶습니다. 구조 분해(destructuring) 패턴으로 풀거나 `.0`, `.1`처럼 번호로 꺼냅니다.

```rust
let tup: (i32, f64, char) = (500, 6.4, '가');
let (x, y, z) = tup;  // 구조 분해
let first = tup.0;    // 번호로 접근
```

원소가 없는 튜플 `()`는 **유닛(unit)** 이라고 부르며 "의미 있는 값이 없음"을 나타냅니다. 반환 타입을 적지 않은 함수는 `()`를 돌려줍니다.

**배열(array)** 은 같은 타입의 값을 묶습니다. 타입은 `[원소 타입; 길이]`로 쓰며, 길이도 타입의 일부입니다. `[값; 개수]`는 같은 값으로 채운 배열을 만듭니다.

```rust
let a: [i32; 5] = [1, 2, 3, 4, 5];
let zeros = [0; 4];  // [0, 0, 0, 0]
let first = a[0];    // 인덱스는 0부터, 타입은 usize
```

원소 개수가 바뀌지 않는 데이터(요일, 월 이름 등)에 배열이 잘 맞습니다. 개수가 변해야 한다면 ch8에서 배울 `Vec`를 씁니다.

튜플과 배열은 `{}`로는 출력할 수 없고, 디버그 형식 `{:?}`로 출력합니다(`println!("{tup:?}")`).

→ [`examples/05_compound_types.rs`](examples/05_compound_types.rs)

### 4. 실행 중 검사: 정수 오버플로와 배열 범위 밖 접근

**패닉(panic)** 은 프로그램이 더 진행하면 안 되는 상황을 만났을 때 오류 메시지를 출력하고 즉시 멈추는 것입니다. 패닉으로 끝난 프로그램의 종료 코드는 101입니다. 패닉을 다루는 방법은 ch9에서 배웁니다.

Rust는 정수 오버플로와 배열 범위 밖 접근을 두 단계로 막습니다.

**컴파일러가 값을 미리 알 수 있으면 컴파일 에러로 막습니다.**

```text
let x: u8 = 255; let y = x + 1;   →  error: this arithmetic operation will overflow
let a = [1, 2, 3, 4, 5]; a[10]    →  error: this operation will panic at runtime
```

**실행해 봐야 아는 값(사용자 입력 등)이면 실행 중에 검사합니다.** 그래서 이 챕터의 04, 06 예제는 표준 입력으로 값을 받습니다.

#### 정수 오버플로

`u8`(0~255)에 250 + 10처럼 범위를 넘는 계산을 하는 것을 **오버플로(overflow)** 라고 합니다. `+` 연산자는 빌드 모드에 따라 다르게 동작합니다.

- **디버그 빌드**(`cargo run`, `cargo build`): 오버플로를 검사해서 `attempt to add with overflow` 패닉으로 멈춥니다.
- **릴리스 빌드**(`--release`): 속도를 위해 검사를 생략하고, 범위를 넘은 만큼 0부터 다시 세는 **래핑(wrapping)** 을 합니다. `250u8 + 10`은 `4`가 됩니다.

어느 쪽이든 의도하지 않은 오버플로는 버그입니다. 오버플로가 일어날 수 있는 계산이라면, 어떻게 처리할지 메서드로 밝혀 둡니다. 이 메서드들은 빌드 모드와 상관없이 항상 같은 결과를 냅니다.

| 메서드 계열 | 오버플로가 나면 | `250u8.xxx_add(10)` 결과 |
| --- | --- | --- |
| `wrapping_*` | 래핑한 값 | `4` |
| `checked_*` | `None` (성공하면 `Some(값)`) | `None` |
| `overflowing_*` | (래핑한 값, 오버플로 여부) 튜플 | `(4, true)` |
| `saturating_*` | 타입의 최댓값/최솟값에서 멈춤 | `255` |

`Some`과 `None`은 "값이 있을 수도 없을 수도 있음"을 나타내는 `Option` 타입의 값입니다. `Option`은 ch6에서 자세히 다룹니다.

→ [`examples/04_integer_overflow.rs`](examples/04_integer_overflow.rs)

#### 배열 범위 밖 접근

`a[index]`를 실행할 때마다 Rust는 `index`가 배열 길이보다 작은지 확인하고, 범위를 벗어나면 `index out of bounds` 패닉으로 멈춥니다. C 같은 언어에서는 이런 접근이 배열 뒤쪽의 엉뚱한 메모리를 조용히 읽어 버려 찾기 어려운 버그나 보안 취약점이 되기도 합니다. Rust는 잘못된 메모리를 읽느니 멈추는 쪽을 택합니다. 이 검사는 메모리 안전을 지키기 위한 것이어서 **릴리스 빌드에서도 꺼지지 않습니다.**

범위 밖일 수 있는 인덱스를 패닉 없이 확인하려면 `get`을 씁니다. `a.get(1)`은 `Some(2)`, `a.get(10)`은 `None`을 돌려줍니다.

→ [`examples/06_array_index.rs`](examples/06_array_index.rs), [`examples/05_compound_types.rs`](examples/05_compound_types.rs)

### 5. 함수

함수는 `fn`으로 정의하고, 이름은 `snake_case`로 짓습니다. 호출하는 곳보다 아래에 정의해도 됩니다.

```rust
fn rectangle_area(width: u32, height: u32) -> u32 {
    width * height
}
```

- **매개변수(parameter)** 에는 반드시 타입을 적습니다. 호출할 때 넘기는 구체적인 값은 **인자(argument)** 라고 부릅니다. 시그니처에 타입이 고정되어 있으므로 컴파일러는 함수 본문과 호출하는 쪽을 따로따로 검사할 수 있고, 그 대가로 다른 곳에서는 타입 표기를 거의 생략할 수 있습니다.
- **반환 타입**은 `->` 뒤에 적습니다. 함수 본문의 **마지막 표현식**(세미콜론이 없는 마지막 줄)의 값이 반환값입니다.
- 함수 중간에서 바로 값을 돌려주고 끝내려면 `return`을 씁니다(조기 반환). 함수 끝에서는 `return` 없이 마지막 표현식으로 반환하는 것이 관례입니다.
- 값 여러 개를 돌려주고 싶으면 튜플을 반환합니다: `fn divide(a: i32, b: i32) -> (i32, i32)`.

→ [`examples/07_functions.rs`](examples/07_functions.rs)

### 6. 구문과 표현식

Rust는 **표현식 기반** 언어입니다. 코드는 두 종류로 나뉩니다.

- **구문(statement)**: 어떤 동작을 하고 값을 내지 않습니다. `let` 선언과 함수 정의가 구문입니다.
- **표현식(expression)**: 계산해서 값을 냅니다. `5`, `x + 1`, 함수 호출, 매크로 호출, 중괄호 블록이 모두 표현식입니다.

`let`은 구문이어서 값이 없으므로 다른 변수에 담을 수 없습니다. C처럼 `x = y = 6`으로 한꺼번에 대입하는 문법도 없습니다.

```text
let x = (let y = 6);   →  error: expected expression, found `let` statement
```

**블록도 표현식입니다.** 블록의 마지막 표현식이 블록 전체의 값이 됩니다.

```rust
let y = {
    let x = 3;
    x + 1      // 세미콜론 없음 → 블록의 값은 4
};
```

**세미콜론은 표현식을 구문으로 바꿉니다.** 마지막 줄에 세미콜론을 붙이면 그 값은 계산된 뒤 버려지고, 블록의 값은 유닛 `()`가 됩니다. 함수 본문도 블록이므로 같은 규칙을 따릅니다. 반환 타입이 있는 함수의 마지막 줄에 세미콜론을 붙이는 것이 가장 흔한 실수입니다([자주 하는 실수](#자주-하는-실수) 3번).

같은 원리로 `if`와 `loop`도 값을 냅니다. 정리하면 다음과 같습니다.

| 표현식 | 값이 되는 것 | 예 |
| --- | --- | --- |
| 블록 `{ ... }` | 마지막 표현식 | `let y = { let x = 3; x + 1 };` |
| 함수 본문 | 마지막 표현식 (또는 `return`의 값) | `fn five() -> i32 { 5 }` |
| `if` | 실행된 분기 블록의 마지막 표현식 | `let n = if ok { 5 } else { 6 };` |
| `loop` | `break` 뒤에 적은 값 | `let r = loop { break 42; };` |

→ [`examples/08_expressions.rs`](examples/08_expressions.rs)

### 7. 제어 흐름

#### `if`

조건은 **반드시 `bool`** 이어야 합니다. JavaScript나 Python과 달리 Rust는 정수 같은 값을 `bool`로 자동 변환하지 않으므로, "0이 아니면"을 원한다면 `if number != 0`처럼 직접 비교합니다. 자동 변환이 없으니 조건이 무엇을 검사하는지 코드에 그대로 드러납니다.

`else if`를 이어 쓰면 위에서부터 검사해서 **처음으로 참인 분기 하나만** 실행합니다. 6은 2로도 나누어떨어지지만 `% 3 == 0` 분기가 먼저 참이면 그 아래 분기는 검사하지 않습니다.

`if`는 표현식이므로 다른 언어의 삼항 연산자(`? :`)처럼 쓸 수 있습니다. 이때 **모든 분기가 같은 타입의 값**을 내야 합니다. 변수의 타입이 컴파일할 때 하나로 정해져야 하기 때문입니다.

```rust
let number = if condition { 5 } else { 6 };       // 가능
let number = if condition { 5 } else { "six" };   // error[E0308]: `if` and `else` have incompatible types
```

→ [`examples/09_if.rs`](examples/09_if.rs)

#### `loop`, `break`, `continue`

`loop`는 `break`를 만날 때까지 끝없이 반복합니다. `continue`는 이번 반복의 나머지를 건너뛰고 다음 반복으로 넘어갑니다.

`break` 뒤에 값을 적으면 그 값이 `loop` 표현식의 값이 됩니다. "조건을 만족할 때까지 반복해서 찾은 값"을 변수에 바로 담을 때 씁니다.

```rust
let mut power = 1;
let first_over_1000 = loop {
    power *= 2;
    if power >= 1000 {
        break power;  // 1024
    }
};
```

`break`에 값을 실을 수 있는 반복문은 `loop`뿐입니다. `while`과 `for`는 `break` 없이 조건이나 원소가 떨어져서 끝날 수도 있으므로, 그때 낼 값이 정해지지 않기 때문입니다. `while` 안에서 `break 값`을 쓰면 다음 에러가 납니다.

```text
error[E0571]: `break` with value from a `while` loop
```

#### 루프 레이블

반복문이 중첩되어 있으면 `break`와 `continue`는 가장 안쪽 루프에만 적용됩니다. 바깥 루프를 지정하려면 루프 앞에 **레이블(label)** 을 붙입니다. 레이블은 작은따옴표로 시작하는 이름입니다.

```rust
let (found_dan, found_n) = 'outer: loop {
    // ...
    loop {
        if n > 9 { break; }                  // 안쪽 루프만 끝냄
        if dan * n > 50 { break 'outer (dan, n); }  // 바깥 루프를 끝내며 값을 냄
        // ...
    }
};
```

`continue 'outer`는 바깥 루프의 다음 반복으로 바로 넘어갑니다.

→ [`examples/10_loop.rs`](examples/10_loop.rs)

#### `while`과 `for`

`while`은 조건이 참인 동안 반복합니다. `loop` + `if` + `break`로도 만들 수 있지만 의도가 더 잘 드러납니다.

`for`는 배열의 원소나 범위(range)의 값을 하나씩 꺼내 반복합니다. 인덱스로 배열을 순회하는 `while`은 조건(`index < 5`)을 배열 길이와 직접 맞춰야 해서, 길이를 바꾸고 조건을 고치지 않으면 패닉하거나 원소를 빠뜨립니다. `for`는 인덱스를 다루지 않으므로 이런 실수가 생기지 않습니다. 그래서 Rust에서 가장 많이 쓰는 반복문은 `for`입니다.

```rust
for element in a { ... }                    // 배열의 원소
for i in 1..4 { ... }                       // 1, 2, 3 (끝 미포함)
for i in 1..=4 { ... }                      // 1, 2, 3, 4 (끝 포함)
for n in (1..4).rev() { ... }               // 3, 2, 1
for (i, day) in weekdays.iter().enumerate() { ... }  // (인덱스, 원소)
```

`iter`, `enumerate`, `rev` 같은 반복자(iterator) 메서드는 ch13에서 자세히 다룹니다.

→ [`examples/11_while_for.rs`](examples/11_while_for.rs)

### 8. 주석

Rust에서 관례적인 주석은 `//`로 시작해 줄 끝까지 이어집니다. 여러 줄이면 줄마다 `//`를 붙이고, 주석은 설명하는 코드의 바로 윗줄에 두는 경우가 많습니다. 이 저장소의 예제 주석도 모두 이 형식입니다. 문서를 만들어 주는 문서 주석(`///`)은 Part 4에서 다룹니다.

## 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_variables.rs`](examples/01_variables.rs) | 불변 변수, `mut`, `const` | 1 |
| [`02_shadowing.rs`](examples/02_shadowing.rs) | 섀도잉과 `mut`의 차이, 타입을 바꾸는 섀도잉 | 1 |
| [`03_scalar_types.rs`](examples/03_scalar_types.rs) | 정수·부동소수점 수·불리언·문자, 리터럴, 수치 연산, `as` | 2 |
| [`04_integer_overflow.rs`](examples/04_integer_overflow.rs) | 오버플로 처리 메서드 4종, 디버그 패닉과 릴리스 래핑 (**입력에 따라 패닉**) | 3 |
| [`05_compound_types.rs`](examples/05_compound_types.rs) | 튜플, 유닛, 배열, 패닉 없는 `get` | 2, 3 |
| [`06_array_index.rs`](examples/06_array_index.rs) | 범위 밖 인덱스의 실행 중 검사 (**입력에 따라 패닉**) | 3 |
| [`07_functions.rs`](examples/07_functions.rs) | 매개변수, 반환값, 튜플 반환, 조기 반환 | 4 |
| [`08_expressions.rs`](examples/08_expressions.rs) | 구문과 표현식, 블록의 값, 세미콜론 | 4 |
| [`09_if.rs`](examples/09_if.rs) | `bool` 조건, `else if`, 값을 내는 `if` | 4, 5 |
| [`10_loop.rs`](examples/10_loop.rs) | `loop`, `continue`, `break 값`, 루프 레이블 | 4, 5 |
| [`11_while_for.rs`](examples/11_while_for.rs) | `while`, `for`, 범위, `rev`, `enumerate` | 5 |
| [`12_fibonacci.rs`](examples/12_fibonacci.rs) | 연습 과제 풀이: n번째 피보나치 수 | 2~5 종합 |
| [`13_temperature.rs`](examples/13_temperature.rs) | 연습 과제 풀이: 화씨·섭씨 변환 | 2~5 종합 |

아래 출력은 프로그램이 출력한 내용만 옮긴 것입니다. `cargo run`은 그 앞에 `Compiling`, `Finished`, `Running` 같은 진행 상황을 함께 보여 줍니다(`-q`를 붙이면 생략됩니다).

### 01_variables — 불변 변수, `mut`, `const`

불변 변수와 가변 변수의 차이, 전역 상수와 함수 안 상수를 보여 줍니다. 마지막 부분은 불변 변수가 실행 중의 값(그 순간의 `count`)을 담을 수 있다는 점에서 상수와 다르다는 것을 보여 줍니다.

```sh
cargo run -p p0-ch3-common-concepts --example 01_variables
```

```text
x = 5
count = 0
count = 2
3시간 = 10800초
최대 점수 = 100000
snapshot = 2, count = 12
```

### 02_shadowing — 섀도잉과 `mut`

같은 "안쪽 블록에서 두 배로 만들기"를 섀도잉과 `mut`로 각각 해 보고, 블록이 끝난 뒤 바깥 변수의 값이 어떻게 다른지 비교합니다(`x`는 6으로 돌아오고 `y`는 12로 남음). 타입을 바꾸는 섀도잉과 값을 불변으로 고정하는 섀도잉도 있습니다.

```sh
cargo run -p p0-ch3-common-concepts --example 02_shadowing
```

```text
안쪽 스코프의 x = 12
바깥 스코프의 x = 6
안쪽 스코프의 y = 12
바깥 스코프의 y = 12
공백 개수 = 3
guess + 1 = 43
total = 30
```

### 03_scalar_types — 스칼라 타입

정수 타입의 범위와 리터럴, 부동소수점 수의 기본 타입과 출력 형식, 정수 나눗셈과 `as` 변환, `bool`, 한글과 이모지를 담은 `char`를 출력합니다. `usize 비트 수`는 64비트 환경 기준이며, 32비트 환경에서는 32가 나옵니다.

```sh
cargo run -p p0-ch3-common-concepts --example 03_scalar_types
```

```text
42 255 -128
i8: -128 ~ 127
u32 최댓값: 4294967295
usize 비트 수: 64
98222 255 63 240 65 57
2 3
2.0 3.0
0.1 + 0.2 = 0.30000000000000004
5 + 10 = 15
95.5 - 4.3 = 91.2
4 * 30 = 120
7 / 2 = 3
-5 / 3 = -1
7.0 / 2.0 = 3.5
43 % 5 = 3
평균 = 4.25
정수 나눗셈 = 4
true false true
z 한 ℤ 😻
'한'의 코드 포인트: U+D55C
```

`let x = 2.0;`을 `{}`로 출력하면 `2`로 보이지만 값은 여전히 `f64`입니다. `{:?}`로 출력하면 `2.0`으로 보입니다.

### 04_integer_overflow — 정수 오버플로 (입력에 따라 패닉)

입력받은 `u8` 값에 10을 더하는 방법 다섯 가지를 비교합니다. **디버그 빌드에서 246 이상을 입력하면 마지막 `+` 연산에서 일부러 패닉하며 끝납니다.** 같은 입력을 디버그 빌드와 릴리스 빌드로 실행해 결과를 비교해 보세요.

```sh
echo 100 | cargo run -p p0-ch3-common-concepts --example 04_integer_overflow
```

```text
0~255 사이의 수를 입력하세요. 여기에 10을 더합니다.
wrapping_add    : 110
checked_add     : Some(110)
overflowing_add : (110, false)
saturating_add  : 110
+ 연산자로 더해 봅니다...
+ 연산자        : 110
```

```sh
echo 250 | cargo run -p p0-ch3-common-concepts --example 04_integer_overflow
echo $?   # 종료 코드 확인
```

```text
0~255 사이의 수를 입력하세요. 여기에 10을 더합니다.
wrapping_add    : 4
checked_add     : None
overflowing_add : (4, true)
saturating_add  : 255
+ 연산자로 더해 봅니다...

thread 'main' (6117691) panicked at part0-getting-started/ch3-common-concepts/examples/04_integer_overflow.rs:56:15:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
101
```

`thread 'main'` 뒤 괄호 안의 숫자는 스레드 ID로, 실행할 때마다 달라집니다. `panicked at` 뒤에는 패닉이 난 파일, 줄, 열이 나옵니다. 마지막 `101`은 `echo $?`가 출력한 종료 코드입니다.

```sh
echo 250 | cargo run --release -p p0-ch3-common-concepts --example 04_integer_overflow
```

```text
0~255 사이의 수를 입력하세요. 여기에 10을 더합니다.
wrapping_add    : 4
checked_add     : None
overflowing_add : (4, true)
saturating_add  : 255
+ 연산자로 더해 봅니다...
+ 연산자        : 4
```

릴리스 빌드에서는 패닉 없이 래핑된 값 `4`가 나옵니다. 메서드 네 가지의 결과는 두 빌드에서 똑같습니다.

### 05_compound_types — 튜플과 배열

튜플의 구조 분해와 번호 접근, 유닛, 배열의 타입 표기와 `[값; 개수]`, 인덱싱, 그리고 범위 밖 인덱스를 패닉 없이 확인하는 `get`을 보여 줍니다.

```sh
cargo run -p p0-ch3-common-concepts --example 05_compound_types
```

```text
tup = (500, 6.4, '가')
x = 500, y = 6.4, z = 가
tup.0 = 500, tup.2 = 가
point = (3, 4)
unit = ()
a = [1, 2, 3, 4, 5], 길이 = 5
zeros = [0, 0, 0, 0]
첫 원소 = 1, 마지막 원소 = 5
셋째 요일 = 수
scores = [70, 85, 90]
a.get(1)  = Some(2)
a.get(10) = None
```

### 06_array_index — 범위 밖 인덱스 (입력에 따라 패닉)

입력받은 인덱스로 배열 `[10, 20, 30, 40, 50]`의 원소를 꺼냅니다. **0~4가 아닌 인덱스를 입력하면 일부러 패닉하며 끝납니다.** 릴리스 빌드(`--release`)로 실행해도 똑같이 패닉합니다.

```sh
echo 2 | cargo run -p p0-ch3-common-concepts --example 06_array_index
```

```text
배열 인덱스를 입력하세요 (0~4).
a[2] = 30
```

```sh
echo 10 | cargo run -p p0-ch3-common-concepts --example 06_array_index
echo $?   # 종료 코드 확인
```

```text
배열 인덱스를 입력하세요 (0~4).

thread 'main' (6122668) panicked at part0-getting-started/ch3-common-concepts/examples/06_array_index.rs:30:19:
index out of bounds: the len is 5 but the index is 10
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
101
```

패닉 메시지가 배열의 길이(5)와 요청한 인덱스(10)를 함께 알려 줍니다. 패닉 뒤의 `println!`은 실행되지 않습니다. 스레드 ID는 실행할 때마다 달라집니다.

### 07_functions — 함수

`main`보다 아래에 정의한 함수 호출, 타입이 있는 매개변수, 마지막 표현식으로 반환하기, 튜플로 값 두 개 반환하기, `return`으로 조기 반환하기를 보여 줍니다.

```sh
cargo run -p p0-ch3-common-concepts --example 07_functions
```

```text
안녕하세요!
측정값: 5h
plus_one(5) = 6
plus_one(plus_one(5)) = 7
넓이 = 12
17 / 5 = 몫 3, 나머지 2
distance(3, 10) = 7
distance(10, 3) = 7
```

### 08_expressions — 구문과 표현식

블록이 마지막 표현식의 값을 내는 것, 세미콜론을 붙이면 블록의 값이 `()`가 되는 것, 함수 본문도 같은 규칙을 따르는 것을 보여 줍니다. `nothing`에 `()`를 담는 부분은 clippy(Rust의 린트 도구)가 `this let-binding has unit value`라고 경고하는 코드입니다. 쓸모없는 `()`를 변수에 담는 것은 보통 실수이기 때문입니다. 여기서는 설명을 위해 일부러 쓴 것이므로 `#[expect(clippy::let_unit_value)]`로 이 경고가 의도된 것임을 표시했습니다.

```sh
cargo run -p p0-ch3-common-concepts --example 08_expressions
```

```text
b = 13
y = 4
area = 12
nothing = ()
plus_one(4) = 5
square_of_sum(1, 2) = 9
```

### 09_if — `if` 표현식

`bool` 조건, 처음으로 참인 분기 하나만 실행되는 `else if`, 변수에 값을 담는 `if`, 함수의 마지막 표현식으로 쓰는 `if`를 보여 줍니다.

```sh
cargo run -p p0-ch3-common-concepts --example 09_if
```

```text
조건이 참입니다
number는 0이 아닙니다
6은 3으로 나누어떨어집니다
value = 5
28도: 적당한 날씨입니다
sign(-7) = -1
sign(0) = 0
sign(42) = 1
```

### 10_loop — `loop`, `break 값`, 루프 레이블

`continue`로 짝수 건너뛰기, `break 값`으로 처음 1000 이상이 되는 2의 거듭제곱 찾기, `break 'outer (dan, n)`으로 구구단에서 처음 50을 넘는 곱을 찾으며 바깥 루프까지 끝내기, `continue 'dans`로 바깥 루프의 다음 반복으로 넘어가기를 보여 줍니다. `break 'outer`로 빠져나간 6단에서는 "6단에는 ..." 줄이 출력되지 않는다는 점을 확인해 보세요.

```sh
cargo run -p p0-ch3-common-concepts --example 10_loop
```

```text
홀수: 1
홀수: 3
홀수: 5
홀수: 7
처음으로 1000 이상이 되는 2의 거듭제곱: 1024
2단에는 50을 넘는 곱이 없습니다
3단에는 50을 넘는 곱이 없습니다
4단에는 50을 넘는 곱이 없습니다
5단에는 50을 넘는 곱이 없습니다
처음으로 50을 넘는 곱: 6 x 9 = 54
3단의 첫 짝수 곱: 3 x 2 = 6
4단의 첫 짝수 곱: 4 x 1 = 4
5단의 첫 짝수 곱: 5 x 2 = 10
```

### 11_while_for — `while`과 `for`

`while` 카운트다운, 같은 배열을 인덱스 `while`과 `for`로 각각 순회하기, 범위 `1..4`와 `1..=100`, `rev`로 거꾸로 세기, `enumerate`로 인덱스와 원소를 함께 꺼내기를 보여 줍니다.

```sh
cargo run -p p0-ch3-common-concepts --example 11_while_for
```

```text
3!
2!
1!
발사!
[10][20][30][40][50]
[10][20][30][40][50]
[1][2][3]
1부터 100까지의 합 = 5050
3!
2!
1!
발사!
0번째 요일: 월
1번째 요일: 화
2번째 요일: 수
```

### 12_fibonacci — 피보나치 수 (The Book 연습 과제 풀이)

The Book 3장 끝의 연습 과제 "n번째 피보나치 수 구하기"의 풀이입니다. `for`로 두 값을 한 칸씩 밀며 계산하고, `(prev, curr) = (curr, prev + curr)`처럼 튜플로 두 변수를 한꺼번에 갱신합니다. 마지막 부분은 `overflowing_add`와 `break 값`으로 `u64`에 담을 수 있는 마지막 피보나치 수를 패닉 없이 찾습니다. F(94)는 `u64`의 범위를 넘으므로 `fibonacci(94)`를 호출하면 디버그 빌드에서 오버플로 패닉이 납니다.

```sh
cargo run -p p0-ch3-common-concepts --example 12_fibonacci
```

```text
0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377
F(50) = 12586269025
F(93) = 12200160415121876738
u64에 담을 수 있는 마지막 피보나치 수: F(93) = 12200160415121876738
```

### 13_temperature — 온도 변환 (The Book 연습 과제 풀이)

The Book 3장 끝의 연습 과제 "화씨와 섭씨 변환"의 풀이입니다. 변환 함수 두 개를 만들고, 배열과 범위를 `for`로 돌며 변환표를 출력합니다. `{:>7.1}`은 오른쪽 정렬, 너비 7칸, 소수점 아래 1자리로 출력하는 서식입니다. 마지막 줄은 실수의 오차를 고려해 `==` 대신 차이의 절댓값으로 왕복 변환 결과를 비교합니다.

```sh
cargo run -p p0-ch3-common-concepts --example 13_temperature
```

```text
화씨 → 섭씨
  -40.0°F =  -40.0°C
    0.0°F =  -17.8°C
   32.0°F =    0.0°C
   98.6°F =   37.0°C
  212.0°F =  100.0°C
섭씨 → 화씨
  -20.0°C =   -4.0°F
    0.0°C =   32.0°F
   20.0°C =   68.0°F
   40.0°C =  104.0°F
   60.0°C =  140.0°F
   80.0°C =  176.0°F
36.5°C → 화씨 → 섭씨 = 36.5°C, 같은가? true
```

## 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일 에러를 내는 코드입니다. 이 챕터 디렉터리(`part0-getting-started/ch3-common-concepts`)에서 다음처럼 직접 컴파일해 에러 메시지를 확인합니다. 컴파일에 실패하므로 실행 파일은 만들어지지 않습니다.

```sh
rustc --edition 2024 compile_fail/e0384_assign_twice_to_immutable.rs
```

에러 메시지는 보통 `error[코드]: 요약` 한 줄, `-->` 뒤의 위치(파일:줄:열), 문제가 된 코드에 `^^^`로 밑줄을 친 설명, 그리고 `help:`로 시작하는 고치는 방법 제안 순서로 나옵니다. 코드의 자세한 설명은 `rustc --explain E0384`처럼 확인할 수 있습니다.

### 1. 불변 변수에 다시 대입하기 (E0384)

[`compile_fail/e0384_assign_twice_to_immutable.rs`](compile_fail/e0384_assign_twice_to_immutable.rs)

```text
error[E0384]: cannot assign twice to immutable variable `x`
  --> compile_fail/e0384_assign_twice_to_immutable.rs:11:5
   |
 9 |     let x = 5;
   |         - first assignment to `x`
10 |     println!("x = {x}");
11 |     x = 6;
   |     ^^^^^ cannot assign twice to immutable variable
   |
help: consider making this binding mutable
   |
 9 |     let mut x = 5;
   |         +++
```

- **원인**: `let x = 5;`로 만든 `x`는 불변입니다. 메시지의 "assign twice"는 불변 변수에 값을 묶을 수 있는 기회가 한 번뿐이라는 뜻입니다. 컴파일러가 첫 대입 위치(`first assignment`)와 두 번째 대입 위치를 함께 짚어 줍니다.
- **해결**: 값이 계속 바뀌어야 한다면 `help:`의 제안대로 `let mut x = 5;`로 선언합니다(`+++`는 추가할 글자 위치입니다). 새 값으로 다시 시작하는 것이 의도라면 `let x = 6;`으로 섀도잉합니다.

### 2. `mut` 변수에 다른 타입의 값 넣기 (E0308)

[`compile_fail/e0308_mut_cannot_change_type.rs`](compile_fail/e0308_mut_cannot_change_type.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_mut_cannot_change_type.rs:10:14
   |
 9 |     let mut spaces = "   ";
   |                      ----- expected due to this value
10 |     spaces = spaces.len();
   |              ^^^^^^^^^^^^ expected `&str`, found `usize`
```

- **원인**: `spaces`의 타입은 첫 값 `"   "` 때문에 `&str`로 정해졌습니다(`expected due to this value`). `mut`는 같은 타입 안에서 값을 바꾸게 해 줄 뿐 타입을 바꾸지는 못하므로, `usize` 값을 넣을 수 없습니다. "기대한 타입(expected)"과 "실제로 받은 타입(found)"을 비교해 읽으면 됩니다.
- **해결**: `let spaces = spaces.len();`처럼 섀도잉으로 새 변수를 만듭니다.

### 3. 함수의 마지막 줄에 세미콜론 붙이기 (E0308)

[`compile_fail/e0308_tail_expression_semicolon.rs`](compile_fail/e0308_tail_expression_semicolon.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_tail_expression_semicolon.rs:9:24
   |
 9 | fn plus_one(x: i32) -> i32 {
   |    --------            ^^^ expected `i32`, found `()`
   |    |
   |    implicitly returns `()` as its body has no tail or `return` expression
10 |     x + 1;
   |          - help: remove this semicolon to return this value
```

- **원인**: `x + 1;`에 세미콜론이 붙어 표현식이 구문이 되었습니다. 구문은 값을 내지 않으므로 함수 본문에 마지막 표현식(tail expression)이 없고, 본문의 값은 `()`가 됩니다. 반환 타입은 `i32`라고 약속했는데 `()`를 돌려주므로 타입이 맞지 않습니다. 구문을 값처럼 쓰려 한 실수라는 점에서 `let x = (let y = 6);`과 같은 계열입니다.
- **해결**: 컴파일러의 `help:` 제안대로 마지막 줄의 세미콜론을 지웁니다.

### 4. `if` 조건에 정수 쓰기 (E0308)

[`compile_fail/e0308_if_condition_not_bool.rs`](compile_fail/e0308_if_condition_not_bool.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_if_condition_not_bool.rs:11:8
   |
11 |     if number {
   |        ^^^^^^ expected `bool`, found integer
```

- **원인**: `if`의 조건 자리는 `bool`을 기대하는데 정수가 들어왔습니다. C, JavaScript, Python에서는 "0이 아니면 참"으로 해석하지만 Rust는 자동 변환을 하지 않습니다.
- **해결**: 원하는 조건을 `if number != 0`처럼 비교식으로 직접 씁니다.

## 연습 문제

이 챕터에서 배운 내용만으로 풀 수 있습니다. 풀이는 이 챕터의 `examples/`에 새 파일(예: `examples/14_fizzbuzz.rs`)을 만들어 `cargo run -p p0-ch3-common-concepts --example 14_fizzbuzz`로 실행해 보면 편합니다.

### 1. FizzBuzz

1부터 30까지 수를 출력하되, 3의 배수는 `Fizz`, 5의 배수는 `Buzz`, 3과 5의 공배수는 `FizzBuzz`로 바꿔 출력하세요.

<details>
<summary>힌트</summary>

- `for n in 1..=30`으로 반복하고 `%`로 나머지를 검사합니다.
- `else if`는 처음으로 참인 분기 하나만 실행합니다. 15의 배수를 3의 배수보다 **먼저** 검사하지 않으면 15가 `Fizz`로 출력됩니다.

</details>

### 2. 초를 시·분·초로 바꾸기

전체 초를 받아 (시, 분, 초) 튜플을 돌려주는 함수 `fn split_seconds(total: u32) -> (u32, u32, u32)`를 작성하세요. 3600, 60 같은 수는 `const`로 이름을 붙여 쓰고, `main`에서 구조 분해로 받아 `3725초 = 1시간 2분 5초`처럼 출력하세요.

<details>
<summary>힌트</summary>

- `const SECONDS_PER_MINUTE: u32 = 60;`, `const SECONDS_PER_HOUR: u32 = 60 * SECONDS_PER_MINUTE;`처럼 상수를 다른 상수로 정의할 수 있습니다.
- 정수 나눗셈 `/`는 몫, `%`는 나머지입니다. 시는 `total / 3600`, 남은 초는 `total % 3600`입니다.
- 함수의 마지막 줄을 `(hours, minutes, seconds)`처럼 세미콜론 없이 쓰면 튜플이 반환됩니다.

</details>

### 3. 배열의 최솟값·최댓값·합계

`[i32; 6]` 배열을 받아 (최솟값, 최댓값, 합계)를 돌려주는 함수를 작성하세요. 예를 들어 `[7, -3, 12, 0, 5, -8]`이면 `(-8, 12, 13)`입니다.

<details>
<summary>힌트</summary>

- 매개변수 타입은 `numbers: [i32; 6]`, 반환 타입은 `(i32, i32, i32)`입니다.
- 최솟값과 최댓값은 `numbers[0]`으로 시작하고, `for n in numbers`로 돌면서 `if`로 비교해 갱신합니다. 갱신할 변수에는 `mut`가 필요합니다.
- 합계를 0으로 시작하면 원소가 모두 음수여도 올바르게 계산됩니다. 최솟값을 0으로 시작하면 왜 틀릴 수 있는지도 생각해 보세요.

</details>

### 4. 콜라츠 수열의 단계 수

양의 정수 n에서 시작해 짝수면 2로 나누고 홀수면 `3n + 1`로 바꾸는 일을 1이 될 때까지 반복합니다. 1이 될 때까지 걸린 단계 수를 돌려주는 함수 `fn collatz_steps(n: u64) -> u32`를 `loop`와 `break 값`으로 작성하세요. 그다음 1부터 30 중 단계 수가 가장 많은 수와 그 단계 수를 출력하세요. (예: 6 → 3 → 10 → 5 → 16 → 8 → 4 → 2 → 1은 8단계)

<details>
<summary>힌트</summary>

- 매개변수를 바꿔 가며 계산하려면 `fn collatz_steps(mut n: u64) -> u32`처럼 매개변수에도 `mut`를 붙일 수 있습니다. 매개변수도 변수이기 때문입니다.
- 루프 안에서 `n == 1`이면 `break steps;`로 단계 수를 내보냅니다. 1에서 시작하면 0단계입니다.
- 가장 긴 수를 찾을 때는 `for i in 1..=30` 바깥에 `let mut best = (1, 0);` 같은 튜플을 두고 갱신하면 편합니다. 정답은 27(111단계)입니다.
- 더 해 보기: 단계 수가 100을 넘는 첫 수를 찾으면 바로 멈추도록 바꿔 보세요. `for` 반복문에서도 `break`(값 없이)는 쓸 수 있습니다.

</details>

## 참고 자료

**The Book**

- [3. Common Programming Concepts](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html)
  - [3.1 Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
  - [3.2 Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)
  - [3.3 Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html)
  - [3.4 Comments](https://doc.rust-lang.org/book/ch03-04-comments.html)
  - [3.5 Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html)

**Rust By Example** — 같은 주제를 짧은 예제 위주로 다시 볼 수 있습니다.

- [Primitives](https://doc.rust-lang.org/rust-by-example/primitives.html): [Literals and operators](https://doc.rust-lang.org/rust-by-example/primitives/literals.html), [Tuples](https://doc.rust-lang.org/rust-by-example/primitives/tuples.html), [Arrays and Slices](https://doc.rust-lang.org/rust-by-example/primitives/array.html) (슬라이스는 ch4에서 다룹니다)
- [Variable Bindings](https://doc.rust-lang.org/rust-by-example/variable_bindings.html): [Mutability](https://doc.rust-lang.org/rust-by-example/variable_bindings/mut.html), [Scope and Shadowing](https://doc.rust-lang.org/rust-by-example/variable_bindings/scope.html), [Declare first](https://doc.rust-lang.org/rust-by-example/variable_bindings/declare.html)
- [Types](https://doc.rust-lang.org/rust-by-example/types.html): [Casting](https://doc.rust-lang.org/rust-by-example/types/cast.html), [Literals](https://doc.rust-lang.org/rust-by-example/types/literals.html), [Inference](https://doc.rust-lang.org/rust-by-example/types/inference.html)
- [constants](https://doc.rust-lang.org/rust-by-example/custom_types/constants.html) (Custom Types 절 아래에 있습니다)
- [Expressions](https://doc.rust-lang.org/rust-by-example/expression.html)
- [Flow of Control](https://doc.rust-lang.org/rust-by-example/flow_control.html): [if/else](https://doc.rust-lang.org/rust-by-example/flow_control/if_else.html), [loop](https://doc.rust-lang.org/rust-by-example/flow_control/loop.html), [Nesting and labels](https://doc.rust-lang.org/rust-by-example/flow_control/loop/nested.html), [Returning from loops](https://doc.rust-lang.org/rust-by-example/flow_control/loop/return.html), [while](https://doc.rust-lang.org/rust-by-example/flow_control/while.html), [for and range](https://doc.rust-lang.org/rust-by-example/flow_control/for.html)
- [Functions](https://doc.rust-lang.org/rust-by-example/fn.html) (첫 페이지만 해당합니다. 하위 절의 메서드와 클로저는 ch5, ch13에서 다룹니다)

**Rustlings** — 컴파일 에러를 하나씩 고쳐 가며 푸는 연습 문제입니다.

- [`01_variables`](https://github.com/rust-lang/rustlings/tree/main/exercises/01_variables): 불변·가변 변수, 섀도잉, 상수
- [`02_functions`](https://github.com/rust-lang/rustlings/tree/main/exercises/02_functions): 매개변수와 반환값
- [`03_if`](https://github.com/rust-lang/rustlings/tree/main/exercises/03_if): `if` 표현식
- [`04_primitive_types`](https://github.com/rust-lang/rustlings/tree/main/exercises/04_primitive_types): 스칼라·복합 타입 (일부 문제는 ch4의 슬라이스를 사용합니다)
