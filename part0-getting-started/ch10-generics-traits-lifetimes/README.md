# ch10. 제네릭, 트레이트, 라이프타임

이 챕터의 세 기능은 모두 "코드 하나를 여러 상황에 쓰되, 잘못 쓰면 컴파일러가 막게 하는" 도구입니다.

- **제네릭(generic)**: 타입을 매개변수로 받아, 타입만 다른 코드를 하나로 합칩니다.
- **트레이트(trait)**: 타입이 가져야 할 능력을 메서드 목록으로 정의하고, 제네릭 코드가 "이 능력이 있는 타입만 받는다"고 요구할 수 있게 합니다.
- **라이프타임(lifetime)**: 참조들이 서로 어떤 관계로 살아 있어야 하는지 적어, 함수와 구조체를 오가는 참조도 댕글링 참조가 되지 않게 합니다.

세 가지 모두 이미 만나 봤습니다. `Option<T>`(ch6), `Vec<T>`와 `HashMap<K, V>`(ch8), `Result<T, E>`(ch9)의 `<>`가 제네릭이고, `{}`와 `{:?}`([ch1](../ch1-hello-cargo/README.md)), `#[derive(Debug)]`([ch5](../ch5-structs/README.md)), `Copy`([ch4](../ch4-ownership/README.md)), `rand`의 `RngExt`([ch2](../ch2-guessing-game/README.md))가 트레이트입니다. ch4에서 함수 안 값의 참조를 반환할 때, ch5에서 구조체 필드에 `&str`을 넣을 때 E0106으로 막혔던 것은 라이프타임 때문이었습니다. 이 챕터에서 이 조각들을 이어 붙입니다.

- 패키지: `p0-ch10-generics-traits-lifetimes`
- 기준 문서: [The Book 10장 Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html)
- 확인한 툴체인: rustc 1.100.0, Edition 2024. 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 타입만 다른 중복 함수를 제네릭 함수로 합치고, 제네릭 구조체·열거형·메서드(`impl<T>`, 특정 타입 전용 `impl Point<f32>`)를 정의할 수 있다. 단형성화 때문에 제네릭에 실행 비용이 없다는 것을 설명할 수 있다.
2. 트레이트를 정의하고 타입에 구현할 수 있다(기본 구현과 재정의 포함). `Display`를 직접 구현하고 `derive`가 하는 일을 설명할 수 있으며, 트레이트를 `use`로 가져와야 하는 이유와 고아 규칙이 막는 문제를 설명할 수 있다.
3. `impl Trait`, `T: Trait`, `+`, `where`로 트레이트 바운드를 적어 제네릭 함수가 요구하는 능력을 표현할 수 있다. 반환 위치 `impl Trait`의 한계, 바운드를 이용한 조건부 구현과 포괄 구현을 설명할 수 있다.
4. 라이프타임 표기가 참조들 사이의 관계를 설명할 뿐 값의 수명을 바꾸지 않는다는 것을 설명하고, `longest`처럼 함수 시그니처에 라이프타임을 적을 수 있다. E0106, E0597, E0515를 읽고 고칠 수 있다.
5. 참조 필드를 가진 구조체와 그 메서드에 라이프타임을 적을 수 있고, 생략 규칙 3가지로 표기가 필요한지 판단할 수 있으며, `'static`이 맞는 경우와 에러를 덮는 잘못된 경우를 구분할 수 있다.

## 2. 핵심 개념

2.1~2.4절은 제네릭, 2.5~2.11절은 트레이트, 2.12~2.17절은 라이프타임을 다루고, 2.18절에서 세 가지를 한 함수에 함께 씁니다.

### 2.1 중복 제거에서 제네릭 함수까지

목록에서 가장 큰 값을 찾는 코드가 여러 곳에 필요하다고 해 봅시다. The Book은 이 문제를 세 단계로 다듬습니다.

| 단계 | 코드 | 남은 문제 |
| --- | --- | --- |
| 1. 반복문 복사 | 목록마다 같은 `for` 반복문 | 고칠 때 모든 사본을 똑같이 고쳐야 합니다. |
| 2. 함수 추출 | `fn largest_i32(list: &[i32]) -> &i32` | `char` 목록에는 본문이 똑같은 `largest_char`를 또 만들어야 합니다. |
| 3. 제네릭 함수 | `fn largest<T: PartialOrd>(list: &[T]) -> &T` | 비교할 수 있는 타입이면 함수 하나로 처리합니다. |

함수 추출이 **값**을 매개변수로 바꿔 중복을 없앴다면, 제네릭은 **타입**을 매개변수로 바꿔 중복을 없앱니다.

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}
```

- 함수 이름 뒤 `<>` 안에 **타입 매개변수(type parameter)** `T`를 선언합니다. "`largest`는 어떤 타입 `T`에 대해 제네릭이다"라고 읽습니다. 이름은 무엇이든 되지만 짧은 UpperCamelCase, 특히 `T`(type)를 관례로 씁니다.
- 선언한 `T`는 `&[T]`, `&T`처럼 타입이 올 자리에 씁니다.
- 호출할 때는 `largest(&number_list)`처럼 인자만 넘깁니다. 컴파일러가 인자의 타입을 보고 `T`를 `i32`로 정합니다.

#### 왜 `T: PartialOrd`가 필요한가

`<T>`만 쓰면 E0369로 거부됩니다(4.1절). 제네릭 함수의 본문은 **`T`에 올 수 있는 모든 타입**에 대해 올바라야 하는데, 직접 만든 구조체처럼 `>`로 비교할 수 없는 타입도 `T`가 될 수 있기 때문입니다. `T: PartialOrd`는 "`T`는 비교 연산을 제공하는 `PartialOrd` 트레이트를 구현한 타입이어야 한다"는 조건이고, 이런 조건을 **트레이트 바운드(trait bound)** 라고 부릅니다(2.9절). `i32`, `f64`, `char`, `&str`은 모두 `PartialOrd`를 구현했으므로 [`01_generic_largest.rs`](examples/01_generic_largest.rs)에서 같은 함수로 처리됩니다.

Rust는 제네릭 함수의 본문을 정의한 곳에서 **바운드만 보고** 한 번 검사합니다. 그래서 문제가 생기면 "본문이 바운드에 없는 기능을 쓴다"(함수 정의의 에러)거나 "넘긴 타입이 바운드를 만족하지 않는다"(호출한 곳의 에러) 중 하나로 바로 보고됩니다. 실제 타입을 대입해 본 뒤에야 본문 깊은 곳의 에러가 드러나는 C++ 템플릿과 다른 점입니다.

`largest`는 빈 목록을 받으면 `&list[0]`에서 패닉이 납니다. `Option<&T>`를 돌려주도록 고치는 것이 연습 1입니다.

### 2.2 제네릭 구조체와 열거형

구조체와 열거형도 이름 뒤에 타입 매개변수를 선언합니다.

```rust
struct Point<T> {          // T 하나: x와 y는 같은 타입
    x: T,
    y: T,
}

struct KeyValue<K, V> {    // 타입 매개변수 둘: 필드마다 다른 타입
    key: K,
    value: V,
}

enum Slot<T> {             // 제네릭 열거형
    Filled(T),
    Empty,
}
```

`Point<T>`는 `T` 하나로 두 필드를 묶었으므로 `Point { x: 5, y: 4.0 }`은 E0308(`expected integer, found floating-point number`)입니다. `x`의 `5`가 `T`를 정수로 정했는데 `y`에 실수를 넣었기 때문입니다. 필드마다 타입이 달라도 되게 하려면 `KeyValue<K, V>`처럼 타입 매개변수를 늘립니다. 다만 타입 매개변수가 많아지면 읽기 어려우므로, 그럴 때는 타입을 더 작게 나눌 수 없는지 먼저 생각해 봅니다.

지금까지 써 온 표준 라이브러리 타입이 바로 이렇게 정의된 제네릭 타입입니다.

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

| 타입 | 처음 쓴 곳 | 타입 매개변수의 뜻 |
| --- | --- | --- |
| `Option<T>` | ch6 | 있을 수도 없을 수도 있는 값의 타입 |
| `Vec<T>` | ch8 | 요소의 타입 |
| `HashMap<K, V>` | ch8 | 키의 타입, 값의 타입 |
| `Result<T, E>` | ch9 | 성공 값의 타입, 에러 값의 타입 |

[`02_generic_types.rs`](examples/02_generic_types.rs)의 `Slot<T>`는 `Option<T>`와 같은 모양을 직접 만들어 본 것입니다. 표준 라이브러리의 타입이라고 특별한 문법을 쓰는 것이 아니라는 뜻입니다.

### 2.3 제네릭 메서드와 특정 타입 전용 `impl`

```rust
impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

- **`impl<T> Point<T>`**: `impl` 바로 뒤의 `<T>`가 "이 블록의 `T`는 타입 매개변수다"라는 선언입니다. 이것이 없으면 컴파일러는 `Point<T>`의 `T`를 `T`라는 이름의 실제 타입으로 찾으려 합니다. 이 블록의 메서드는 `T`가 무엇이든 모든 `Point<T>`에 생깁니다.
- **`impl Point<f32>`**: `impl` 뒤에 `<T>`가 없고 **구체 타입(concrete type)** 을 적었습니다. `distance_from_origin`은 `Point<f32>`에만 생깁니다. `powi`(거듭제곱)와 `sqrt`(제곱근)는 실수 타입에만 있는 메서드라서 아무 `T`에나 구현할 수 없기 때문입니다. 정수 `Point`에서 부르면 E0599가 납니다.

```text
error[E0599]: no method named `distance_from_origin` found for struct `Point<{integer}>` in the current scope
...
   = note: the method was found for `Point<f32>`
```

`{integer}`는 "아직 어느 정수 타입인지 정해지지 않은 정수"라는 뜻이고, `note:`가 이 메서드는 `Point<f32>`에만 있다고 알려 줍니다.

메서드는 구조체와 별개로 자기만의 타입 매개변수를 가질 수 있습니다. 구조체의 것은 `impl` 뒤에, 메서드에서만 쓰는 것은 메서드 이름 뒤에 선언합니다.

