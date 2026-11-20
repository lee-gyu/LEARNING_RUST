# ch6. 열거형과 패턴 매칭

구조체가 여러 값을 "모두 함께" 묶는 타입이라면, 열거형(enum)은 값이 "여러 경우 중 정확히 하나"라는 것을 나타내는 타입입니다. 이 챕터에서는 배리언트마다 다른 데이터를 담는 열거형을 만들고, ch2에서 맛보기로 쓴 `match`를 제대로 배웁니다. 그리고 Rust가 null 없이 "값이 없을 수 있음"을 표현하는 `Option<T>`와, 한 가지 경우만 관심 있을 때 쓰는 `if let`, `let...else`를 다룹니다.

- 패키지: `p0-ch6-enums-pattern-matching`
- 기준 문서: [The Book 6장 Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
- 확인한 툴체인: rustc 1.100.0, Edition 2024. 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 배리언트마다 다른 형태(데이터 없음·튜플형·구조체형)의 데이터를 담는 열거형을 정의하고 `impl`로 메서드를 붙일 수 있으며, 같은 내용을 구조체로 표현했을 때보다 열거형이 나은 점을 설명할 수 있다.
2. `match`로 배리언트마다 다른 코드를 실행하고 패턴으로 안의 값을 꺼낼(바인딩) 수 있으며, 완전성 검사가 막아 주는 실수와 포괄 패턴(`other`, `_`)을 언제 쓰고 언제 피할지 설명할 수 있다.
3. Rust에 null이 없는 이유와 `Option<T>`가 `T`와 다른 타입인 이유를 설명하고, `Option` 값을 `match`와 `unwrap_or` 등으로 처리할 수 있다.
4. `match`에서 값이 바인딩된 변수로 이동하는 경우와 참조로 매칭해 원본이 남는 경우를 구분할 수 있다.
5. `if let`/`else`와 `let...else`로 한 가지 패턴만 처리하는 코드를 간결하게 쓰고, 각각 `match`와 비교해 무엇을 얻고 무엇을 잃는지 설명할 수 있다.

## 2. 핵심 개념

### 2.1 열거형: 가능한 경우를 나열한 타입

ch2에서 쓴 `Ordering`은 `Less`, `Greater`, `Equal` 세 경우 중 하나인 열거형이었습니다. 직접 정의할 때는 `enum` 키워드 뒤에 가능한 경우, 즉 **배리언트(variant)** 를 나열합니다.

```rust
enum StatusKind {
    Received,
    Shipped,
    Delivered,
    Cancelled,
}

let kind = StatusKind::Shipped;
```

- 배리언트는 열거형 이름 아래에 있으므로 `StatusKind::Shipped`처럼 `::`로 부릅니다.
- 네 배리언트는 모두 **같은 타입** `StatusKind`의 값입니다. 그래서 `fn route(kind: StatusKind)` 같은 함수 하나가 네 경우를 모두 받을 수 있습니다.
- `StatusKind` 타입의 값은 언제나 넷 중 **정확히 하나**입니다. 둘 다이거나 아무것도 아닌 값은 존재할 수 없습니다.

여기까지는 C나 Java의 enum과 비슷합니다. Rust 열거형이 다른 점은 다음 절처럼 **배리언트마다 서로 다른 데이터를 담을 수 있다**는 것입니다. TypeScript의 판별 유니언(discriminated union)이나 Swift의 연관값을 가진 enum, Kotlin의 sealed class를 써 봤다면 그쪽에 더 가깝습니다.

### 2.2 배리언트에 데이터 담기

배리언트는 ch5에서 배운 구조체의 세 가지 모양을 그대로 가질 수 있습니다.

```rust
enum OrderStatus {
    Received,                                   // 데이터 없음
    Shipped { courier: String, tracking: u32 }, // 구조체형: 이름 붙은 필드
    Delivered(String),                          // 튜플형: 수령 장소
    Cancelled(String),                          // 튜플형: 취소 사유
}

let shipped = OrderStatus::Shipped {
    courier: String::from("번개택배"),
    tracking: 1234,
};
let cancelled = OrderStatus::Cancelled(String::from("단순 변심"));
```

| 배리언트 모양 | 정의 | 값 만들기 | 비슷한 구조체(ch5) |
| --- | --- | --- | --- |
| 데이터 없음 | `Received` | `OrderStatus::Received` | 유닛 구조체 |
| 튜플형 | `Cancelled(String)` | `OrderStatus::Cancelled(String::from("..."))` | 튜플 구조체 |
| 구조체형 | `Shipped { courier: String, tracking: u32 }` | `OrderStatus::Shipped { courier: ..., tracking: ... }` | 일반 구조체 |

튜플형 배리언트의 이름은 값을 받아 열거형 값을 돌려주는 함수처럼 쓰입니다. `OrderStatus::Cancelled`에 `String`을 넣으면 `OrderStatus` 값이 나옵니다. 튜플형은 값의 의미가 괄호 안 위치로만 정해지므로, 값이 하나이거나 의미가 분명할 때 씁니다. 값이 여럿이라 헷갈릴 수 있으면 필드 이름이 붙는 구조체형이 읽기 쉽습니다.

`Delivered(String)`과 `Cancelled(String)`은 담는 데이터의 타입이 같지만 서로 다른 배리언트입니다. 같은 `String`이라도 수령 장소인지 취소 사유인지를 배리언트가 구분해 줍니다.

### 2.3 구조체 대신 열거형을 쓰는 이유

같은 내용을 구조체만으로 표현하는 방법은 두 가지가 있습니다. 둘 다 문제가 있습니다.

**(가) 구조체 하나 + "종류" 필드**

```rust
struct OrderRecord {
    kind: StatusKind,
    courier: String, // Shipped일 때만 의미가 있음
    tracking: u32,   // Shipped일 때만 의미가 있음
    place: String,   // Delivered일 때만 의미가 있음
    reason: String,  // Cancelled일 때만 의미가 있음
}
```

- 구조체는 모든 필드를 항상 채워야 하므로, 접수 단계의 주문에도 택배사와 운송장 번호를 넣어야 합니다. 그래서 `String::new()`나 `0`을 넣고 "이 값은 없다는 뜻"이라고 약속하게 되는데, 이 약속은 코드 어디에도 드러나지 않습니다.
- `kind`는 `Cancelled`인데 운송장 번호가 있고 취소 사유는 빈 문자열인 값처럼, **말이 안 되는 조합**도 만들 수 있습니다. 규칙이 타입에 들어 있지 않으니 컴파일러는 이상하다는 것을 알 수 없습니다.

**(나) 배리언트마다 구조체를 따로**

```rust
struct Received;
struct Shipped { courier: String, tracking: u32 }
// ...
```

이렇게 하면 각 구조체는 필요한 데이터만 갖지만, 이번에는 **타입이 서로 달라집니다.** 한 배열에 담으려 하면 배열의 모든 원소는 타입이 같아야 하므로 컴파일되지 않습니다.

```
error[E0308]: mismatched types
...
4 |     let history = [Received, Shipped { courier: String::from("a"), tracking: 1 }];
  |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Received`, found `Shipped`
```

함수도 마찬가지로, "어느 상태든 받는 함수" 하나를 만들 수 없고 타입마다 함수를 따로 만들어야 합니다.

**열거형은 두 장점을 모두 가집니다.** 각 배리언트는 자기에게 필요한 데이터만 갖고(가의 문제 해결), 모든 배리언트가 한 타입이라 한 배열에 담고 한 함수로 받을 수 있습니다(나의 문제 해결). "취소됐는데 운송장 번호가 있는" 값은 아예 만들 방법이 없습니다. `Cancelled`에는 사유만 들어가기 때문입니다.

정리하면 구조체는 "A **그리고** B"를, 열거형은 "A **또는** B"를 표현하는 타입입니다. 데이터의 모양이 경우에 따라 달라진다면 열거형으로 표현해, 잘못된 상태를 타입 수준에서 만들 수 없게 하는 것이 Rust다운 설계입니다. [`01_enum_variants.rs`](examples/01_enum_variants.rs)가 (가)와 열거형을 나란히 보여 줍니다.

### 2.4 `match`: 배리언트마다 갈래 나누기

열거형 값이 어느 배리언트인지에 따라 다른 코드를 실행할 때 `match`를 씁니다.

```rust
enum Drink {
    Water,
    Coffee { shots: u32, iced: bool },
    Juice(u32), // 용량(ml)
}

fn price(drink: Drink) -> u32 {
    match drink {
        Drink::Water => 0,
        Drink::Coffee { shots, iced } => {
            let base = 2000 + 500 * shots;
            if iced { base + 500 } else { base }
        }
        Drink::Juice(ml) => ml * 10,
    }
}
```

- `match 값 { 패턴 => 표현식, ... }` 모양입니다. `=>` 왼쪽이 패턴, 오른쪽이 실행할 코드이고, 이 한 쌍을 **갈래(arm)** 라고 부릅니다.
- 값을 **위에서부터 차례로** 각 갈래의 패턴과 맞춰 보고, **처음 맞는 갈래 하나만** 실행합니다. 동전을 크기별 구멍에 굴려 처음 맞는 구멍으로 떨어뜨리는 분류기를 떠올리면 됩니다.
- `if`의 조건은 `bool`이어야 하지만, `match`는 어떤 타입의 값이든 받을 수 있습니다.
- `match`는 **표현식**입니다. 실행된 갈래의 값이 `match` 전체의 값이 되므로, 위 함수에서는 `match`가 마지막 표현식으로서 그대로 반환됩니다. 그래서 모든 갈래의 값은 타입이 같아야 합니다.
- 갈래에 여러 줄이 필요하면 중괄호 블록으로 감쌉니다. 블록의 마지막 표현식이 그 갈래의 값이고, 블록으로 된 갈래 뒤의 쉼표는 생략할 수 있습니다.

### 2.5 패턴으로 값 꺼내기 (바인딩)

위 코드의 `Drink::Coffee { shots, iced }`에서 `shots`와 `iced`는 **이 자리에서 새로 만드는 변수**입니다. `drink`가 `Coffee`이면 그 안에 든 값이 이 이름들에 묶입니다. 이것을 **바인딩(binding)** 이라고 합니다.

- 구조체형 배리언트는 필드 이름으로 꺼냅니다: `Drink::Coffee { shots, iced }`
- 튜플형 배리언트는 괄호 안의 위치로 꺼내며, 변수 이름은 자유롭게 짓습니다: `Drink::Juice(ml)`
- 바인딩된 변수는 그 갈래 안에서만 쓸 수 있습니다.

패턴은 "이 값이 `Coffee`인가?"를 검사하는 일과 "`Coffee`라면 안의 데이터를 꺼내는" 일을 **한 번에** 합니다. 그래서 `Coffee`인지 확인하지 않고 `shots`를 읽는 코드는 쓸 방법이 없습니다. 2.3절 (가)의 구조체에서는 `kind`를 확인하는 것을 잊고 `tracking`을 읽는 실수가 가능했지만, 열거형에서는 데이터에 접근하는 유일한 통로가 배리언트를 확인하는 패턴입니다.

### 2.6 열거형에 메서드 정의하기

구조체처럼 열거형에도 `impl` 블록으로 연관 함수와 메서드를 정의합니다. 열거형의 메서드는 대개 `match self`로 배리언트를 나눕니다.

```rust
impl Drink {
    // 연관 함수: Drink::americano()로 호출합니다.
    fn americano() -> Self {
        Self::Coffee { shots: 2, iced: false }
    }

    // 메서드: order.describe()로 호출합니다.
    fn describe(&self) -> String {
        match self {
            Self::Water => String::from("물"),
            Self::Coffee { shots, iced } => {
                let temperature = if *iced { "아이스" } else { "따뜻한" };
                format!("{temperature} 커피 {shots}샷")
            }
            Self::Juice(ml) => format!("주스 {ml}ml"),
        }
    }
}
```

- `impl` 블록 안에서 `Self`는 `Drink`를 뜻하므로, `Self::Water`와 `Drink::Water`는 같습니다.
- `format!`은 `println!`과 같은 서식 문자열을 받지만, 출력하는 대신 그 결과를 `String`으로 돌려줍니다.
- `self`의 타입은 `&Drink`, 즉 참조입니다. 참조를 `match`하면 바인딩된 변수도 참조가 되어 `shots`는 `&u32`, `iced`는 `&bool`이 됩니다. `if`의 조건은 `bool`이어야 하므로 `*iced`로 참조를 따라가 값을 읽습니다(ch4의 역참조). `{shots}`처럼 출력할 때는 참조도 값처럼 그대로 쓸 수 있습니다. 이 규칙은 2.9절에서 자세히 다룹니다.

전체 코드는 [`02_match_binding.rs`](examples/02_match_binding.rs)에 있습니다.

### 2.7 `Option<T>`: null 대신 쓰는 열거형

#### null의 문제

많은 언어에는 "값이 없음"을 뜻하는 null(또는 nil, None)이 있습니다. null을 처음 도입한 토니 호어(Tony Hoare)는 이것을 "10억 달러짜리 실수"라고 불렀습니다. 문제는 null 자체가 아니라, **타입만 보고는 값이 null일 수 있는지 알 수 없다**는 데 있습니다. Java의 `String` 변수는 문자열일 수도 null일 수도 있으므로, 원칙대로라면 쓸 때마다 null인지 확인해야 합니다. 하지만 사람은 확인을 빠뜨리고, 그러면 실행 중에 `NullPointerException` 같은 에러가 납니다.

#### Rust의 해결: 없을 수 있는 값은 타입이 다르다

Rust에는 null이 없습니다. 대신 "값이 있거나 없음"을 표준 라이브러리의 열거형 `Option<T>`로 표현합니다.

```rust
enum Option<T> {
    None,
    Some(T),
}
```

- `<T>`는 "T 자리에 어떤 타입이든 올 수 있다"는 표시입니다. `Some(5)`는 `Option<i32>`, `Some('가')`는 `Option<char>`가 됩니다. 이런 문법을 제네릭(generic)이라고 하며 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 배웁니다.
- `Option`, `Some`, `None`은 모든 프로그램에 자동으로 가져와지는 이름들(prelude)에 들어 있어서, `Option::Some`이라고 쓰지 않고 `Some`, `None`으로 바로 씁니다.
- `None`만 보고는 무엇의 "없음"인지 알 수 없으므로, `let absent: Option<i32> = None;`처럼 타입을 적어 줘야 합니다. 적지 않으면 E0282(`` type annotations needed for `Option<_>` ``)가 납니다.

핵심은 **`Option<i32>`와 `i32`가 서로 다른 타입**이라는 점입니다.

```rust
let x: i8 = 5;
let y: Option<i8> = Some(5);
let sum = x + y; // E0277: cannot add `Option<i8>` to `i8`
```

`y`는 `None`일 수도 있으므로, 그 경우를 정하지 않은 채 `i8`처럼 쓸 수 없습니다. 거꾸로 말하면, **타입이 `i8`인 값은 언제나 유효한 `i8`이므로 확인할 필요가 없습니다.** "없을 수 있음"이 타입에 드러나 있으니, 확인이 필요한 곳(`Option`)과 필요 없는 곳(그 밖의 모든 타입)이 컴파일러 눈에 분명히 구분됩니다. null이 있는 언어에서 실행 중에 터지던 실수가 Rust에서는 컴파일 에러(4.2절)가 됩니다.

#### `Option`을 돌려주는 함수들

결과가 없을 수 있는 함수는 `Option`을 돌려줍니다. 반환 타입만 봐도 "없을 수 있다"는 것이 드러나므로, 호출하는 쪽은 그 경우를 처리하지 않을 수 없습니다.

```rust
"hello".find('l')     // Some(2): 찾은 바이트 위치
"hello".find('z')     // None
10_i32.checked_div(0) // None: 0으로 나눌 수 없음
```

ch3에서 본 `checked_add`와 배열의 `get`도 같은 이유로 `Option`을 돌려줍니다. 직접 만드는 함수도 마찬가지입니다.

```rust
fn first_even(numbers: &[i32]) -> Option<i32> {
    for n in numbers {
        if n % 2 == 0 {
            return Some(*n);
        }
    }
    None
}
```

#### 안의 값을 꺼내는 방법

| 방법 | `Some(v)`일 때 | `None`일 때 | 다루는 곳 |
| --- | --- | --- | --- |
| `match` | 갈래에서 `v`를 바인딩 | `None` 갈래에서 할 일을 직접 정함 | 2.4절 |
| `if let`, `let...else` | 블록에서 `v`를 바인딩 | `else`에서 할 일을 정함 | 2.10, 2.11절 |
| `opt.unwrap_or(기본값)` | `v` | 기본값 | 이 절 |
| `opt.unwrap()`, `opt.expect("메시지")` | `v` | 패닉으로 프로그램 종료 | 이 절 |
| `opt.is_some()`, `opt.is_none()` | `true`, `false` (값은 꺼내지 않음) | `false`, `true` | 이 절 |

`unwrap()`은 `None`이면 ``called `Option::unwrap()` on a `None` value``라는 메시지와 함께 패닉합니다. ch2에서 `Result`에 쓴 `expect`처럼, "여기서는 절대 `None`일 수 없다"고 확신할 때만 씁니다. `map`, `and_then`, `ok_or` 같은 더 많은 메서드는 [Part 1 ch2](../../part1-standard-lib/ch2-option-result/README.md)에서 다룹니다. 예제는 [`03_option.rs`](examples/03_option.rs)에 있습니다.

### 2.8 완전성 검사와 포괄 패턴

#### `match`는 모든 경우를 다뤄야 합니다

`match`의 갈래는 가능한 값을 **빠짐없이** 다뤄야 합니다. 이것을 **완전성 검사(exhaustiveness checking)** 라고 합니다. 하나라도 빠지면 E0004로 컴파일되지 않습니다(4.1절). `Option`을 `match`하면서 `None`을 빠뜨릴 수 없다는 뜻이기도 합니다. "값이 없는 경우를 잊는" null 버그를 컴파일러가 구조적으로 막는 장치입니다.

정수처럼 가능한 값이 많은 타입도 마찬가지입니다. `u8`(0~255)을 `1`과 `6` 두 갈래로만 `match`하면, 컴파일러가 빠진 범위를 계산해 알려 줍니다. [`04_catch_all.rs`](examples/04_catch_all.rs)의 `move_piece`에서 `other` 갈래를 지우면 직접 볼 수 있습니다.

```
error[E0004]: non-exhaustive patterns: `0_u8`, `2_u8..=5_u8` and `7_u8..=u8::MAX` not covered
...
10 |     match roll {
   |           ^^^^ patterns `0_u8`, `2_u8..=5_u8` and `7_u8..=u8::MAX` not covered
```

#### 나머지를 한꺼번에 받는 포괄 패턴

모든 값을 일일이 적을 수 없을 때는 **포괄 패턴(catch-all pattern)** 으로 나머지를 받습니다.

```rust
match roll {
    1 => println!("출발점으로 돌아갑니다"),
    6 => println!("6칸 가고 한 번 더 던집니다"),
    other => println!("{other}칸 앞으로 갑니다"), // 나머지 값을 other에 묶음
}
```

| 패턴 | 맞는 값 | 값을 묶는가 | 쓰는 때 |
| --- | --- | --- | --- |
| `other` (변수 이름) | 모든 값 | 묶음 | 나머지 값을 갈래에서 써야 할 때 |
| `_` (와일드카드) | 모든 값 | 묶지 않음 | 나머지 값이 필요 없을 때 |
| `_ => ()` | 모든 값 | 묶지 않음 | 나머지 경우에 아무것도 하지 않을 때 |

`()`는 ch3에서 본 유닛, 즉 "의미 있는 값이 없음"입니다. `_ => ()`라고 쓰면 "다른 값은 일부러 무시한다"는 뜻이 코드에 드러나고, 완전성도 지켜집니다. `_`는 값을 묶지 않으므로, 변수를 만들고 쓰지 않을 때 나는 "unused variable" 경고도 나지 않습니다.

포괄 패턴은 모든 값과 맞으므로 **반드시 마지막 갈래**에 둡니다. 갈래는 위에서부터 맞춰 보기 때문에, 포괄 패턴 아래의 갈래는 절대 실행되지 않습니다. `move_piece`에서 `6` 갈래를 `other` 갈래 아래로 옮기면 컴파일러가 이렇게 경고합니다.

```
warning: unreachable pattern
...
14 |         other => println!("    {other}: {other}칸 앞으로 갑니다"),
   |         ----- matches any value
15 |         6 => println!("    {roll}: 6칸 가고 한 번 더 던집니다"),
   |         ^ no value can reach this
```

#### 열거형에서 `_`를 쓸 때 주의할 점

정수는 값이 너무 많아 포괄 패턴이 꼭 필요하지만, 열거형은 배리언트가 몇 개 되지 않습니다. 그리고 열거형에 `_`를 쓰면 완전성 검사의 장점 하나를 잃습니다.

```rust
fn price_with_wildcard(ticket: &Ticket) -> u32 {
    match ticket {
        Ticket::Adult => 15000,
        _ => 10000, // Teen, Child, Senior... 그리고 앞으로 추가될 배리언트까지
    }
}
```

나중에 `Ticket`에 `Infant`(영유아)를 추가하면, 이 함수는 아무 경고 없이 영유아에게 10000원을 받습니다. 배리언트를 모두 적은 `match`라면 E0004가 나서 고칠 곳을 컴파일러가 하나하나 짚어 줍니다. 배리언트마다 동작이 달라야 하는 곳에서는 `_` 대신 배리언트를 모두 적는 편이 안전합니다. [`04_catch_all.rs`](examples/04_catch_all.rs)의 4번 구역과 [`compile_fail/e0004_missing_match_arm.rs`](compile_fail/e0004_missing_match_arm.rs)가 이 상황을 보여 줍니다.

### 2.9 `match`와 소유권

패턴의 바인딩도 대입과 같은 소유권 규칙(ch4)을 따릅니다. 무엇을 `match`하느냐에 따라 바인딩된 변수가 값을 **가져가거나**, **빌리거나**, **복사**합니다.

```rust
let status = OrderStatus::Cancelled(String::from("배송 지연"));
match status {
    OrderStatus::Cancelled(reason) => println!("취소 사유: {reason}"), // String이 reason으로 이동
    other => println!("다른 상태: {other:?}"),                         // 값 전체가 other로 이동
}
// println!("{status:?}"); // E0382: borrow of moved value: `status`
```

`reason`은 `String`의 새 소유자가 되고, 갈래가 끝날 때 drop됩니다. 어느 갈래로 가든 `status`의 일부(안의 `String`)나 전부가 이동하므로 `match` 뒤에는 `status`를 쓸 수 없습니다. 이동한 것이 값의 일부뿐이면 메시지가 `borrow of partially moved value`로 바뀝니다(4.3절). 원본을 계속 써야 한다면 **참조를 `match`** 합니다.

```rust
match &status {
    OrderStatus::Shipped { courier, tracking } => println!("{courier} {tracking}"), // &String, &u32
    OrderStatus::Received => println!("접수됨"),
    OrderStatus::Cancelled(reason) => println!("취소됨: {reason}"),                // &String
}
println!("{status:?}"); // 아무것도 이동하지 않았으므로 쓸 수 있습니다.
```

| `match`하는 것 | 바인딩되는 것 | `match` 뒤의 원본 |
| --- | --- | --- |
| 값 (`match status`) | `Copy`가 아닌 데이터는 **이동**, `Copy` 데이터는 **복사** | 이동된 부분이 있으면 쓸 수 없음 |
| 불변 참조 (`match &status`) | 불변 참조 (`&String`, `&u32`) | 그대로 쓸 수 있음 |
| 가변 참조 (`match &mut status`) | 가변 참조 (`&mut String`, `&mut u32`) | 그대로 쓸 수 있고, 바인딩을 통해 안의 데이터를 고칠 수 있음 |

알아 둘 점이 몇 가지 더 있습니다.

- **`_`는 값을 묶지 않으므로 이동도 일어나지 않습니다.** `match status { OrderStatus::Cancelled(_) => ..., ... }` 뒤에는 `status`를 그대로 쓸 수 있습니다. 반대로 이름 있는 포괄 패턴(`other`)은 값 전체를 묶으므로, 그 갈래로 가면 값 전체가 이동합니다.
- **바인딩되는 데이터가 `Copy` 타입이면** 값을 `match`해도 복사본이 묶일 뿐이라 원본이 남습니다. `Option<i32>`를 `Some(n)`으로 `match`한 뒤에도 원래 변수를 쓸 수 있는 이유입니다.
- **값을 옮기는 `match`도 쓸모가 있습니다.** 원본이 더 필요 없고 안의 데이터만 가져오고 싶다면, 값을 받아 `match`하는 것이 `clone()` 없이 데이터를 꺼내는 방법입니다. [`05_match_ownership.rs`](examples/05_match_ownership.rs)의 `into_cancel_reason`이 그 예입니다.
- **`&self` 메서드의 `match self`는 참조를 `match`하는 것입니다.** 2.6절에서 `shots`가 `&u32`였던 이유가 이것이고, `&mut self` 메서드라면 바인딩은 가변 참조가 됩니다.

참조를 `OrderStatus::Shipped { ... }`처럼 참조가 아닌 패턴과 맞춰도 되는 것은, 컴파일러가 참조를 따라가 맞춰 본 뒤 바인딩을 참조로 만들어 주기 때문입니다. 이 규칙을 기본 바인딩 모드(default binding mode)라고 하며, 패턴 문법 전체와 함께 [ch18](../ch18-patterns/README.md)에서 다룹니다. E0382 메시지의 `help:`가 제안하는 `Some(ref n)`의 `ref`는 "이 바인딩은 값을 옮기지 말고 참조로 빌려라"라는 키워드로, `match &name`과 같은 효과를 냅니다.

### 2.10 `if let`: 한 가지 패턴만 관심 있을 때

설정값이 있을 때만 무언가를 하고 싶다면, `match`로는 쓸모없는 `_ => ()` 갈래를 덧붙여야 합니다.

```rust
match max_connections {
    Some(max) => println!("최대 접속 수: {max}"),
    _ => (),
}
```

`if let`은 이것을 줄여 쓰는 문법입니다. `let`처럼 **패턴을 왼쪽, 값을 오른쪽**에 씁니다. 값이 패턴과 맞으면 블록을 실행하고(바인딩도 그 블록 안에서 쓸 수 있습니다), 맞지 않으면 건너뜁니다.

```rust
if let Some(max) = max_connections {
    println!("최대 접속 수: {max}");
}
```

- `else`를 붙이면 `match`의 `_` 갈래 역할을 합니다. `else if let`으로 다른 패턴을 이어서 검사할 수도 있습니다.
- `if let`도 표현식이라 값을 만들 수 있습니다. 이때는 `else`가 꼭 있어야 합니다. `else`가 없으면 패턴이 맞지 않을 때의 값이 `()`가 되어 E0317(`` `if` may be missing an `else` clause ``)이 납니다.

```rust
fn greeting(nickname: Option<&str>) -> String {
    if let Some(name) = nickname {
        format!("안녕하세요, {name}님!")
    } else {
        String::from("안녕하세요, 손님!")
    }
}
```

**무엇을 잃는가**: `if let`은 짧은 대신 **완전성 검사를 하지 않습니다.** `else`를 빠뜨리거나 `else if let` 사슬에서 배리언트 하나를 잊어도 컴파일러는 알려 주지 않습니다. 정말 한 경우만 관심 있을 때 `if let`을 쓰고, 여러 경우를 각각 다르게 처리해야 한다면 `match`를 씁니다. Rust 코드 검사 도구인 Clippy도 갈래가 하나뿐인 `match ... _ => ()`를 보면 `if let`을 쓰라고 제안합니다(`clippy::single_match`). 예제는 [`06_if_let.rs`](examples/06_if_let.rs)에 있습니다.

### 2.11 `let...else`: 행복한 경로 지키기

#### 들여쓰기가 깊어지는 문제

설정 파일의 한 줄 `"port=8080"`에서 포트 번호를 읽으려면 세 가지를 확인해야 합니다. `=`가 있는지, 키가 `port`인지, 값이 숫자인지. `if let`을 겹쳐 쓰면 확인할 것이 늘어날 때마다 들여쓰기가 깊어지고, 실패했을 때의 처리는 조건에서 멀리 떨어진 아래쪽 `else`로 밀려납니다.

```rust
fn apply_nested(line: &str) {
    if let Some((key, value)) = line.split_once('=') {
        if key.trim() == "port" {
            if let Ok(port) = value.trim().parse::<u16>() {
                println!("포트를 {port}번으로 설정합니다");
            } else {
                println!("[건너뜀] 포트 번호가 아닙니다: {value}");
            }
        } else {
            println!("[건너뜀] 모르는 설정입니다: {key}");
        }
    } else {
        println!("[건너뜀] '='가 없습니다: {line}");
    }
}
```

- `split_once('=')`는 처음 나오는 `=`의 앞뒤를 `(앞, 뒤)` 튜플로 나눠 `Some`에 담아 돌려주고, `=`가 없으면 `None`을 돌려줍니다. 그래서 패턴 `Some((key, value))`로 튜플까지 한 번에 풀 수 있습니다.
- `parse::<u16>()`의 `::<u16>`은 "u16으로 바꿔라"라고 타입을 직접 알려 주는 문법입니다. ch2에서는 `let guess: u32`처럼 변수 쪽에 타입을 적었습니다. `parse`는 성공하면 `Ok(값)`, 실패하면 `Err(에러)`를 돌려주는 `Result`이고, `Ok(port)`도 같은 방식으로 패턴에 쓸 수 있습니다(`Result`는 [ch9](../ch9-error-handling/README.md)에서 다룹니다).

#### `let...else`로 펴기

`let...else`는 `let`처럼 생겼지만 오른쪽 값이 패턴과 **맞지 않을 수도 있는** 경우에 씁니다.

```rust
fn apply(line: &str) {
    let Some((key, value)) = line.split_once('=') else {
        println!("[건너뜀] '='가 없습니다: {line}");
        return;
    };

    if key.trim() != "port" {
        println!("[건너뜀] 모르는 설정입니다: {key}");
        return;
    }

    let Ok(port) = value.trim().parse::<u16>() else {
        println!("[건너뜀] 포트 번호가 아닙니다: {value}");
        return;
    };

    println!("포트를 {port}번으로 설정합니다");
}
```

- 패턴이 맞으면 바인딩(`key`, `value`, `port`)이 **바깥 스코프에 남아** 함수 끝까지 쓸 수 있습니다. `if let`의 바인딩이 블록 안에서만 유효한 것과 다른 점입니다.
- 패턴이 맞지 않으면 `else` 블록을 실행합니다. 이 블록은 반드시 **흐름을 끊어야(diverge)** 합니다. 즉 `return`, `continue`, `break`, `panic!`처럼 다음 줄로 이어지지 않는 코드로 끝나야 합니다. 바인딩에 넣을 값이 없는 채로 다음 줄로 넘어갈 수는 없기 때문입니다. [`07_let_else.rs`](examples/07_let_else.rs)의 `apply`에서 첫 번째 `else`의 `return;`을 지우면 다음 에러가 납니다.

```
error[E0308]: `else` clause of `let...else` does not diverge
...
34 |       let Some((key, value)) = line.split_once('=') else {
   |  ________________________________________________________^
35 | |         println!("    [건너뜀] '='가 없습니다: {line}");
36 | |     };
   | |_____^ expected `!`, found `()`
...
   = help: try adding a diverging expression, such as `return` or `panic!(..)`
```

`` expected `!` ``의 `!`는 "값을 만들지 않고 흐름이 끝난다"는 것을 나타내는 타입(never 타입, [ch19](../ch19-advanced-features/README.md))입니다. `else` 블록이 `println!`으로 끝나 `()`를 내므로, 흐름이 끊기지 않는다는 뜻입니다.

실패하는 경우를 그 자리에서 처리하고 빠져나가므로, 확인을 통과한 뒤의 코드, 즉 값이 제대로 있는 정상 흐름이 들여쓰기 없이 위에서 아래로 이어집니다. 이 정상 흐름을 **행복한 경로(happy path)** 라고 부릅니다. ch3에서 배운 조기 반환을 패턴과 결합한 것이라고 볼 수 있고, Swift의 `guard let ... else { return }`과 같은 발상입니다.

#### 반박 가능한 패턴과 반박 불가능한 패턴

그냥 `let Some(port) = find_port(...);`라고 쓰면 E0005로 컴파일되지 않습니다(4.4절).

- 일반 `let`의 패턴은 **어떤 값이 와도 반드시 맞아야** 합니다. ch3의 `let (x, y) = (1, 2);`처럼 튜플을 푸는 패턴은 실패할 일이 없습니다. 이런 패턴을 **반박 불가능한 패턴(irrefutable pattern)** 이라고 합니다.
- `Some(port)`는 값이 `None`이면 맞지 않습니다. 이런 패턴을 **반박 가능한 패턴(refutable pattern)** 이라고 합니다. 맞지 않을 때 `port`에 무엇을 넣어야 할지 정해지지 않으므로, 일반 `let`에는 쓸 수 없습니다.

`match`, `if let`, `let...else`는 "맞지 않을 때 할 일"을 정할 수 있으므로 반박 가능한 패턴을 받습니다. 셋 중 무엇을 쓸지는 다음 표를 기준으로 고릅니다. 전체 코드는 [`07_let_else.rs`](examples/07_let_else.rs)에 있습니다.

| 문법 | 완전성 검사 | 바인딩이 유효한 곳 | 어울리는 상황 |
| --- | --- | --- | --- |
| `match` | 함 | 각 갈래 안 | 여러 경우를 각각 다르게 처리할 때 |
| `if let` (`else`) | 하지 않음 | `if` 블록 안 | 한 경우만 관심 있고, 나머지는 무시하거나 간단히 처리할 때 |
| `let...else` | 하지 않음 | `let` 아래 스코프 전체 | 값이 없으면 빠져나가고, 있으면 그 값으로 계속 진행할 때 |

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_enum_variants.rs`](examples/01_enum_variants.rs) | 배리언트의 세 가지 모양, 구조체 하나로 표현했을 때와 비교 | 1 |
| [`02_match_binding.rs`](examples/02_match_binding.rs) | `match`와 바인딩, 표현식으로서의 `match`, 열거형의 연관 함수와 메서드 | 1, 2 |
| [`03_option.rs`](examples/03_option.rs) | `Some`/`None`, `Option`을 돌려주는 함수, `match`·`unwrap_or`로 값 꺼내기 | 3 |
| [`04_catch_all.rs`](examples/04_catch_all.rs) | 이름 있는 포괄 패턴, `_`, `_ => ()`, 열거형에서 `_`를 쓸 때와 모두 적을 때 | 2 |
| [`05_match_ownership.rs`](examples/05_match_ownership.rs) | 값을 옮기는 `match`, `&`/`&mut`로 매칭, `Copy` 바인딩, `_`와 이동 | 4 |
| [`06_if_let.rs`](examples/06_if_let.rs) | `match`와 `if let` 비교, `else`, `else if let` | 5 |
| [`07_let_else.rs`](examples/07_let_else.rs) | 겹친 `if let`과 `let...else` 비교, `else`에서 `return`/`continue` | 5 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 4개 (4절) | 2~5 |

### 01_enum_variants — 데이터를 가진 열거형

1·2번 구역은 구조체 하나로 상태를 표현할 때 생기는 빈 필드와 "말이 안 되는 조합"을, 3·4번 구역은 같은 내용을 열거형으로 표현한 모습을 보여 줍니다. 이 예제는 값을 `{:?}`로 출력하기만 하고 필드를 읽지 않습니다. `derive(Debug)`가 만든 출력 코드는 "필드를 읽었다"로 치지 않아 dead_code 경고가 나므로, 파일 맨 위에 `#![expect(dead_code)]`로 의도한 경고임을 표시했습니다.

```sh
cargo run -p p0-ch6-enums-pattern-matching --example 01_enum_variants
```

```
1. 구조체 하나로 표현하면 쓰지 않는 필드도 채워야 합니다
    OrderRecord { kind: Received, courier: "", tracking: 0, place: "", reason: "" }
2. 구조체로는 말이 안 되는 조합도 만들 수 있습니다
    OrderRecord { kind: Cancelled, courier: "번개택배", tracking: 1234, place: "", reason: "" }
3. 열거형은 배리언트마다 필요한 데이터만 담습니다
    Received
    Shipped { courier: "번개택배", tracking: 1234 }
    Delivered("문 앞")
    Cancelled("단순 변심")
4. 모양이 달라도 타입은 하나이므로 한 함수로 받고, 한 배열에 담을 수 있습니다
    [로그] Shipped { courier: "번개택배", tracking: 1234 }
    [로그] Cancelled("단순 변심")
    배열 길이: 4
```

### 02_match_binding — `match`와 바인딩, 열거형의 메서드

`price` 함수는 음료 값을 받아 배리언트마다 가격을 계산합니다. `Drink::americano()`는 열거형의 연관 함수, `describe`는 `match self`를 쓰는 메서드입니다. 4번 구역에서는 `describe`가 `drink`를 빌리기만 하고 `price`가 소유권을 가져가므로, 호출 순서를 바꾸면 E0382가 납니다.

```sh
cargo run -p p0-ch6-enums-pattern-matching --example 02_match_binding
```

```
1. 배리언트마다 다른 갈래가 실행되고, 그 갈래의 값이 match의 값이 됩니다
    물: 0원
    주스 350ml: 3500원
2. 패턴에 쓴 이름에 배리언트 안의 값이 묶입니다
    따뜻한 커피 2샷: 3000원
    아이스 커피 3샷: 4000원
3. 열거형에도 연관 함수와 메서드를 정의할 수 있습니다
    Coffee { shots: 2, iced: false } → 따뜻한 커피 2샷
4. 메뉴판: 메서드로 이름을 만들고(빌림), 함수로 가격을 계산합니다(이동)
    물: 0원
    아이스 커피 1샷: 3000원
    주스 500ml: 5000원
```

### 03_option — `Option<T>`

`Some`과 `None`으로 값을 만들고, 표준 라이브러리와 직접 만든 함수가 `Option`을 돌려주는 모습, `match`와 `unwrap_or`로 값을 꺼내는 방법을 보여 줍니다. `plus_one`은 Clippy가 `x.map(|i| i + 1)`로 줄여 쓰라고 제안하는 코드지만(`clippy::manual_map`), `map`과 클로저는 뒤에서 배우므로 `#[expect(clippy::manual_map)]`을 달고 `match`로 두었습니다.

```sh
cargo run -p p0-ch6-enums-pattern-matching --example 03_option
```

```
1. Some(값)으로 값을 담고, None으로 값이 없음을 나타냅니다
    Some(5), Some('가'), None
2. 표준 라이브러리도 결과가 없을 수 있으면 Option을 돌려줍니다
    "hello".find('l') = Some(2)
    "hello".find('z') = None
    10.checked_div(3) = Some(3)
    10.checked_div(0) = None
3. 직접 만든 함수도 Option으로 "없을 수 있음"을 알립니다
    [1, 3, 4, 6] → Some(4)
    [1, 3, 5]    → None
    []           → None
4. Option<i32>는 i32가 아니므로, 꺼낸 뒤에야 더할 수 있습니다
    sum = 10
5. Option을 받아 Option을 돌려주는 함수
    plus_one(Some(5)) = Some(6)
    plus_one(None)    = None
6. 자주 쓰는 메서드: unwrap_or, is_some, is_none
    unwrap_or(0): 8, 0
    is_some: true, is_none: true
```

### 04_catch_all — 완전성 검사와 포괄 패턴

주사위 값(`u8`)에 대해 이름 있는 포괄 패턴, `_`, `_ => ()`를 차례로 씁니다. 3번 구역에서 `draw_card(3)`은 아무것도 출력하지 않습니다. 4번 구역은 같은 `Ticket`에 대해 `_`를 쓴 함수와 배리언트를 모두 적은 함수의 결과를 나란히 출력합니다.

```sh
cargo run -p p0-ch6-enums-pattern-matching --example 04_catch_all
```

```
1. 이름 있는 포괄 패턴(other)은 나머지 값을 받아 이름에 묶습니다
    1: 출발점으로 돌아갑니다
    4: 4칸 앞으로 갑니다
    6: 6칸 가고 한 번 더 던집니다
2. _ 패턴은 나머지 값을 받지만 묶지는 않습니다
    1 → 아쉬워요!, 3 → 좋아요., 6 → 최고예요!
3. _ => ()는 "나머지는 아무것도 하지 않는다"는 뜻입니다
    1: 벌칙 카드를 뽑습니다
    6: 보너스 카드를 뽑습니다
4. 열거형에서 _를 쓸지, 배리언트를 모두 적을지
    Adult: _ 사용 15000원, 모두 적음 15000원
    Teen: _ 사용 10000원, 모두 적음 12000원
    Child: _ 사용 10000원, 모두 적음 8000원
    Senior: _ 사용 10000원, 모두 적음 10000원
```

### 05_match_ownership — `match`와 소유권

값을 `match`해 `String`을 옮기는 경우, `&`와 `&mut`로 매칭해 원본을 남기거나 고치는 경우, `Copy` 데이터와 `_`가 이동을 일으키지 않는 경우를 차례로 보여 줍니다. 1번 구역 아래에 주석 처리된 `println!("{status:?}");`의 주석을 풀면 E0382를 직접 볼 수 있습니다. 이때 컴파일러는 값 전체를 옮기는 `other` 갈래를 짚어 `` borrow of moved value: `status` ``라고 알려 주고, `help:`로 `ref other`를 제안합니다.

```sh
cargo run -p p0-ch6-enums-pattern-matching --example 05_match_ownership
```

```
1. 값을 match하면 String은 바인딩된 변수로 이동합니다
    취소 사유: 배송 지연
2. 소유권을 넘겨 데이터를 꺼내 오는 함수
    Some("단순 변심"), None
3. 참조를 match하면 바인딩도 참조가 되어 원본을 계속 쓸 수 있습니다
    번개택배 운송장 1234번
    match 뒤에도 원본 사용 가능: Shipped { courier: "번개택배", tracking: 1234 }
4. 가변 참조를 match하면 바인딩으로 원본 안의 데이터를 고칠 수 있습니다
    Shipped { courier: "번개택배(확인)", tracking: 1235 }
    Cancelled("주소 오류(확인)")
5. 묶은 값이 Copy 타입이면 이동이 아니라 복사입니다
    n = 3
    count는 그대로: Some(3)
6. _는 값을 묶지 않으므로 이동도 일어나지 않습니다
    취소된 주문입니다
    match 뒤에도 원본 사용 가능: Cancelled("품절")
```

### 06_if_let — `if let`과 `else`

1번 구역의 `match`는 2번 구역의 `if let`과 비교하려고 일부러 남긴 코드라서, Clippy의 `single_match` 제안을 `#[expect(clippy::single_match)]`로 표시했습니다. 4번 구역은 배열을 `&orders`로 순회하므로 `if let`의 바인딩도 참조입니다(2.9절).

```sh
cargo run -p p0-ch6-enums-pattern-matching --example 06_if_let
```

```
1. match로 한 패턴만 처리하면 _ => ()가 덧붙습니다
    최대 접속 수: 100
2. if let으로 쓰면 같은 일을 더 짧게 쓸 수 있습니다
    최대 접속 수: 100
3. if let ... else: 패턴이 맞지 않을 때 할 일을 정합니다
    안녕하세요, 러스타님!
    안녕하세요, 손님!
4. 열거형에서 한 배리언트만 골라 처리하고, 나머지는 else에서 셉니다
    배송 중: 번개택배 운송장 1234번
    배송 중: 새벽택배 운송장 5678번
    배송 중이 아닌 주문: 2건
5. else if let으로 다른 패턴을 이어서 검사할 수 있습니다
    접수
    그 밖의 상태: Shipped { courier: "번개택배", tracking: 1234 }
    취소: 단순 변심
    그 밖의 상태: Shipped { courier: "새벽택배", tracking: 5678 }
```

### 07_let_else — `let...else`와 행복한 경로

같은 설정 줄 다섯 개를 겹친 `if let` 버전(`apply_nested`)과 `let...else` 버전(`apply`)으로 처리해 결과가 같다는 것을 확인합니다. 두 함수의 코드 모양을 나란히 놓고 비교해 보세요. 3번 구역은 반복문 안에서 `else`에 `continue`를 써서 잘못된 입력을 건너뜁니다. `70000`은 `u16`의 범위(0~65535)를 넘으므로 `parse`가 실패합니다.

```sh
cargo run -p p0-ch6-enums-pattern-matching --example 07_let_else
```

```
1. if let을 겹쳐 쓴 버전
    포트를 8080번으로 설정합니다
    포트를 443번으로 설정합니다
    [건너뜀] 모르는 설정입니다: host
    [건너뜀] 포트 번호가 아닙니다: abc
    [건너뜀] '='가 없습니다: # 주석
2. let...else 버전 (결과는 같습니다)
    포트를 8080번으로 설정합니다
    포트를 443번으로 설정합니다
    [건너뜀] 모르는 설정입니다: host
    [건너뜀] 포트 번호가 아닙니다: abc
    [건너뜀] '='가 없습니다: # 주석
3. 반복문 안에서는 else에서 continue로 다음 차례로 넘어갈 수 있습니다
    건너뜀: 스물
    건너뜀: 70000
    합계: 45
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch6-enums-pattern-matching
rustc --edition 2024 compile_fail/e0004_missing_match_arm.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 코드의 자세한 설명은 `rustc --explain E0004`처럼 확인할 수 있습니다.

### 4.1 `match`에서 배리언트를 빠뜨림 — E0004

파일: [`compile_fail/e0004_missing_match_arm.rs`](compile_fail/e0004_missing_match_arm.rs)

`Ticket` 열거형에 `Infant` 배리언트를 새로 추가했지만, 가격을 정하는 `match`에 갈래를 추가하는 것을 잊은 상황입니다.

```
error[E0004]: non-exhaustive patterns: `Ticket::Infant` not covered
  --> compile_fail/e0004_missing_match_arm.rs:17:11
   |
17 |     match ticket {
   |           ^^^^^^ pattern `Ticket::Infant` not covered
   |
note: `Ticket` defined here
  --> compile_fail/e0004_missing_match_arm.rs:8:6
   |
 8 | enum Ticket {
   |      ^^^^^^
...
13 |     Infant, // 새로 추가한 배리언트
   |     ------ not covered
   = note: the matched value is of type `Ticket`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
21 ~         Ticket::Senior => 10000,
22 ~         Ticket::Infant => todo!(),
   |
```

- **읽는 법**: 첫 줄이 빠진 패턴(`Ticket::Infant`)을 알려 주고, `note:`가 그 배리언트가 정의된 위치를 `not covered`로 짚어 줍니다. 열거형 정의와 `match`가 서로 다른 파일에 있어도 양쪽을 모두 보여 줍니다.
- **원인**: `match`는 가능한 모든 값을 다뤄야 하는데 `Infant`를 처리하는 갈래가 없습니다.
- **해결**: `Ticket::Infant => 0,`처럼 빠진 배리언트의 갈래를 추가합니다. `help:`가 제안하는 `todo!()`는 "아직 구현하지 않음"이라는 패닉을 일으키는 매크로로, 컴파일러가 자리만 채워 준 것이므로 실제 값으로 바꿔야 합니다. 여기서 `_ => 0`을 추가해도 컴파일은 되지만, 2.8절에서 본 것처럼 다음에 배리언트가 늘어날 때 이 도움을 받을 수 없게 됩니다.

### 4.2 `Option<T>`를 `T`처럼 사용 — E0277

파일: [`compile_fail/e0277_add_option_to_int.rs`](compile_fail/e0277_add_option_to_int.rs)

```
error[E0277]: cannot add `Option<i8>` to `i8`
  --> compile_fail/e0277_add_option_to_int.rs:12:17
   |
12 |     let sum = x + y;
   |                 ^ no implementation for `i8 + Option<i8>`
   |
   = help: the trait `Add<Option<i8>>` is not implemented for `i8`
help: the following other types implement trait `Add<Rhs>`
...
   = note: `i8` implements `Add`
...
   = note: `&i8` implements `Add<i8>`
...
```

- **읽는 법**: 첫 줄의 `` cannot add `Option<i8>` to `i8` ``이 핵심입니다. 그 아래 `help:`와 `note:`에는 표준 라이브러리 내부 경로가 섞인 긴 목록이 이어지는데(위에서는 `...`로 줄였습니다), 요약하면 "`i8`에 더할 수 있는 것은 `i8`과 그 참조뿐"이라는 뜻입니다. `+` 연산자는 `Add`라는 트레이트로 정의되는데, 트레이트는 ch10에서 배우고 지금은 "타입 조합마다 연산이 따로 정의되어 있다"고 보면 됩니다.
- **원인**: `y`는 `None`일 수도 있는 `Option<i8>`입니다. `None`일 때 어떻게 할지 정하지 않았으므로 더할 수 없습니다.
- **해결**: `None`일 때 할 일을 정해서 `i8`을 꺼냅니다. 기본값으로 충분하면 `let sum = x + y.unwrap_or(0);`, 경우마다 다르게 처리해야 하면 `match`를 씁니다([`03_option.rs`](examples/03_option.rs)의 4번 구역).

### 4.3 `match`로 값을 옮긴 뒤 원본 사용 — E0382

파일: [`compile_fail/e0382_use_after_match_move.rs`](compile_fail/e0382_use_after_match_move.rs)

```
error[E0382]: borrow of partially moved value: `name`
  --> compile_fail/e0382_use_after_match_move.rs:15:16
   |
11 |         Some(n) => println!("이름: {n}"),
   |              - value partially moved here
...
15 |     println!("{name:?}");
   |                ^^^^ value borrowed here after partial move
   |
   = note: partial move occurs because value has type `String`, which does not implement the `Copy` trait
help: borrow this binding in the pattern to avoid moving the value
   |
11 |         Some(ref n) => println!("이름: {n}"),
   |              +++
```

- **읽는 법**: ch4에서 본 E0382와 같은 에러인데, "moved" 대신 "**partially** moved"라고 나옵니다. `name` 전체가 아니라 그 안의 `String`만 `n`으로 빠져나갔다는 뜻입니다. 일부가 빠져나간 값은 통째로 쓸 수 없습니다.
- **원인**: `match name`은 값을 `match`하므로, `Some(n)` 패턴이 `String`의 소유권을 `n`으로 옮깁니다.
- **해결**: `match` 뒤에도 원본이 필요하면 참조를 `match`합니다. `match &name { Some(n) => ..., None => ... }`으로 바꾸면 `n`은 `&String`이 되고 아무것도 이동하지 않습니다. `help:`가 제안하는 `Some(ref n)`도 같은 효과를 냅니다(2.9절). 원본이 더 필요 없다면 에러가 난 `println!`을 지우는 것이 맞는 해결일 수도 있습니다.

### 4.4 일반 `let`에 반박 가능한 패턴 사용 — E0005

파일: [`compile_fail/e0005_refutable_pattern_in_let.rs`](compile_fail/e0005_refutable_pattern_in_let.rs)

```
error[E0005]: refutable pattern in local binding
  --> compile_fail/e0005_refutable_pattern_in_let.rs:17:9
   |
17 |     let Some(port) = find_port("host=localhost");
   |         ^^^^^^^^^^ pattern `None` not covered
   |
   = note: `let` bindings require an "irrefutable pattern", like a `struct` or an `enum` with only one variant
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-02-refutability.html
   = note: the matched value is of type `Option<u16>`
help: you might want to use `let...else` to handle the variant that isn't matched
   |
17 |     let Some(port) = find_port("host=localhost") else { todo!() };
   |                                                  ++++++++++++++++
```

- **읽는 법**: "refutable pattern"은 반박 가능한 패턴, "irrefutable pattern"은 반박 불가능한 패턴입니다(2.11절). ``pattern `None` not covered``는 E0004와 같은 표현으로, 값이 `None`인 경우를 이 `let`이 처리하지 못한다는 뜻입니다. 링크된 The Book 19.2절은 이 저장소의 [ch18](../ch18-patterns/README.md)에 해당합니다.
- **원인**: 일반 `let`의 패턴은 어떤 값이 와도 맞아야 하는데, `Some(port)`는 `None`과 맞지 않습니다.
- **해결**: `help:`대로 `let...else`를 쓰고, `todo!()` 대신 실제로 할 일을 적습니다. 예를 들어 `else { println!("포트 설정이 없습니다"); return; }`처럼 씁니다. 값이 있을 때만 무언가를 하면 된다면 `if let Some(port) = ... { ... }`, 기본값으로 충분하다면 `let port = find_port(...).unwrap_or(80);`도 해결책입니다.

## 5. 연습 문제

### 연습 1. 계산 명령 열거형

다음 세 가지 계산 명령을 하나의 열거형 `Operation`으로 정의하세요.

- `Add`: 두 `i32`를 더함 (튜플형)
- `Divide`: `dividend`를 `divisor`로 나눔 (구조체형)
- `Negate`: `i32` 하나의 부호를 바꿈 (튜플형)

그리고 계산 결과를 돌려주는 메서드 `fn apply(&self) -> Option<i32>`를 구현하세요. 0으로 나누는 경우에는 `None`을 돌려줍니다. 여러 명령을 배열에 담아 순회하면서 결과를 출력해 보세요.

```
Add(2, 3) = 5
Divide { dividend: 10, divisor: 3 } = 3
Divide { dividend: 1, divisor: 0 } = 계산할 수 없음
Negate(7) = -7
```

<details>
<summary>힌트</summary>

`apply`는 `match self`로 배리언트를 나눕니다. `self`가 참조이므로 바인딩도 참조입니다(`a`, `b`는 `&i32`). `a + b`나 `-n` 같은 산술 연산은 참조에도 그대로 쓸 수 있지만, `*divisor == 0`처럼 비교할 때는 `*`로 값을 꺼내는 편이 명확합니다. 0으로 나누는 경우를 직접 검사하는 대신 [`03_option.rs`](examples/03_option.rs)에서 본 `checked_div`를 쓰면 `Option`이 바로 나옵니다. 출력은 `match op.apply() { Some(v) => ..., None => ... }`로 나누고, 열거형에 `#[derive(Debug)]`를 붙여 `{op:?}`로 명령을 출력합니다.

</details>

### 연습 2. 새 배리언트 추가하기

[`04_catch_all.rs`](examples/04_catch_all.rs)의 `Ticket`에 `Infant` 배리언트를 추가하고 컴파일해 보세요. 실행하기 **전에** 먼저 예측해 보세요. `price_with_wildcard`와 `price_exhaustive` 중 어느 함수에서 에러가 날까요? 그다음 영유아 가격을 0원으로 정해 에러를 고치고, `tickets` 배열에도 `Ticket::Infant`를 넣어 출력을 확인하세요.

<details>
<summary>힌트</summary>

에러는 `price_exhaustive`에서만 E0004로 납니다. `price_with_wildcard`는 `_` 갈래가 `Infant`도 받아 버리므로 아무 말 없이 컴파일됩니다. 두 함수가 `&Ticket`(참조)을 받아 `match`하므로 메시지에는 ``pattern `&Ticket::Infant` not covered``처럼 `&`가 붙어 나옵니다. `price_exhaustive`에 `Ticket::Infant => 0,`을 추가하면 고쳐지고, 출력의 `Infant` 줄에서 두 함수의 가격이 10000원과 0원으로 다르게 나옵니다. 이것이 열거형에서 `_`를 조심해야 하는 이유입니다.

</details>

### 연습 3. 소유권을 가져가지 않고 꺼내기

[`05_match_ownership.rs`](examples/05_match_ownership.rs)의 `into_cancel_reason`은 상태의 소유권을 가져갑니다. 원본을 그대로 두고 취소 사유만 빌려 오는 `fn cancel_reason(status: &OrderStatus) -> Option<&str>`을 작성하세요. 호출한 뒤에도 원래 상태 값을 출력할 수 있어야 합니다.

<details>
<summary>힌트</summary>

`status`가 참조이므로 `match status { OrderStatus::Cancelled(reason) => Some(reason), _ => None }`에서 `reason`은 `&String`입니다. 반환 타입이 `Option<&str>`이므로 `Some(reason)`의 `&String`은 `&str`로 자동 변환됩니다(ch4에서 본 역참조 강제 변환). 매개변수에 참조가 하나뿐이라 반환하는 참조가 그 매개변수에서 빌려 온 것이라고 컴파일러가 판단할 수 있으므로, ch4의 `first_word`처럼 라이프타임을 적지 않아도 됩니다.

</details>

### 연습 4. `let...else`로 성적 처리하기

`"이름:점수"` 형식의 문자열 배열에서 이름과 점수를 꺼내 출력하고, 올바른 줄의 점수 평균(정수 나눗셈)을 구하세요. `:`가 없거나 점수가 숫자가 아닌 줄은 안내를 출력하고 건너뜁니다. `let...else`를 두 번 쓰고, 행복한 경로에는 들여쓰기가 생기지 않게 작성하세요.

```rust
let records = ["김철수:85", "이영희:92", "박민수", "최지우:백"];
```

```
김철수: 85점
이영희: 92점
형식 오류(':' 없음): 박민수
점수가 숫자가 아님: 최지우
평균: 88
```

<details>
<summary>힌트</summary>

[`07_let_else.rs`](examples/07_let_else.rs)의 3번 구역과 같은 구조입니다. `for record in records` 안에서 `let Some((name, score_text)) = record.split_once(':') else { ...; continue; };`로 이름과 점수 문자열을 나누고, `let Ok(score) = score_text.parse::<u32>() else { ...; continue; };`로 숫자로 바꿉니다. 합계와 개수를 `let mut` 변수에 더해 두었다가 반복이 끝난 뒤 나눕니다. (85 + 92) / 2는 정수 나눗셈이라 88입니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [6. Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
  - [6.1 Defining an Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html): 열거형 정의, 배리언트의 데이터, `Option`과 null
  - [6.2 The match Control Flow Construct](https://doc.rust-lang.org/book/ch06-02-match.html): `match`, 바인딩, 완전성 검사, 포괄 패턴
  - [6.3 Concise Control Flow with if let and let...else](https://doc.rust-lang.org/book/ch06-03-if-let.html): `if let`, `let...else`
- Rust By Example
  - [Custom Types > Enums](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html): 세 가지 모양의 배리언트를 가진 `WebEvent`와 `match`. `to_owned()`는 `String::from`처럼 `&str`에서 `String`을 만드는 메서드입니다. 아래쪽의 타입 별칭(`type`)은 [ch19](../ch19-advanced-features/README.md)에서 다룹니다.
  - [Flow of Control > match](https://doc.rust-lang.org/rust-by-example/flow_control/match.html)와 그 하위 절 [Destructuring > enums](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_enum.html): 정수와 열거형에 대한 `match`, 배리언트 구조 분해
  - [Flow of Control > if let](https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html): `if let`과 `else`, `else if`. 예제의 `value @ 100` 같은 `@` 바인딩은 [ch18](../ch18-patterns/README.md)에서 다룹니다.
  - [Flow of Control > let-else](https://doc.rust-lang.org/rust-by-example/flow_control/let_else.html): `let...else`와 `else` 블록이 흐름을 끊어야 한다는 규칙
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`08_enums`](https://github.com/rust-lang/rustlings/tree/main/exercises/08_enums): 열거형 정의와 `match`. `enums3.rs`는 메시지 열거형의 배리언트마다 구조체의 메서드를 호출하도록 `match`를 작성하는 문제입니다.
  - [`12_options`](https://github.com/rust-lang/rustlings/tree/main/exercises/12_options): `Option` 다루기. `options3.rs`는 이 챕터 4.3절의 E0382와 같은 문제로, `match` 대상에 `&`를 붙이거나 `ref`를 쓰면 풀립니다. `options1.rs`의 테스트(`#[test]`, `assert_eq!`)는 [ch11](../ch11-testing/README.md), `options2.rs`의 `Vec`과 `while let`은 [ch8](../ch8-collections/README.md)와 [ch18](../ch18-patterns/README.md)에서 다루지만, `while let`은 "패턴이 맞는 동안 반복하는 `if let`"이라고 보면 이 챕터의 내용으로 풀 수 있습니다.
- 에러 설명: `rustc --explain E0004`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