```rust
impl<K, V> KeyValue<K, V> {
    fn with_value<V2>(self, value: V2) -> KeyValue<K, V2> {
        KeyValue { key: self.key, value }
    }
}
```

ch5 2.12절에서 "한 타입에 `impl` 블록을 여러 개 둘 수 있고, 블록을 나누는 것이 필요한 경우는 ch10에서 만난다"고 했습니다. `impl<T> Point<T>`와 `impl Point<f32>`처럼 **적용 대상이 다른 블록**이 그런 경우입니다. 2.11절의 조건부 구현도 같은 이유로 블록을 나눕니다.

### 2.4 단형성화: 제네릭에는 실행 비용이 없습니다

제네릭 코드는 실행 중에 "`T`가 무엇인지" 확인하지 않습니다. 컴파일러가 제네릭이 실제로 쓰인 곳을 모두 찾아 **쓰인 타입마다 구체적인 코드를 따로 만들어 내기** 때문입니다. 이것을 **단형성화(monomorphization)** 라고 부릅니다. 예를 들어 `Some(5)`와 `Some(5.0)`을 쓰면 컴파일러는 대략 다음과 같은 두 타입을 만듭니다(실제로 붙는 이름은 다릅니다).

```rust
enum Option_i32 {
    Some(i32),
    None,
}

enum Option_f64 {
    Some(f64),
    None,
}
```

`largest`도 `i32` 목록과 `char` 목록으로 불렀으므로 `i32`용과 `char`용 두 벌이 만들어집니다. 2.1절에서 손으로 쓴 `largest_i32`, `largest_char`를 컴파일러가 대신 써 주는 셈이라, 손으로 쓴 코드와 똑같이 빠릅니다. 대신 쓰인 타입이 많을수록 컴파일 시간과 실행 파일 크기가 늘어납니다.

[`02_generic_types.rs`](examples/02_generic_types.rs)의 6번 구역은 같은 정의에서 나온 `Point<u8>`(2바이트)과 `Point<f64>`(16바이트)의 크기가 다르다는 것을 보여 줍니다. 타입마다 별개의 구체 타입이 만들어졌다는 증거입니다.

### 2.5 트레이트 정의와 구현

**트레이트(trait)** 는 여러 타입이 공유할 수 있는 동작을 메서드 시그니처의 목록으로 정의한 것입니다. Java나 C#의 인터페이스와 비슷하지만, 타입을 정의할 때가 아니라 나중에 별도의 `impl` 블록으로 구현하고, 이미 있는 타입(표준 라이브러리의 `Vec` 등)에도 구현할 수 있다는 점이 다릅니다.

```rust
pub trait Summary {
    fn summarize_author(&self) -> String; // 구현하는 타입이 반드시 작성합니다.

    fn summarize(&self) -> String {       // 기본 구현
        format!("({}님의 글 더 보기...)", self.summarize_author())
    }
}

impl Summary for NewsArticle {
    fn summarize_author(&self) -> String {
        self.author.clone()
    }

    fn summarize(&self) -> String {       // 기본 구현을 재정의합니다.
        format!("{}, {} ({})", self.headline, self.author, self.location)
    }
}

impl Summary for SocialPost {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
    // summarize는 기본 구현을 그대로 씁니다.
}
```

- `impl 트레이트 for 타입`으로 구현합니다. ch5의 `impl Rectangle`은 타입에 직접 메서드를 붙이는 블록이었고, 이것은 트레이트가 정한 메서드를 그 타입에 맞게 채우는 블록입니다.
- 본문 없이 `;`로 끝나는 메서드는 구현하는 타입이 반드시 작성해야 합니다. 빠뜨리면 E0046(``not all trait items implemented, missing: `summarize_author` ``)입니다.
- 본문이 있는 메서드는 **기본 구현(default implementation)** 입니다. 구현하는 타입이 작성하지 않으면 기본 구현이 쓰이고, 같은 시그니처로 작성하면 그 본문으로 **재정의(override)** 됩니다.
- 기본 구현은 같은 트레이트의 다른 메서드를 부를 수 있습니다. 위의 기본 `summarize`는 타입마다 구현한 `summarize_author`를 불러 씁니다. 그래서 `SocialPost`는 짧은 메서드 하나만 구현하고도 `summarize`를 얻습니다. 반대로 메서드를 재정의한 본문 안에서 그 메서드의 기본 구현을 불러 쓸 수는 없습니다.

### 2.6 트레이트는 스코프에 있어야 메서드를 쓸 수 있습니다

[`03_summary_trait.rs`](examples/03_summary_trait.rs)는 트레이트와 타입을 `aggregator` 모듈에 두고 `main`에서 `use aggregator::{NewsArticle, SocialPost, Summary};`로 가져옵니다. 여기서 `Summary`를 빼면 타입은 보이지만 메서드를 부를 수 없습니다.

```text
error[E0599]: no method named `summarize` found for struct `NewsArticle` in the current scope
...
   = help: items from traits can only be used if the trait is in scope
help: trait `Summary` which provides `summarize` is implemented but not in scope; perhaps you want to import it
   |
 6 + use crate::aggregator::Summary;
   |
```

ch2에서 `random_range`를 부르려고 `rand::RngExt`를 가져온 것이 같은 이유였습니다. 트레이트의 메서드는 타입이 아니라 트레이트에 속하므로, 그 트레이트를 가져온 곳에서만 보입니다. 서로 다른 트레이트가 같은 이름의 메서드를 가질 수 있는데, 이 규칙 덕분에 쓰는 쪽이 가져온 트레이트의 메서드만 후보가 됩니다. 다른 크레이트가 새 트레이트를 추가해도 내 코드의 메서드 호출 뜻이 몰래 바뀌지 않습니다.

`clone()`, `to_string()`, `==`, `<`처럼 `use` 없이 써 온 기능은 그 트레이트(`Clone`, `ToString`, `PartialEq`, `PartialOrd`)가 프렐류드(prelude, ch8)에 들어 있기 때문입니다. 반면 `Display`는 프렐류드에 없어서, 바운드나 `impl`에 이름을 쓰려면 `use std::fmt::Display;`로 가져와야 합니다.

트레이트 메서드는 `Summary::summarize(&article)`처럼 `트레이트::메서드(&값)` 형태로도 부를 수 있습니다. ch5의 `Account::balance(&account)`와 같은 원리이며, 어느 트레이트의 메서드인지 코드에 드러납니다.

### 2.7 고아 규칙: 트레이트나 타입 중 하나는 내 것이어야 합니다

`impl 트레이트 for 타입`은 **트레이트와 타입 중 적어도 하나가 지금 크레이트(ch7)에서 정의된 것**일 때만 쓸 수 있습니다.

| impl | 트레이트 | 타입 | 허용 | 예제 |
| --- | --- | --- | --- | --- |
| `impl Summary for NewsArticle` | 내 것 | 내 것 | 예 | 03 |
| `impl Summary for Vec<SocialPost>` | 내 것 | 표준(`Vec`) | 예 | 03 |
| `impl Display for SocialPost` | 표준 | 내 것 | 예 | 05 |
| `impl Display for Vec<SocialPost>` | 표준 | 표준(`Vec`) | 아니요 (E0117) | - |

마지막 줄은 `Vec` 안에 내 타입이 들어 있어도 `Vec` 자체가 표준 라이브러리의 타입이라 거부됩니다.

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
...
7 | impl fmt::Display for Vec<SocialPost> {
  | ^^^^^^^^^^^^^^^^^^^^^^---------------
  |                       |
  |                       `Vec` is not defined in the current crate
  |
  = note: impl doesn't have any local type before any uncovered type parameters
  = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
  = note: define and implement a trait or new type instead
```

이 규칙을 **고아 규칙(orphan rule)** 이라고 부릅니다. 트레이트도 타입도 내 크레이트에 없는 구현은 "부모가 없는" 구현이라는 뜻입니다. 이 규칙이 없다면 서로 다른 두 크레이트가 각자 `impl Display for Vec<T>`를 작성할 수 있고, 두 크레이트를 함께 쓰는 프로그램은 어느 구현을 써야 할지 정할 수 없습니다. 고아 규칙은 "한 프로그램 안에서 (트레이트, 타입) 쌍마다 구현은 많아야 하나"라는 성질, 즉 **일관성(coherence)** 을 지키는 장치입니다. 덕분에 의존성을 하나 추가했다고 다른 크레이트의 구현이 내 코드와 충돌하는 일이 생기지 않습니다.

남의 타입에 남의 트레이트를 꼭 구현해야 한다면, 마지막 `note:`처럼 그 타입을 감싼 내 타입을 만들어 거기에 구현합니다(뉴타입 패턴, [ch19](../ch19-advanced-features/README.md)).

### 2.8 `Display`, `Debug`, 그리고 `derive`

ch1에서 `{}`는 기본 형식, `{:?}`는 디버그 형식이라고 했습니다. 정확히 말하면 `{}`는 `std::fmt::Display` 트레이트의 `fmt` 메서드를, `{:?}`는 `std::fmt::Debug` 트레이트의 `fmt` 메서드를 부릅니다. 어떤 타입을 `{}`로 출력할 수 있다는 것은 그 타입이 `Display`를 구현했다는 뜻입니다.

#### `Display` 직접 구현하기

ch5에서 미뤄 둔, 구조체의 기본 형식을 직접 정하는 방법입니다.

```rust
use std::fmt;

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

- `f`는 출력이 쌓이는 곳(`Formatter`)입니다. `fmt`가 화면에 직접 찍지 않고 여기에 쓰기 때문에, 같은 구현을 `println!`, `format!`, `to_string()` 어디서든 쓸 수 있습니다.
- `write!(f, ...)`는 `println!`과 같은 서식 문자열을 받아 `f`에 씁니다.
- 반환 타입 `fmt::Result`는 쓰기가 성공했는지 알리는 `Result`입니다. `write!`가 돌려준 값을 그대로 돌려주면 됩니다.

이제 `println!("{p}")`가 `(3, -2)`를 출력하고, `p.to_string()`과 `format!("{p}")`도 쓸 수 있습니다(`to_string`이 저절로 생기는 이유는 2.11절). 직접 구현한 `Display`에서 너비·정렬 같은 서식 지정을 지원하는 방법 등 자세한 내용은 [Part 1 ch3](../../part1-standard-lib/ch3-strings-fmt/README.md)에서 다룹니다.

`Display`는 derive할 수 없습니다. 사용자에게 보여 줄 모양은 타입마다 달라서 컴파일러가 정해 줄 수 없기 때문입니다.

#### `derive`는 트레이트 구현을 만들어 줍니다

ch5의 `#[derive(Debug)]`는 "`Debug` 트레이트의 `impl` 블록을 컴파일러가 대신 작성해 달라"는 요청이었습니다. 이 챕터의 예제에서 쓰는 derive는 다음과 같습니다.

| 트레이트 | 얻는 기능 |
| --- | --- |
| `Debug` | `{:?}`, `{:#?}`로 출력 |
| `Clone` | `.clone()`으로 복제 |
| `Copy` | 대입·전달할 때 이동 대신 복사(ch4). `Clone`도 함께 derive해야 합니다. |
| `PartialEq` | `==`, `!=` |
| `PartialOrd` | `<`, `>`, `<=`, `>=` (필드를 선언 순서대로 비교) |

이 밖에 `Eq`, `Ord`, `Hash`, `Default`도 derive할 수 있습니다. derive에는 두 가지 조건이 있습니다.

- **모든 필드가 그 트레이트를 구현해야** 합니다. 자동 구현은 필드마다 같은 트레이트의 기능을 불러 쓰기 때문입니다. `Debug`가 없는 타입을 필드로 가진 구조체에 `#[derive(Debug)]`를 붙이면 E0277(`` `Inner` doesn't implement `Debug` ``)입니다.
- **`Copy`는 `Clone`을 전제로 합니다.** `#[derive(Debug, Copy)]`만 쓰면 E0277(``the trait bound `Version: Clone` is not satisfied``)입니다.

[`04_display_and_derive.rs`](examples/04_display_and_derive.rs)의 `Version(1, 8, 2) < Version(1, 10, 0)`은 `true`입니다. derive한 `PartialOrd`는 첫 필드부터 차례로 비교하므로, 첫 필드 `1 == 1` 다음의 `8 < 10`으로 결정됩니다. 문자열 `"1.8.2" < "1.10.0"`으로 비교했다면 `'8' > '1'`이라서 `false`였을 것입니다. 이렇게 `PartialOrd`를 얻은 `Version`은 `largest<T: PartialOrd>`의 바운드도 만족하므로 그대로 넘길 수 있습니다. derive 목록에서 `PartialOrd`를 지우면 그 호출이 거부됩니다.

```text
error[E0277]: can't compare `Version` with `Version`
...
   = help: the trait `PartialOrd` is not implemented for `Version`
note: required by a bound in `largest`
...
help: consider annotating `Version` with `#[derive(PartialOrd)]`
```

### 2.9 트레이트 바운드: 이 능력이 있는 타입만 받기

트레이트를 매개변수 자리에 쓰면 "이 트레이트를 구현한 타입이면 무엇이든 받는다"는 함수를 만들 수 있습니다. 쓰는 법이 여러 가지인데, 모두 [`05_trait_bounds.rs`](examples/05_trait_bounds.rs)에 있습니다.

| 쓰는 법 | 뜻 |
| --- | --- |
| `fn notify(item: &impl Summary)` | `Summary`를 구현한 어떤 타입이든 받습니다. |
| `fn notify<T: Summary>(item: &T)` | 위와 같은 뜻을 트레이트 바운드 문법으로 쓴 것입니다. |
| `fn f(a: &impl Summary, b: &impl Summary)` | `a`와 `b`가 서로 다른 타입이어도 됩니다. |
| `fn f<T: Summary>(a: &T, b: &T)` | `a`와 `b`가 반드시 같은 타입이어야 합니다. |
| `fn f(item: &(impl Summary + Display))`<br>`fn f<T: Summary + Display>(item: &T)` | `+`로 연결한 트레이트를 모두 구현해야 합니다. |
| `fn f<T, U>(t: &T, u: &U) -> String`<br>`where T: Summary + Display, U: Summary + Debug` | 바운드를 `where` 절로 옮겼을 뿐 뜻은 같습니다. |

- `impl Trait` 문법은 트레이트 바운드 문법의 줄임말입니다. 간단할 때는 `impl Trait`가 읽기 쉽고, 두 인자가 같은 타입이어야 하는 것처럼 타입 매개변수에 이름이 필요할 때는 바운드 문법을 씁니다. `notify_same(&article, &post)`처럼 다른 타입을 섞어 넘기면 E0308(``expected `&NewsArticle`, found `&SocialPost` ``)입니다.
- 함수 안에서는 **바운드에 적은 트레이트의 기능만** 쓸 수 있습니다. 넘어온 값이 실제로 `SocialPost`라도 `item.username`은 E0609(``no field `username` on type `&impl Summary` ``)입니다. 본문은 바운드만 보고 검사되기 때문입니다(2.1절).
- 바운드를 만족하지 않는 값을 넘기면 **호출한 곳**에서 E0277이 납니다(4.2절). 바운드는 함수와 호출자 사이의 약속입니다. 함수는 바운드에 있는 기능만 쓰고, 호출자는 바운드를 만족하는 값만 넘깁니다.
- `where` 절은 바운드가 길어질 때 시그니처 뒤로 옮겨 함수 이름·매개변수·반환 타입이 한눈에 보이게 합니다.

```rust
fn compare<T, U>(t: &T, u: &U) -> String
where
    T: Summary + Display,
    U: Summary + Debug,
{
    format!("{t} vs {u:?}")
}
```

### 2.10 반환 위치의 `impl Trait`

```rust
fn make_reply(to: &str) -> impl Summary {
    SocialPost {
        username: String::from("bot"),
        content: format!("{to}님, 글 잘 봤습니다"),
    }
}
```

반환 타입에 `impl Summary`를 쓰면 "`Summary`를 구현한 어떤 타입 하나를 돌려준다"는 뜻입니다. 호출한 쪽은 그 타입이 `SocialPost`라는 것을 모르고 `Summary`의 메서드만 쓸 수 있습니다. `reply.username`은 E0609(``no field `username` on type `impl Summary` ``)입니다. 구체 타입을 숨겨 두면, 함수를 만든 쪽이 나중에 `Summary`를 구현한 다른 타입으로 바꿔도 호출하는 코드를 고칠 필요가 없습니다. 이 문법이 특히 쓸모 있는 곳은 클로저와 반복자처럼 타입 이름이 아주 길거나 아예 이름을 쓸 수 없는 경우이며, [ch13](../ch13-closures-iterators/README.md)에서 보게 됩니다.

다만 이름만 숨겼을 뿐, 컴파일러는 반환 타입을 **정확히 하나**로 정해야 합니다. 호출하는 쪽의 코드를 컴파일할 때 반환값의 크기와 어느 타입의 메서드를 부를지를 알아야 하기 때문입니다. `if`와 `else`에서 서로 다른 타입을 돌려주면 E0308로 거부됩니다(4.6절). 실행 중에 여러 타입 중 하나를 돌려주는 방법(트레이트 객체 `dyn Trait`)은 [ch17](../ch17-trait-objects-oop/README.md)에서 배웁니다.

### 2.11 바운드를 이용한 조건부 구현과 포괄 구현

#### 조건을 만족하는 타입에만 메서드 주기

`impl` 블록의 타입 매개변수에 바운드를 달면, 그 조건을 만족하는 타입에만 메서드가 생깁니다.

```rust
impl<T> Pair<T> {                      // 모든 Pair<T>
    fn new(x: T, y: T) -> Self { Self { x, y } }
}

impl<T: Display + PartialOrd> Pair<T> { // T가 Display + PartialOrd일 때만
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("더 큰 쪽은 x = {}", self.x);
        } else {
            println!("더 큰 쪽은 y = {}", self.y);
        }
    }
}
```

`cmp_display`의 본문은 `>=`(`PartialOrd`)와 `{}`(`Display`)를 쓰므로 두 능력이 꼭 필요합니다. [`06_conditional_and_blanket_impl.rs`](examples/06_conditional_and_blanket_impl.rs)에서 `Pair<i32>`와 `Pair<&str>`은 `cmp_display`를 부를 수 있지만, 아무 트레이트도 구현하지 않은 `Ticket`으로 만든 `Pair<Ticket>`에는 `new`만 있습니다.

```text
error[E0599]: the method `cmp_display` exists for struct `Pair<Ticket>`, but its trait bounds were not satisfied
...
note: the following trait bounds were not satisfied:
      `Ticket: PartialOrd`
      `Ticket: std::fmt::Display`
```

#### 포괄 구현: 조건을 만족하는 모든 타입에 트레이트 구현하기

타입 매개변수 자체에 트레이트를 구현할 수도 있습니다. 이것을 **포괄 구현(blanket implementation)** 이라고 부릅니다.

```rust
trait Bracketed {
    fn bracketed(&self) -> String;
}

impl<T: Display> Bracketed for T {
    fn bracketed(&self) -> String {
        format!("[{self}]")
    }
}
```

"`Display`를 구현한 모든 타입 `T`"에 `Bracketed`를 구현했으므로 `42.bracketed()`, `"러스트".bracketed()`, 직접 `Display`를 구현한 `Score(95).bracketed()`가 모두 동작합니다. 앞으로 누군가 `Display`를 구현할 타입도 자동으로 포함됩니다.

표준 라이브러리도 같은 방법을 씁니다. `ToString` 문서의 구현 목록(Implementors)에는 `impl<T> ToString for T where T: Display + ?Sized`가 있습니다(`?Sized`는 `str`처럼 크기가 정해지지 않은 타입도 포함한다는 뜻이며 [ch19](../ch19-advanced-features/README.md)에서 다룹니다). 2.8절에서 `Display`만 구현했는데 `to_string()`이 생긴 이유가 이것입니다. 그래서 문서는 `ToString`을 직접 구현하지 말고 `Display`를 구현하라고 안내합니다.

### 2.12 라이프타임은 댕글링 참조를 막기 위한 것입니다

ch4의 대여 규칙 두 번째는 "참조는 항상 유효해야 한다"였습니다. 이것을 검사하려고 Rust는 모든 참조에 **라이프타임(lifetime)**, 즉 그 참조가 유효해야 하는 구간을 매깁니다. 대여 검사기(borrow checker)는 두 구간을 비교합니다.

- **값이 살아 있는 구간**: 선언한 곳부터 소유자의 스코프가 끝나 drop되는 곳까지(ch4)
- **참조가 쓰이는 구간**: 참조를 만든 곳부터 그 참조가 마지막으로 쓰이는 곳까지(ch4 2.10절)

```rust
let r;
{
    let x = 5;              // --+-- x가 살아 있는 구간
    r = &x;                 //   |   --+-- r이 쓰이는 구간
}                           // --+     |   x가 drop됨
println!("r: {r}");         //       --+   r의 마지막 사용
```

`r`이 쓰이는 구간이 `x`가 살아 있는 구간 밖으로 나가므로, 이 코드는 해제된 값을 읽는 댕글링 참조가 됩니다. 대여 검사기는 이를 E0597로 거부합니다.

[`07_longest_lifetime.rs`](examples/07_longest_lifetime.rs) 1번 구역의 주석 처리된 `println!("{r}");`의 주석을 풀면 다음 에러를 볼 수 있습니다.

```text
error[E0597]: `x` does not live long enough
...
28 |         let x = 5;
   |             - binding `x` declared here
29 |         r = &x;
   |             ^^ borrowed value does not live long enough
30 |         println!("    블록 안에서 r = {r}"); // r의 마지막 사용이 x가 살아 있는 블록 안이므로 괜찮습니다.
31 |     } // x가 여기서 drop됩니다.
   |     - `x` dropped here while still borrowed
...
35 |     println!("{r}");
   |                - borrow later used here
```

주석을 풀기 전에는 `r`의 마지막 사용(30번 줄)이 `x`의 구간 안에 있으므로 컴파일됩니다. 메시지의 세 표시가 "어디서 빌렸고(`^^`) → 어디서 값이 사라졌고(`dropped here`) → 그 뒤에 어디서 썼는지(`later used here`)"를 순서대로 보여 줍니다.

#### 함수 경계에서는 표기가 필요합니다

함수 본문 안에서는 컴파일러가 모든 구간을 직접 볼 수 있으므로 라이프타임을 적을 일이 없습니다. 그런데 컴파일러는 함수를 하나씩 따로 검사하고, 함수를 부르는 곳을 검사할 때는 그 함수의 **본문을 보지 않고 시그니처만** 봅니다. 매개변수의 타입을 시그니처에 적어야 하는 것처럼, "반환된 참조가 어느 인자에서 빌린 것인지"도 시그니처에 드러나야 호출하는 쪽을 검사할 수 있습니다. 그 정보를 적는 것이 **라이프타임 표기(lifetime annotation)** 입니다.

### 2.13 `longest`: 함수 시그니처의 라이프타임

두 문자열 슬라이스 중 긴 쪽을 돌려주는 함수를 표기 없이 쓰면 E0106입니다(4.3절).

```text
9 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
```

`help:`가 문제를 정확히 짚습니다. 반환값이 `x`를 빌린 것인지 `y`를 빌린 것인지 시그니처가 말해 주지 않습니다. 본문을 봐도 `if` 갈래는 `x`를, `else` 갈래는 `y`를 돌려주므로 정할 수 없습니다.

라이프타임 표기의 문법은 다음과 같습니다.

```rust
&i32        // 참조
&'a i32     // 라이프타임을 명시한 참조
&'a mut i32 // 라이프타임을 명시한 가변 참조
```

라이프타임 이름은 `'`로 시작하는 짧은 소문자이고, 관례상 `'a`부터 씁니다. 제네릭 타입 매개변수처럼 함수 이름 뒤 `<>`에 **라이프타임 매개변수(lifetime parameter)** 로 선언합니다.

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

이 시그니처는 "어떤 구간 `'a`에 대해, `x`와 `y`는 둘 다 적어도 `'a` 동안 유효하고, 반환된 참조도 `'a` 동안 유효하다"는 뜻입니다. 실제로 호출하면 `'a`는 `x`와 `y`가 **모두** 유효한 구간, 즉 두 인자 중 짧게 사는 쪽에 맞춰집니다. 그래서 결과는 짧은 쪽 인자가 살아 있는 동안만 쓸 수 있습니다.

| 호출하는 코드 | 결과 | 예제 |
| --- | --- | --- |
| `result`를 짧게 사는 인자의 블록 **안**에서만 사용 | 컴파일됨 | 07의 3번 구역 |
| `result`를 짧게 사는 인자의 블록 **밖**에서 사용 | E0597 | 4.4절 |

두 번째 경우, `result`가 실제로는 오래 사는 쪽을 가리키더라도 거부됩니다. 컴파일러는 본문이 어느 쪽을 돌려줄지가 아니라 시그니처만 보고 판단하기 때문입니다.

#### 표기는 관계를 설명할 뿐, 수명을 바꾸지 않습니다

`'a`를 붙인다고 짧게 사는 인자가 더 오래 살지는 않습니다. 값이 언제 drop될지는 소유권 규칙이 정하고, 라이프타임 표기는 "반환값과 인자들이 이런 관계여야 한다"는 조건을 적을 뿐입니다. 대여 검사기는 그 조건을 어기는 코드를 거부합니다. 그러니 시그니처에는 함수가 **실제로 하는 일**을 그대로 적어야 합니다.

```rust
fn strip_tag<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}
```

`strip_tag`의 결과는 언제나 `text`의 일부이므로 `text`에만 `'a`를 붙였습니다. 그러면 `prefix`로 넘긴 문자열은 결과보다 먼저 drop되어도 됩니다(07의 4번 구역). 본문을 한 글자도 바꾸지 않고 `prefix`에도 `'a`를 붙이면, 같은 호출 코드가 E0597(`` `tag` does not live long enough ``)로 거부됩니다. 시그니처가 "결과는 `prefix`도 빌린다"고 말하게 되기 때문입니다.

반대로 본문이 시그니처의 약속보다 많은 것을 돌려주려 하면 함수 정의에서 거부됩니다. `longest`의 `y`에서 `'a`를 빼면 다음 에러가 납니다.

```text
error[E0621]: explicit lifetime required in the type of `y`
...
2 |     if x.len() > y.len() { x } else { y }
  |                                       ^ lifetime `'a` required
```

호출하는 쪽은 시그니처를 믿고 검사받고, 함수 본문은 시그니처를 지키는지 검사받습니다. 양쪽을 따로 검사할 수 있게 해 주는 것이 시그니처의 라이프타임입니다.

#### 지역 변수의 참조는 어떤 표기로도 반환할 수 없습니다

반환하는 참조가 매개변수가 아니라 함수 안에서 만든 값을 가리키면, 라이프타임을 어떻게 적어도 E0515입니다(4.5절). 그 값은 함수가 끝날 때 drop되기 때문입니다. ch4의 `dangle`에 `'a`를 붙여 `fn dangle<'a>() -> &'a String`으로 바꿔도 E0106이 ``E0515: cannot return reference to local variable `s` ``로 바뀔 뿐입니다. 해결책은 ch4에서와 같이 소유한 값(`String`)을 돌려주는 것입니다(07의 `emphasize`).

### 2.14 구조체의 라이프타임

구조체 필드에 참조를 넣으려면 라이프타임 매개변수를 선언하고 필드에 붙입니다.

```rust
struct ImportantExcerpt<'a> {
    part: &'a str,
}
```

"`ImportantExcerpt` 인스턴스는 `part`가 가리키는 값보다 오래 살 수 없다"는 뜻입니다. ch5 4.5절에서 E0106으로 막혔던 `User`도 컴파일러의 `help:`가 제안한 대로 고치면 됩니다.

```rust
struct User<'a> {
    username: &'a str,
    email: &'a str,
}
```

[`08_struct_elision_static.rs`](examples/08_struct_elision_static.rs)에서 `excerpt`는 `novel`의 일부를 빌리므로, `novel`이 살아 있는 동안만 쓸 수 있습니다. `novel`을 안쪽 블록에 두고 `excerpt`를 블록 밖에서 쓰면 E0597(`` `novel` does not live long enough ``)입니다.

구조체의 기본 선택은 여전히 데이터를 소유하는 필드(`String` 등)입니다. 참조 필드는 큰 데이터의 일부를 복사하지 않고 잠깐 들여다보는 구조체(예: 문자열을 앞에서부터 읽어 나가는 파서)에 어울립니다. 라이프타임 매개변수가 붙은 타입은 그 타입을 쓰는 곳마다 관계가 따라다니므로, 확신이 없으면 소유한 필드로 시작하는 편이 쉽습니다.

### 2.15 라이프타임 생략 규칙

ch4의 `first_word(s: &str) -> &str`은 참조를 돌려주는데도 표기 없이 컴파일되었습니다. 초기 Rust에서는 모든 참조에 라이프타임을 적어야 했지만, 사람들이 같은 모양을 반복해 적는다는 것을 알게 된 뒤 흔한 경우를 컴파일러가 채워 넣도록 규칙을 정했습니다. 이것이 **라이프타임 생략 규칙(lifetime elision rules)** 입니다. 매개변수에 붙는 라이프타임을 **입력 라이프타임**, 반환값에 붙는 것을 **출력 라이프타임**이라고 부릅니다.

1. **규칙 1 (입력)**: 참조인 매개변수마다 서로 다른 라이프타임이 하나씩 붙습니다. `fn f(x: &i32, y: &i32)`는 `fn f<'a, 'b>(x: &'a i32, y: &'b i32)`가 됩니다.
2. **규칙 2 (출력, 입력이 하나)**: 입력 라이프타임이 정확히 하나면, 모든 출력 라이프타임에 그것이 붙습니다.
3. **규칙 3 (출력, 메서드)**: 입력 라이프타임이 여러 개라도 그중 하나가 `&self`나 `&mut self`면, 모든 출력 라이프타임에 `self`의 라이프타임이 붙습니다.

세 규칙을 적용하고도 출력 라이프타임이 정해지지 않으면 컴파일러는 추측하지 않고 E0106을 냅니다.

| 시그니처 | 적용되는 규칙 | 결과 | 예제 |
| --- | --- | --- | --- |
| `fn first_word(s: &str) -> &str` | 1 → 2 | `fn first_word<'a>(s: &'a str) -> &'a str` | 08 |
| `fn split_at_space(s: &str) -> (&str, &str)` | 1 → 2 | 두 출력 모두 `s`의 라이프타임 | 08 |
| `fn largest<T: PartialOrd>(list: &[T]) -> &T` | 1 → 2 | 출력은 `list`의 라이프타임 | 01 |
| `fn announce_and_return_part(&self, announcement: &str) -> &str` | 1 → 3 | 출력은 `self`의 라이프타임 | 08 |
| `fn longest(x: &str, y: &str) -> &str` | 1만 | 출력이 정해지지 않음 → E0106 | 4.3절 |
| `fn level_name(level: i32) -> &str` | 없음(입력 참조 없음) | 출력이 정해지지 않음 → E0106 | 2.17절 |

01의 `largest`가 표기 없이 컴파일된 것도 규칙 2 덕분입니다. 반환값은 `list`를 빌린 것이므로, 01의 4번 구역에서 반환된 참조가 `number_list` 안의 원소를 가리킨 것입니다.

08의 `first_word_explicit`는 규칙 1·2가 채워 넣는 라이프타임을 직접 적은 함수로, `first_word`와 완전히 같습니다. 생략할 수 있는 라이프타임을 적으면 Clippy가 `clippy::needless_lifetimes` 경고를 내므로, 예제에서는 `#[expect(...)]`와 이유를 달아 두었습니다.

### 2.16 메서드의 라이프타임

```rust
impl<'a> ImportantExcerpt<'a> {
    fn level(&self) -> i32 {
        3
    }

    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("알림: {announcement}");
        self.part
    }
}
```

`impl<'a>`는 `impl<T>`처럼 라이프타임 매개변수의 선언이고, 라이프타임은 구조체 타입의 일부이므로 `ImportantExcerpt<'a>`까지 적습니다. 메서드 시그니처에는 규칙 3 덕분에 표기가 거의 필요 없습니다. `level`은 참조를 돌려주지 않고, `announce_and_return_part`는 입력 참조가 둘이지만 `&self`가 있어 출력이 `self`의 라이프타임으로 정해집니다.

생략 규칙은 흔한 경우를 정해 둔 약속일 뿐, 본문을 보고 정하는 것이 아닙니다. 규칙 3이 정한 관계(반환값은 `self`에서 빌림)와 달리 `announcement`를 돌려주면 에러가 납니다.

```text
error: lifetime may not live long enough
...
5 |     fn announce(&self, announcement: &str) -> &str {
  |                 -                    - let's call the lifetime of this reference `'1`
  |                 |
  |                 let's call the lifetime of this reference `'2`
6 |         println!("{}", self.part);
7 |         announcement
  |         ^^^^^^^^^^^^ method was supposed to return data with lifetime `'2` but it is returning data with lifetime `'1`
```

이 에러에는 E 코드가 없습니다. 컴파일러가 이름 없는 라이프타임에 `'1`, `'2`라는 임시 이름을 붙여 설명합니다. `help:`가 제안하는 대로 `fn announce<'b>(&self, announcement: &'b str) -> &'b str`처럼 실제 관계를 직접 적으면 됩니다.

### 2.17 `'static` 라이프타임

`'static`은 "프로그램이 끝날 때까지 유효하다"는 뜻의 특별한 라이프타임입니다. 문자열 리터럴은 프로그램 실행 파일 안에 들어 있으므로 모두 `&'static str`입니다.

```rust
let s: &'static str = "문자열 리터럴은 모두 'static입니다";
```

`'static`이 꼭 필요한 대표적인 경우는 입력 참조가 없는 함수가 리터럴을 돌려줄 때입니다. 입력 라이프타임이 없으니 생략 규칙으로 출력을 정할 수 없어(2.15절 표의 마지막 줄) 직접 적어야 합니다.

```rust
fn level_name(level: i32) -> &'static str {
    match level {
        1 => "참고",
        2 => "중요",
        _ => "매우 중요",
    }
}
```

`'static`을 빼면 E0106의 `help:`가 `'static`을 제안하면서 ``but this is uncommon unless you're returning a borrowed value from a `const` or a `static` ``라고 덧붙입니다. 리터럴도 프로그램에 고정된 값이라 이 경우에 해당합니다.

**주의**: 에러 메시지가 `'static`을 제안한다고 그대로 붙이면 안 됩니다. ch4의 `dangle`에서도 E0106이 `'static`을 제안했지만, 함수 안에서 만든 `String`의 참조는 `'static`을 붙여도 E0515로 거부됩니다. 이런 제안은 대부분 댕글링 참조나 잘못 적은 관계가 진짜 원인입니다. `'static`을 붙이기 전에 "이 참조가 정말 프로그램 끝까지 남는 데이터를 가리키는가"를 확인하세요.

`'static`은 `T: 'static`처럼 트레이트 바운드 자리에도 쓰이는데, 이때는 뜻이 조금 다릅니다. 참고 자료의 Rust By Example [Static](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html) 절에 두 쓰임이 정리되어 있고, 깊이 있는 내용은 Part 10에서 다룹니다.

### 2.18 세 가지를 함께 쓰기

라이프타임도 제네릭 매개변수의 한 종류라서, 타입 매개변수와 같은 `<>` 안에 선언합니다. 관례상 라이프타임을 먼저 씁니다.

```rust
fn longest_with_an_announcement<'a, T>(x: &'a str, y: &'a str, ann: T) -> &'a str
where
    T: Display,
{
    println!("알림! {ann}");
    if x.len() > y.len() { x } else { y }
}
```

`'a`는 `longest`와 같은 관계를, `T: Display`는 본문에서 `ann`을 `{}`로 출력한다는 요구를 적은 것입니다. `ann`은 값으로 받고 반환값과 상관이 없으므로 라이프타임이 없습니다. [`09_generics_bounds_lifetimes.rs`](examples/09_generics_bounds_lifetimes.rs)는 같은 함수에 `&str`, `i32`, 직접 만든 `Temperature`를 차례로 `ann`으로 넘깁니다.

세 가지가 서로 맞물린 함수도 있습니다.

```rust
fn longer_display<'a, T: Display>(a: &'a T, b: &'a T) -> &'a T {
    if a.to_string().len() >= b.to_string().len() { a } else { b }
}
```

- `T`: 어떤 타입이든 받습니다(2.1절).
- `T: Display`: 길이를 재려고 쓴 `to_string()`은 `Display`가 있어야 생깁니다(2.11절의 포괄 구현).
- `'a`: 입력 참조가 둘이라 생략 규칙으로 정해지지 않으므로, 반환값이 두 인자 모두에서 올 수 있다고 적습니다(2.13절).

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_generic_largest.rs`](examples/01_generic_largest.rs) | 함수 추출 → 제네릭 함수 `largest`, `T: PartialOrd` | 1 |
| [`02_generic_types.rs`](examples/02_generic_types.rs) | 제네릭 구조체·열거형·메서드, `impl Point<f32>`, 단형성화 | 1 |
| [`03_summary_trait.rs`](examples/03_summary_trait.rs) | `Summary` 트레이트 정의·구현, 기본 구현, `use`로 가져오기, `Vec`에 구현 | 2 |
| [`04_display_and_derive.rs`](examples/04_display_and_derive.rs) | `Display` 직접 구현, `derive`가 만든 구현으로 비교·복사 | 2 |
| [`05_trait_bounds.rs`](examples/05_trait_bounds.rs) | `impl Trait`, `T: Trait`, `+`, `where`, 반환 위치 `impl Trait` | 3 |
| [`06_conditional_and_blanket_impl.rs`](examples/06_conditional_and_blanket_impl.rs) | 바운드를 단 `impl`로 조건부 메서드, 포괄 구현, `ToString` | 3 |
| [`07_longest_lifetime.rs`](examples/07_longest_lifetime.rs) | 대여 검사기의 구간 비교, `longest<'a>`, 표기는 관계를 설명할 뿐이라는 점 | 4 |
| [`08_struct_elision_static.rs`](examples/08_struct_elision_static.rs) | 참조 필드 구조체, 생략 규칙 3가지, 메서드의 라이프타임, `'static` | 5 |
| [`09_generics_bounds_lifetimes.rs`](examples/09_generics_bounds_lifetimes.rs) | 제네릭·트레이트 바운드·라이프타임을 함께 쓰는 함수 | 1, 3, 4 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 6개 (4절) | 1, 3, 4 |

### 01_generic_largest — 중복 제거에서 제네릭 함수까지

위쪽에 `largest_i32`, `largest_char`, `largest` 세 함수가 차례로 정의되어 있습니다. 세 함수의 본문이 똑같고 시그니처만 다르다는 것을 먼저 확인하세요. 4번 구역의 `std::ptr::eq`는 두 참조가 같은 메모리를 가리키는지 비교하는 함수로, 반환값이 복사본이 아니라 목록 안 원소의 참조라는 것을 확인합니다.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 01_generic_largest
```

```text
1. 타입마다 따로 만든 함수
    largest_i32  → 100
    largest_char → y
2. 제네릭 함수 하나로 같은 일을 합니다
    largest(&number_list) → 100
    largest(&char_list)   → y
3. PartialOrd를 구현한 타입이라면 무엇이든 넘길 수 있습니다
    가장 큰 실수: 12.25
    사전순으로 가장 뒤: 체리
4. 반환값은 목록 안 원소의 참조입니다
    number_list[3]을 가리키는가? true
```

`&str`은 글자의 유니코드 코드 포인트 순서로 비교됩니다. 한글 음절은 코드 포인트가 대체로 가나다순으로 배치되어 있어 이 예에서는 `체리` > `사과` > `바나나`입니다.

### 02_generic_types — 제네릭 구조체·열거형·메서드

주석 처리된 두 줄의 주석을 풀면 2.2절의 E0308(`Point { x: 5, y: 4.0 }`)과 2.3절의 E0599(`distance_from_origin`)를 직접 볼 수 있습니다. 6번 구역의 크기는 64비트 환경에서 확인한 값입니다.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 02_generic_types
```

```text
1. 제네릭 구조체는 담는 타입만 바꿔 여러 번 쓸 수 있습니다
    integer = Point { x: 5, y: 10 }, float = Point { x: 1.0, y: 4.0 }
    integer.x() = 5, float.y() = 4
2. 타입 매개변수가 두 개면 필드마다 다른 타입을 담을 수 있습니다
    점수 = 95
3. Point<f32>에만 있는 메서드
    원점에서의 거리: 5
4. 메서드가 자기만의 타입 매개변수를 가질 수 있습니다
    KeyValue { key: "점수", value: "A등급" }
5. 제네릭 열거형: Option<T>와 같은 모양을 직접 만들 수 있습니다
    Filled(7), Empty
    get_or(0): 7, 0
    표준 Option의 unwrap_or: 85, 0
6. 단형성화: 쓰인 타입마다 별도의 구체 타입이 만들어집니다
    Point<u8>:  2바이트
    Point<f64>: 16바이트
    Slot<u8>:   2바이트
    Slot<u64>:  16바이트
```

`float.y()`가 `4.0`이 아니라 `4`로 찍히는 것은 `f64`의 기본 형식(`{}`)이 소수부가 0이면 생략하기 때문이고, 디버그 형식(`{:?}`)에서는 `4.0`으로 찍힙니다. `Slot<u8>`이 2바이트인 것은 값 1바이트에 "어느 배리언트인지"를 나타내는 1바이트가 더해졌기 때문입니다.

### 03_summary_trait — `Summary` 트레이트

`aggregator` 모듈이 라이브러리 역할을 하고, `main`이 그것을 가져다 씁니다. 파일 중간의 `use aggregator::{NewsArticle, SocialPost, Summary};`에서 `Summary`를 지우고 컴파일해 2.6절의 E0599를 확인해 보세요.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 03_summary_trait
```

```text
1. 같은 트레이트를 구현한 서로 다른 타입
    기사: 페리스, 러스트 마스코트 10주년, 김기자 (서울)
    SNS:  (@ferris님의 글 더 보기...)
2. 기본 구현은 구현 타입이 정의한 다른 메서드를 불러 씁니다
    post.summarize_author() = @ferris
    post.content = 오늘은 트레이트를 배웠다
3. 트레이트 메서드도 `트레이트::메서드(&값)` 형태로 부를 수 있습니다
    페리스, 러스트 마스코트 10주년, 김기자 (서울)
4. 내 트레이트를 표준 라이브러리 타입 Vec에 구현했습니다
    글 2개가 이어진 스레드 / 작성자 2명
```

### 04_display_and_derive — `Display`와 `derive`

`Point`는 `Debug`를 derive하고 `Display`는 직접 구현했으므로 1번 구역에서 두 형식을 나란히 비교할 수 있습니다. 4번 구역은 derive한 `PartialOrd`가 01의 `largest`와 같은 제네릭 함수의 바운드를 만족시킨다는 것을 보여 줍니다.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 04_display_and_derive
```

```text
1. {}는 Display, {:?}는 Debug 트레이트의 구현을 부릅니다
    Display: (3, -2)
    Debug:   Point { x: 3, y: -2 }
    필드:    x = 3, y = -2
2. Display를 구현하면 to_string()과 format!도 쓸 수 있습니다
    text = "(3, -2)", sentence = 점 (3, -2)에서 출발
3. derive가 만들어 준 구현으로 비교와 복사를 합니다
    v1.8.2 == v1.8.2 ? true
    v1.8.2 < v1.10.0 ? true
4. derive한 PartialOrd 덕분에 제네릭 함수 largest의 조건을 만족합니다
    최신 버전: v2.0.1
```

1번 구역 제목의 `{}`, `{:?}`는 서식 문자열 안에서 `{{}}`, `{{:?}}`로 써서 중괄호 자체를 출력한 것입니다.

### 05_trait_bounds — 트레이트 바운드

함수 일곱 개가 2.9절 표의 쓰는 법을 하나씩 보여 줍니다. `SocialPost`는 `Summary`와 `Display`를, `NewsArticle`은 `Summary`와 `Debug`를 구현했으므로 어느 함수에 어느 값을 넘길 수 있는지 비교해 보세요. 주석에 적힌 호출(`notify(&42)`, `notify_same(&article, &post)` 등)을 직접 넣어 보면 각각의 에러를 볼 수 있습니다.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 05_trait_bounds
```

```text
1. impl Trait 매개변수와 트레이트 바운드는 같은 뜻입니다
    속보! 러스트 1.98 출시 - 김기자
    속보! @ferris: 바운드를 배우는 중
2. impl Trait 두 개는 서로 다른 타입을 받습니다
    1) 러스트 1.98 출시 - 김기자
    2) @ferris: 바운드를 배우는 중
3. 같은 T를 쓰면 같은 타입만 받습니다
    같은 타입 둘: @ferris: 바운드를 배우는 중 / @crab: 저도요
4. +로 건 바운드는 모두 만족해야 합니다
    요약: @ferris: 바운드를 배우는 중
    표시: [ferris] 바운드를 배우는 중
5. where 절로 쓴 바운드
    [crab] 저도요 vs NewsArticle { headline: "러스트 1.98 출시", author: "김기자" }
6. 반환 위치의 impl Trait
    @bot: ferris님, 글 잘 봤습니다
```

### 06_conditional_and_blanket_impl — 조건부 구현과 포괄 구현

2번 구역 아래의 `tickets.cmp_display();` 주석을 풀면 2.11절의 E0599를 볼 수 있습니다. 3번 구역의 `bracketed()`는 `i32`, `f64`, `&str`, `Score` 어디에도 따로 구현하지 않았다는 점에 주목하세요.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 06_conditional_and_blanket_impl
```

```text
1. 바운드를 만족하는 Pair에는 cmp_display가 있습니다
    더 큰 쪽은 y = 7
    더 큰 쪽은 x = 사과
2. 바운드를 만족하지 않는 Pair에도 new는 있습니다
    좌석: 12, 30
3. 포괄 구현: Display를 구현한 모든 타입이 bracketed()를 얻습니다
    [42] [2.5] [러스트] [95점]
4. 표준 라이브러리도 포괄 구현을 씁니다: Display가 있으면 to_string()이 생깁니다
    "3", "80점"
```

### 07_longest_lifetime — 대여 검사기와 `longest`

1번 구역 아래의 `println!("{r}");` 주석을 풀면 2.12절의 E0597을 볼 수 있습니다. 3번 구역과 4번 구역은 모두 "결과를 쓰는 곳이 한 인자의 블록 밖"인 상황인데, 3번은 결과를 블록 안에서만 써서, 4번은 시그니처가 그 인자를 빌리지 않는다고 말해서 컴파일됩니다.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 07_longest_lifetime
```

```text
1. 참조는 가리키는 값이 살아 있는 동안에만 쓸 수 있습니다
    블록 안에서 r = 5
2. longest는 두 문자열 슬라이스 중 긴 쪽을 돌려줍니다
    더 긴 문자열: abcd
3. 'a는 두 인자가 모두 유효한 구간, 즉 더 짧은 쪽에 맞춰집니다
    블록 안에서: long string is long
4. 라이프타임 표기는 관계를 설명할 뿐, 값을 더 오래 살게 하지 않습니다
    본문: 내일은 휴강입니다
5. 함수 안에서 만든 값은 소유한 값으로 돌려줍니다
    **러스트**
```

`longest`는 `len()`으로 길이를 비교하므로 글자 수가 아니라 바이트 수를 비교합니다(ch8). 예제는 이 차이가 드러나지 않도록 영문 문자열을 썼습니다.

### 08_struct_elision_static — 구조체, 생략 규칙, `'static`

1번 구역에서 `find('.')`는 첫 `.`의 바이트 위치를 `Option<usize>`로 돌려줍니다. `.`은 1바이트 문자라 그 위치에서 잘라도 한글 글자 경계가 깨지지 않습니다(ch4). 3번 구역의 `알림:` 줄은 `announce_and_return_part` 안의 `println!`이 찍은 것입니다.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 08_struct_elision_static
```

```text
1. 참조 필드를 가진 구조체
    발췌: 나를 이스마엘이라 불러 다오
    ferris <ferris@example.com>
2. 생략 규칙 1·2: 입력 참조가 하나면 표기를 생략할 수 있습니다
    first_word = hello, first_word_explicit = hello
    split_at_space → (안녕, 러스트)
3. 생략 규칙 3: 메서드의 반환 참조에는 self의 라이프타임이 붙습니다
    level = 3
    알림: 발췌를 소개합니다
    돌려받은 부분: 나를 이스마엘이라 불러 다오
4. 'static: 프로그램이 끝날 때까지 유효한 참조
    매우 중요 / 문자열 리터럴은 모두 'static입니다
```

### 09_generics_bounds_lifetimes — 세 가지를 함께

1번 구역은 `T`만 바꿔 같은 함수를 세 번 부릅니다. 2번 구역의 `longer_display`는 더 "큰" 값이 아니라 표시했을 때 더 "긴" 값을 고르므로, `35`보다 한 글자 긴 `-15`가 선택됩니다. 3번 구역의 `"러스트"`는 한글 3글자 x 3바이트 = 9바이트라서 `"Rust"`(4바이트)보다 깁니다.

```sh
cargo run -p p0-ch10-generics-traits-lifetimes --example 09_generics_bounds_lifetimes
```

```text
1. 같은 함수에 T만 바꿔 넘깁니다
    알림! 오늘은 누군가의 생일입니다!
    더 긴 문자열: abcd
    알림! 2024
    더 긴 문자열: abcd
    알림! 섭씨 21도
    더 긴 문자열: abcd
2. 제네릭·바운드·라이프타임이 맞물린 함수
    정수: 12345
    온도: 섭씨 -15도
3. 반환된 참조는 두 인자가 모두 살아 있는 동안 쓸 수 있습니다
    블록 안에서: 러스트
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch10-generics-traits-lifetimes
rustc --edition 2024 compile_fail/e0369_largest_without_bound.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 코드의 설명은 `rustc --explain E0369`처럼 볼 수 있습니다.

### 4.1 바운드 없이 제네릭 값을 비교 — E0369

파일: [`compile_fail/e0369_largest_without_bound.rs`](compile_fail/e0369_largest_without_bound.rs)

```text
error[E0369]: binary operation `>` cannot be applied to type `&T`
  --> compile_fail/e0369_largest_without_bound.rs:12:17
   |
12 |         if item > largest {
   |            ---- ^ ------- &T
   |            |
   |            &T
   |
help: consider restricting type parameter `T` with trait `PartialOrd`
   |
 8 | fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
   |             ++++++++++++++++++++++
```

- **원인**: 본문은 `T`에 올 수 있는 모든 타입에 대해 올바라야 하는데, `<T>`만으로는 `T`가 비교 가능하다는 보장이 없습니다(2.1절).
- **해결**: `help:`대로 바운드를 답니다. `PartialOrd`는 프렐류드에 있으므로 `std::cmp::`를 빼고 `fn largest<T: PartialOrd>(list: &[T]) -> &T`로 써도 됩니다. 일반적으로 "제네릭 값에 어떤 연산이나 메서드를 쓰려면, 그 기능을 주는 트레이트를 바운드로 요구한다"고 기억하면 됩니다.

### 4.2 바운드를 만족하지 않는 타입을 넘김 — E0277

파일: [`compile_fail/e0277_trait_bound_not_satisfied.rs`](compile_fail/e0277_trait_bound_not_satisfied.rs)

```text
error[E0277]: the trait bound `Weather: Summary` is not satisfied
  --> compile_fail/e0277_trait_bound_not_satisfied.rs:44:12
   |
44 |     notify(&today); // Weather는 Summary를 구현하지 않았습니다.
   |     ------ ^^^^^^ unsatisfied trait bound
   |     |
   |     required by a bound introduced by this call
   |
help: the trait `Summary` is not implemented for `Weather`
  --> compile_fail/e0277_trait_bound_not_satisfied.rs:23:1
   |
23 | struct Weather {
   | ^^^^^^^^^^^^^^
help: the trait `Summary` is implemented for `SocialPost`
  --> compile_fail/e0277_trait_bound_not_satisfied.rs:17:1
   |
17 | impl Summary for SocialPost {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
note: required by a bound in `notify`
  --> compile_fail/e0277_trait_bound_not_satisfied.rs:28:23
   |
28 | fn notify(item: &impl Summary) {
   |                       ^^^^^^^ required by this bound in `notify`
```

- **읽는 법**: 첫 줄의 `` `Weather: Summary` ``는 "`Weather`가 `Summary`를 구현한다"는 조건이고, 그것이 만족되지 않았다는 뜻입니다. 마지막 `note:`가 그 조건이 어디서 왔는지(`notify`의 바운드)를 가리킵니다. 같은 E0277이 ch5에서 `Debug` 없이 `{:?}`를 쓸 때도 났습니다. `{:?}`도 결국 "`Debug`를 구현해야 한다"는 바운드였던 것입니다.
- **원인**: `notify`는 `Summary`를 구현한 타입만 받겠다고 약속했는데, `Weather`는 구현하지 않았습니다. 필드 모양이 비슷하거나 `summarize`라는 이름의 메서드를 따로 만들어 두었더라도, `impl Summary for Weather`가 없으면 `Summary`를 구현한 것이 아닙니다.
- **해결**: `impl Summary for Weather { fn summarize(&self) -> String { ... } }`를 작성합니다. `Weather`를 요약할 이유가 없다면 이 함수에 넘기지 않는 것이 맞습니다.

### 4.3 반환 참조의 라이프타임을 정할 수 없음 — E0106

파일: [`compile_fail/e0106_longest_missing_lifetime.rs`](compile_fail/e0106_longest_missing_lifetime.rs)

```text
error[E0106]: missing lifetime specifier
 --> compile_fail/e0106_longest_missing_lifetime.rs:9:33
  |
9 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
  |
9 | fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
  |           ++++     ++          ++          ++
```

- **원인**: 입력 참조가 둘이고 메서드도 아니므로 생략 규칙으로 출력 라이프타임이 정해지지 않습니다(2.15절). 메시지의 `----` 두 개가 후보인 두 입력을 가리킵니다.
- **해결**: 반환값이 실제로 어느 인자에서 오는지 보고 적습니다. `longest`는 둘 중 하나를 돌려주므로 `help:`대로 둘 다에 `'a`를 붙입니다. 언제나 `x`만 돌려주는 함수라면 `x`와 반환값에만 붙입니다(2.13절의 `strip_tag`).
- 같은 E0106이 ch4(함수 안 값의 참조 반환)와 ch5(참조 필드)에서도 났습니다. ch5의 경우는 2.14절처럼 구조체에 라이프타임 매개변수를 달고, ch4의 경우는 소유한 값을 반환해야 합니다(4.5절).

### 4.4 결과를 짧게 사는 인자보다 오래 사용 — E0597

파일: [`compile_fail/e0597_result_outlives_argument.rs`](compile_fail/e0597_result_outlives_argument.rs)

```text
error[E0597]: `string2` does not live long enough
  --> compile_fail/e0597_result_outlives_argument.rs:19:44
   |
18 |         let string2 = String::from("xyz");
   |             ------- binding `string2` declared here
19 |         result = longest(string1.as_str(), string2.as_str());
   |                                            ^^^^^^^ borrowed value does not live long enough
20 |     } // string2가 여기서 drop됩니다.
   |     - `string2` dropped here while still borrowed
21 |     println!("더 긴 문자열: {result}");
   |                              ------ borrow later used here
```

- **원인**: `longest`의 시그니처는 "결과는 두 인자가 모두 유효한 동안만 유효하다"고 말합니다. 21번 줄은 `string2`가 drop된 뒤에 `result`를 쓰므로 그 약속을 어깁니다. 실제로 `result`는 더 긴 `string1`을 가리키지만, 컴파일러는 본문이 아니라 시그니처를 보고 판단합니다(2.13절).
- **해결**: (가) `println!`을 안쪽 블록 안으로 옮기거나, (나) `string2`를 바깥 스코프에 선언해 `result`보다 오래 살게 합니다. 시그니처에 `'a`를 더 붙이거나 빼서 해결하려 하면 안 됩니다. 표기는 값의 수명을 바꾸지 못합니다.

### 4.5 함수 안에서 만든 값의 참조를 반환 — E0515

파일: [`compile_fail/e0515_return_reference_to_local.rs`](compile_fail/e0515_return_reference_to_local.rs)

```text
error[E0515]: cannot return value referencing local variable `result`
  --> compile_fail/e0515_return_reference_to_local.rs:11:5
   |
11 |     result.as_str()
   |     ------^^^^^^^^^
   |     |
   |     returns a value referencing data owned by the current function
   |     `result` is borrowed here
```

- **원인**: 라이프타임 `'a`를 적어 E0106은 피했지만, 반환하는 참조가 매개변수가 아니라 함수 안의 `result`를 가리킵니다. `result`는 함수가 끝날 때 drop되므로 이 참조는 댕글링 참조가 됩니다.
- **해결**: 소유한 값을 반환합니다. 시그니처를 `-> String`으로 바꾸고 `result`를 그대로 돌려주면, 소유권이 호출한 쪽으로 넘어가 아무것도 해제되지 않습니다. 반환하는 참조는 반드시 매개변수에서 빌린 것이어야 한다고 기억하세요(2.13절).

### 4.6 반환 위치 `impl Trait`에서 서로 다른 타입 반환 — E0308

파일: [`compile_fail/e0308_impl_trait_two_types.rs`](compile_fail/e0308_impl_trait_two_types.rs)

```text
error[E0308]: `if` and `else` have incompatible types
  --> compile_fail/e0308_impl_trait_two_types.rs:40:9
   |
35 | /       if switch {
36 | | /         NewsArticle {
37 | | |             headline: String::from("러스트 2024 에디션 공개"),
38 | | |         }
   | | |_________- expected because of this
39 | |       } else {
40 | | /         SocialPost {
41 | | |             username: String::from("ferris"),
42 | | |         }
   | | |_________^ expected `NewsArticle`, found `SocialPost`
43 | |       }
   | |_______- `if` and `else` have incompatible types
   |
help: you could change the return type to be a boxed trait object
   |
34 - fn returns_summarizable(switch: bool) -> impl Summary {
34 + fn returns_summarizable(switch: bool) -> Box<dyn Summary> {
   |
```

(이 아래에 `Box::new(...)`로 두 갈래를 감싸라는 `help:`가 더 나옵니다.)

- **원인**: `impl Summary`는 "구체 타입 하나"를 숨긴 것이므로, 첫 갈래가 `NewsArticle`로 타입을 정한 뒤 다른 갈래가 `SocialPost`를 돌려줄 수 없습니다(2.10절). 두 타입이 모두 `Summary`를 구현했다는 사실은 상관이 없습니다.
- **해결**: 한 가지 타입만 돌려주도록 설계를 바꿉니다. `help:`가 제안하는 `Box<dyn Summary>`는 실행 중에 여러 타입 중 하나를 고르는 트레이트 객체로, `Box`는 [ch14](../ch14-smart-pointers/README.md)에서, `dyn Trait`은 [ch17](../ch17-trait-objects-oop/README.md)에서 배웁니다.

## 5. 연습 문제

### 연습 1. 빈 목록에도 안전한 `largest`

[`01_generic_largest.rs`](examples/01_generic_largest.rs)의 `largest`는 빈 슬라이스를 받으면 패닉이 납니다. 시그니처를 `fn largest<T: PartialOrd>(list: &[T]) -> Option<&T>`로 바꿔, 빈 슬라이스면 `None`을 돌려주게 고치세요. `largest(&[34, 50, 25, 100, 65])`는 `Some(100)`, 빈 배열 `let empty: [i32; 0] = [];`에 대해서는 `None`이 `{:?}`로 출력되어야 합니다. 반환 타입에 라이프타임을 적어야 할까요?

<details>
<summary>힌트</summary>

슬라이스의 `first()` 메서드는 첫 원소의 참조를 `Option<&T>`로 돌려줍니다(비어 있으면 `None`). ch9의 `?`는 `Option`에도 쓸 수 있으므로 `let mut largest = list.first()?;`로 시작하면, 빈 슬라이스일 때 그 자리에서 `None`을 반환합니다. 반복문은 그대로 두고 마지막을 `Some(largest)`로 바꿉니다. 입력 참조가 `list` 하나뿐이므로 생략 규칙 2가 `Option<&T>` 안의 참조에도 적용되어 라이프타임을 적을 필요가 없습니다.

</details>

### 연습 2. 넓이 트레이트

다음을 작성하세요.

- 트레이트 `Area`: 반드시 구현해야 하는 `fn area(&self) -> f64`와, 기본 구현이 있는 `fn describe(&self) -> String`(기본 구현은 `"넓이 12.00"`처럼 `area()`를 소수점 아래 2자리로 표시)
- 구조체 `Rectangle { width: f64, height: f64 }`와 `Circle { radius: f64 }`에 `Area` 구현. `Circle`은 `describe`를 재정의해 `"반지름 1인 원, 넓이 3.14"`처럼 표시합니다.
- `Rectangle`에 `Display` 구현: `3x4 사각형`
- 함수 `total_area`: `Area`를 구현한 서로 다른 타입의 값 두 개를 받아 넓이의 합을 돌려줍니다.

`Rectangle { width: 3.0, height: 4.0 }`와 `Circle { radius: 1.0 }`으로 실행하면 다음과 같이 출력되어야 합니다.

```text
3x4 사각형: 넓이 12.00
반지름 1인 원, 넓이 3.14
합계: 15.14
```

<details>
<summary>힌트</summary>

기본 구현은 `format!("넓이 {:.2}", self.area())`입니다(`{:.2}`는 ch1). 원주율은 `std::f64::consts::PI`를 씁니다(`3.141592` 같은 근삿값을 직접 쓰면 Clippy의 `approx_constant`에 걸립니다). 두 인자가 서로 다른 타입이어야 하므로 `fn total_area(a: &impl Area, b: &impl Area) -> f64`처럼 `impl Trait`를 두 번 씁니다. `<T: Area>(a: &T, b: &T)`로 쓰면 `Rectangle`과 `Circle`을 함께 넘길 수 없습니다(2.9절). `Display`의 `f64` 출력은 `3.0`을 `3`으로 표시하므로 `write!(f, "{}x{} 사각형", self.width, self.height)`로 충분합니다.

</details>

### 연습 3. 라이프타임 표기가 필요한 시그니처 고르기

아래 다섯 함수 중 **컴파일되지 않는** 것을 생략 규칙(2.15절)으로 먼저 예측하고, 필요한 곳에만 표기를 추가해 고치세요.

```rust
fn trim_both(s: &str) -> &str {
    s.trim()
}

fn pick(first: &str, second: &str, use_first: bool) -> &str {
    if use_first { first } else { second }
}

fn prefix_of(text: &str, len: usize) -> &str {
    &text[..len]
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &str {
        &self.input[self.pos..]
    }
}

fn default_name() -> &str {
    "익명"
}
```

<details>
<summary>힌트</summary>

입력 참조의 개수를 셉니다. `trim_both`와 `prefix_of`는 참조 매개변수가 하나(`usize`는 참조가 아님)라서 규칙 2가, `rest`는 `&self`가 있어 규칙 3이 적용됩니다. 컴파일되지 않는 것은 `pick`(입력 참조 둘, 메서드 아님)과 `default_name`(입력 참조 없음)이고, 둘 다 E0106입니다. `pick`은 두 인자 중 하나를 돌려주므로 `fn pick<'a>(first: &'a str, second: &'a str, use_first: bool) -> &'a str`, `default_name`은 리터럴을 돌려주므로 `-> &'static str`로 고칩니다.

</details>

### 연습 4. 조건부로 생기는 `largest` 메서드

[`06_conditional_and_blanket_impl.rs`](examples/06_conditional_and_blanket_impl.rs)의 `Pair<T>`에, `T`가 `PartialOrd`를 구현할 때만 생기는 메서드 `largest(&self) -> &T`를 추가하세요. 두 필드 중 큰 쪽의 참조를 돌려줍니다. `Pair::new(3, 7).largest()`는 `7`, `Pair::new("사과", "바나나").largest()`는 `사과`여야 합니다. 그다음 `tickets.largest()`를 호출하면 어떤 에러가 나는지 예측하고 확인하세요.

<details>
<summary>힌트</summary>

새 블록 `impl<T: PartialOrd> Pair<T> { ... }`를 만듭니다. 본문은 `if self.x >= self.y { &self.x } else { &self.y }`입니다. `&self`가 있으므로 생략 규칙 3에 따라 반환 참조의 라이프타임은 적지 않아도 됩니다. 이 메서드는 `{}`로 출력하지 않으므로 `Display` 바운드는 필요 없습니다. `tickets.largest()`는 E0599(``the method `largest` exists for struct `Pair<Ticket>`, but its trait bounds were not satisfied``)입니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [10. Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html)
  - [10.1 Generic Data Types](https://doc.rust-lang.org/book/ch10-01-syntax.html)
  - [10.2 Defining Shared Behavior with Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
  - [10.3 Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
  - [Generics](https://doc.rust-lang.org/rust-by-example/generics.html): 하위 절 [Functions](https://doc.rust-lang.org/rust-by-example/generics/gen_fn.html), [Implementation](https://doc.rust-lang.org/rust-by-example/generics/impl.html), [Bounds](https://doc.rust-lang.org/rust-by-example/generics/bounds.html), [Multiple bounds](https://doc.rust-lang.org/rust-by-example/generics/multi_bounds.html), [Where clauses](https://doc.rust-lang.org/rust-by-example/generics/where.html)가 이 챕터의 2.1~2.4절, 2.9절에 해당합니다. Bounds 절의 `HasArea` 예제는 연습 2와 비슷합니다. 뒤쪽의 New Type Idiom, Associated items, Phantom type parameters는 [ch19](../ch19-advanced-features/README.md) 이후에 읽으면 됩니다.
  - [Traits](https://doc.rust-lang.org/rust-by-example/trait.html): 트레이트 정의, 기본 메서드, `Self`를 돌려주는 연관 함수. 하위 절 [Derive](https://doc.rust-lang.org/rust-by-example/trait/derive.html)는 2.8절의 derive 목록을, [impl Trait](https://doc.rust-lang.org/rust-by-example/trait/impl_trait.html)는 2.9~2.10절을 다룹니다. impl Trait 절의 예제는 반복자와 클로저([ch13](../ch13-closures-iterators/README.md))를 쓰므로 그 챕터 뒤에 다시 보면 좋습니다.
  - [Formatted print > Display](https://doc.rust-lang.org/rust-by-example/hello/print/print_display.html): `fmt::Display` 직접 구현과 `write!`. 끝의 활동 과제(`Complex` 출력)는 2.8절의 복습으로 좋습니다.
  - [Scoping rules > Lifetimes](https://doc.rust-lang.org/rust-by-example/scope/lifetime.html): 하위 절 [Explicit annotation](https://doc.rust-lang.org/rust-by-example/scope/lifetime/explicit.html), [Functions](https://doc.rust-lang.org/rust-by-example/scope/lifetime/fn.html), [Methods](https://doc.rust-lang.org/rust-by-example/scope/lifetime/methods.html), [Structs](https://doc.rust-lang.org/rust-by-example/scope/lifetime/struct.html), [Static](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html), [Elision](https://doc.rust-lang.org/rust-by-example/scope/lifetime/elision.html)이 2.12~2.17절에 해당합니다. Static 절에는 `T: 'static` 바운드와 `Box::leak`처럼 이 챕터에서 다루지 않은 내용도 있습니다.
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`14_generics`](https://github.com/rust-lang/rustlings/tree/main/exercises/14_generics): `generics1.rs`는 `Vec<T>`의 `T`를 직접 적는 문제이고(`into()`는 타입 변환 트레이트로, 지금은 "숫자를 더 큰 정수 타입으로 바꾼다" 정도로 보면 됩니다), `generics2.rs`는 `u32`만 담던 `Wrapper` 구조체를 제네릭으로 바꾸는 문제입니다.
  - [`15_traits`](https://github.com/rust-lang/rustlings/tree/main/exercises/15_traits): `traits1.rs`·`traits2.rs`는 `String`과 `Vec<String>`에 내 트레이트를 구현하고(2.7절의 고아 규칙이 허용하는 경우), `traits3.rs`는 기본 구현, `traits4.rs`·`traits5.rs`는 `impl Trait`와 `+`로 매개변수 타입을 채우는 문제입니다.
  - [`16_lifetimes`](https://github.com/rust-lang/rustlings/tree/main/exercises/16_lifetimes): `lifetimes1.rs`는 `longest`의 시그니처(4.3절), `lifetimes2.rs`는 한 줄을 옮겨 E0597 고치기(4.4절), `lifetimes3.rs`는 참조 필드 구조체(2.14절)입니다.
  - 대부분의 문제가 `#[cfg(test)]`와 `assert_eq!`로 된 테스트([ch11](../ch11-testing/README.md))로 채점합니다. 테스트 코드는 고치지 말고 "이 값들이 같아야 한다"는 조건으로 읽으면 됩니다.
- 에러 설명: `rustc --explain E0597`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
