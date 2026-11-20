# ch5. 구조체로 관련된 데이터 묶기

구조체(struct)는 서로 관련된 값 여러 개를 하나로 묶고, 묶음 전체와 각 값에 이름을 붙여 새 타입을 만드는 방법입니다. 객체 지향 언어를 써 보셨다면 클래스의 데이터 부분(필드)과 메서드를 떠올리면 됩니다. 다만 Rust에는 클래스와 상속이 없고, 데이터는 `struct`로, 동작은 `impl` 블록으로 따로 정의합니다.

ch4에서 drop 시점을 관찰하려고 만든 `Noisy`가 바로 구조체였고, ch1에서는 "직접 만든 타입을 `{:?}`로 출력하는 방법은 ch5에서 배운다"고 미뤄 두었습니다. 이 챕터에서 그 두 가지를 이어받아 정리합니다.

- 패키지: `p0-ch5-structs`
- 기준 문서: [The Book 5장 Using Structs to Structure Related Data](https://doc.rust-lang.org/book/ch05-00-structs.html)
- 확인한 툴체인: rustc 1.100.0, Edition 2024. 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 구조체를 정의하고 인스턴스를 만들어 필드를 읽고 바꿀 수 있다. 필드 초기화 축약과 구조체 갱신 문법을 쓰고, 갱신 문법에서 어떤 필드가 이동하는지 설명할 수 있다.
2. `#[derive(Debug)]`를 붙여 구조체를 `{:?}`, `{:#?}`, `dbg!`로 출력하고, `{}`로는 출력되지 않는 이유를 설명할 수 있다.
3. 튜플 구조체와 유닛 구조체를 정의하고, 필드 타입이 같아도 이름이 다른 구조체는 서로 다른 타입이라는 점을 활용할 수 있다.
4. 흩어진 값을 구조체와 메서드로 묶을 수 있고, 메서드의 첫 매개변수로 `&self`, `&mut self`, `self` 중 알맞은 것을 고를 수 있다. 메서드 호출 때 일어나는 자동 참조·역참조를 설명할 수 있다.
5. 연관 함수로 생성자(`new`)를 만들어 `타입::함수()`로 호출하고, `Self`와 여러 개의 `impl` 블록을 쓸 수 있다.

## 2. 핵심 개념

### 2.1 구조체 정의와 인스턴스

ch3에서 배운 튜플도 여러 값을 묶지만, 원소에 이름이 없어서 `.0`이 무엇이었는지 기억해야 합니다. 구조체는 각 값에 **필드(field)** 라는 이름을 붙입니다.

```rust
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}
```

정의는 "이런 모양의 값"이라는 틀이고, 실제 값은 **인스턴스(instance)** 라고 부릅니다. 인스턴스를 만들 때는 모든 필드에 `이름: 값`을 줍니다. 이름으로 짝을 맞추므로 순서는 정의와 달라도 됩니다. 필드를 하나라도 빠뜨리면 컴파일 에러입니다(``error[E0063]: missing field `active` in initializer of `User` ``). 초기화되지 않은 필드가 있는 값은 Rust에 존재할 수 없습니다.

```rust
let user1 = User {
    email: String::from("ferris@example.com"),
    username: String::from("ferris"),
    active: true,
    sign_in_count: 1,
};
println!("{}", user1.email); // 점(.)으로 필드를 읽습니다.
```

구조체 이름은 `UpperCamelCase`, 필드 이름은 `snake_case`로 짓는 것이 관례입니다. 함수가 구조체를 만들어 돌려줄 때는 다른 값과 마찬가지로 본문의 마지막 표현식에 구조체를 쓰면 됩니다.

### 2.2 가변성은 인스턴스 전체에 붙습니다

필드 하나를 바꾸려 해도 인스턴스를 담은 변수가 `mut`여야 합니다. Rust에는 "이 필드만 가변"이라는 표시가 없습니다.

```rust
let mut user2 = User { /* ... */ };
user2.email = String::from("new@example.com");
user2.sign_in_count += 1;
```

ch3에서 `mut`를 변수 선언에 붙였듯이, 가변성은 값이 아니라 **변수(바인딩)** 의 성질입니다. `let user1`은 "이 변수로는 값을 바꾸지 않겠다"는 약속일 뿐이므로, 소유권을 `let mut moved = user1;`처럼 `mut` 변수로 옮기면 새 소유자는 값을 바꿀 수 있습니다. 어느 쪽이든 "누가 이 값을 바꿀 수 있는가"가 변수 선언 한 곳에 드러나므로, 코드를 읽을 때 필드마다 가변 여부를 확인할 필요가 없습니다. 불변 변수의 필드를 바꾸려 하면 E0594로 거부됩니다(4.3절).

### 2.3 필드 초기화 축약

변수 이름과 필드 이름이 같으면 `email: email` 대신 `email`만 써도 됩니다. 이것을 **필드 초기화 축약(field init shorthand)** 이라고 부릅니다.

```rust
fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username, // username: username
        email,    // email: email
        sign_in_count: 1,
    }
}
```

축약은 함수 매개변수뿐 아니라 같은 이름의 지역 변수가 있으면 어디서든 쓸 수 있습니다. Rust 코드 검사 도구인 Clippy는 `email: email`처럼 같은 이름을 반복하면 축약하라고 경고합니다(`clippy::redundant_field_names`). [`01_define_and_instantiate.rs`](examples/01_define_and_instantiate.rs)는 축약 전의 모습을 비교하려고 일부러 반복하고, 그 경고를 `#[expect(...)]`로 표시해 두었습니다.

### 2.4 구조체 갱신 문법과 이동

기존 인스턴스에서 몇 개 필드만 바꾼 새 인스턴스를 만들 때는 **구조체 갱신 문법(struct update syntax)** `..기존_인스턴스`를 씁니다. 직접 적지 않은 나머지 필드를 그 인스턴스에서 가져옵니다.

```rust
let user2 = User {
    email: String::from("another@example.com"),
    ..user1 // 반드시 마지막에 쓰고, 뒤에 쉼표를 붙이지 않습니다.
};
```

`..user1`은 나머지 필드를 하나씩 `username: user1.username`처럼 **대입**하는 것과 같습니다. 그래서 ch4의 규칙이 필드마다 그대로 적용됩니다.

| `user1`의 필드 | `user2`가 가져갔나 | 타입 | 결과 | 이후 `user1.필드` |
| --- | --- | --- | --- | --- |
| `email` | 아니요 (새 값을 줌) | `String` | 그대로 남음 | 사용 가능 |
| `username` | 예 | `String` | **이동** | 사용 불가 (E0382) |
| `active`, `sign_in_count` | 예 | `bool`, `u64` | 복사 (`Copy`) | 사용 가능 |

이렇게 필드 일부만 빠져나간 상태를 **부분 이동(partial move)** 이라고 합니다. 남은 필드는 쓸 수 있지만, 빠진 필드가 있으므로 `user1` 전체를 출력하거나 함수에 넘길 수는 없습니다(4.2절). [`02_struct_update.rs`](examples/02_struct_update.rs)는 `user2.username`이 원래 `user1.username`과 같은 힙 주소를 가리키는 것(`true`)으로 이동을 확인합니다.

`String` 필드를 모두 새로 주면 원본에서 가져가는 것은 `Copy` 필드뿐이므로 원본이 온전히 남습니다. 원본의 `String`까지 가져오면서 원본도 계속 쓰고 싶다면 복제본에서 가져오면 됩니다(4.2절).

### 2.5 구조체는 자기 데이터를 소유합니다

`User`의 필드가 `&str`이 아니라 `String`인 것은 의도한 선택입니다. 인스턴스가 자기 데이터를 모두 **소유**하면, 인스턴스가 살아 있는 동안 데이터도 반드시 살아 있습니다. 인스턴스가 drop될 때 필드도 함께 drop됩니다.

필드에 참조를 넣을 수도 있지만, 그러려면 "이 구조체는 참조가 가리키는 값보다 오래 살 수 없다"는 관계를 **라이프타임(lifetime)** 으로 적어야 합니다. 적지 않으면 E0106으로 거부됩니다(4.5절). 그렇지 않으면 구조체가 남아 있는 동안 참조가 가리키던 값이 먼저 drop되어 ch4의 댕글링 참조가 생길 수 있기 때문입니다. 라이프타임은 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 배우고, 그전까지는 구조체 필드에 `String`처럼 데이터를 소유하는 타입을 씁니다.

### 2.6 구조체 출력하기: `derive(Debug)`, `{:?}`, `{:#?}`, `dbg!`

#### `{}`로는 출력할 수 없습니다

`{}`(기본 형식)는 사용자에게 보여 줄 모양입니다. 정수 `5`를 보여 주는 방법은 거의 하나뿐이지만, 사각형을 보여 주는 방법은 "30x50", "가로 30, 세로 50" 등 프로그램마다 다릅니다. 그래서 Rust는 구조체의 기본 형식을 대신 정해 주지 않습니다.

```text
error[E0277]: `Rectangle` doesn't implement `std::fmt::Display`
...
10 |     println!("{rect}");
   |               ^^^^^^ `Rectangle` cannot be formatted with the default formatter
   |
...
   = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
```

사용자에게 보여 줄 문장은 필드를 꺼내 직접 만듭니다. 기본 형식을 구조체에 직접 정의하는 방법(`Display` 구현)은 트레이트를 배운 뒤에 다룹니다.

#### `#[derive(Debug)]`로 디버그 형식 얻기

개발 중에 값을 확인하려는 용도라면 `{:?}`(디버그 형식)를 씁니다. 그런데 직접 만든 구조체는 처음에는 디버그 형식도 갖고 있지 않아서, 구조체 정의 위에 `#[derive(Debug)]`를 붙여야 합니다.

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}
```

`#[...]`는 바로 아래 항목에 붙는 **속성(attribute)** 이고, `derive`는 "이 기능의 구현을 컴파일러가 자동으로 만들어 달라"는 뜻입니다. `Debug`와 `Display`는 트레이트(trait)라고 부르는 "타입이 가진 기능"의 이름이며, 트레이트 자체는 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 배웁니다. 지금은 `#[derive(Debug)]`를 붙이면 `{:?}`를 쓸 수 있다고 기억하면 충분합니다. 필드의 타입도 모두 디버그 형식을 지원해야 derive할 수 있는데, 표준 라이브러리의 타입은 대부분 지원하고 직접 만든 구조체는 각각 derive하면 됩니다.

| 쓰는 법 | 결과 | 쓰임 |
| --- | --- | --- |
| `{p:?}` | `Player { name: "페리스", position: Point { x: 3, y: -2 }, hp: 100 }` | 한 줄로 빠르게 확인 |
| `{p:#?}` | 필드마다 줄을 바꾸고 들여 씀 | 필드가 많거나 구조체가 겹겹이 있을 때 |
| `dbg!(&p)` | 파일·줄·열, 식, `{:#?}` 모양의 값을 **표준 에러**로 출력 | 계산 중간값 확인 |

#### `dbg!` 매크로

`dbg!(식)`은 식과 그 값을 위치 정보와 함께 표준 에러로 출력하고, **값을 그대로 돌려줍니다.** 그래서 `hp: dbg!(player.hp + bonus)`처럼 계산 중간에 끼워 넣어도 결과가 바뀌지 않습니다.

```text
[part0-getting-started/ch5-structs/examples/03_debug_print.rs:63:13] player.hp + bonus = 120
```

`dbg!`는 인자의 소유권을 가져갔다가 돌려줍니다. `dbg!(player);`처럼 인스턴스를 값으로 넘기고 반환값을 받지 않으면 인스턴스가 이동해서 사라지므로, 확인만 할 때는 `dbg!(&player)`처럼 참조를 넘깁니다. `println!`은 표준 출력, `dbg!`는 표준 에러로 나가므로 프로그램의 정상 출력과 섞이지 않습니다. 위치의 파일 경로는 저장소 루트(워크스페이스)를 기준으로 표시되며, 챕터 디렉터리에서 실행해도 같습니다.

#### 주의: 디버그 형식으로만 출력한 필드는 "읽지 않은 필드"입니다

`{:?}`로 구조체를 출력해도 컴파일러는 그 필드를 **읽은 것으로 치지 않습니다.** 필드를 어디서도 직접 읽지 않으면 다음 경고가 납니다.

```text
warning: fields `x` and `y` are never read
...
  = note: `Point` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
```

자동으로 만든 디버그 출력은 "프로그램이 이 필드를 실제로 쓴다"는 증거가 되지 않기 때문입니다. 이 챕터의 예제들이 `{:?}` 출력과 별도로 필드를 직접 읽는 줄을 둔 이유입니다.

### 2.7 튜플 구조체와 유닛 구조체

**튜플 구조체(tuple struct)** 는 이름은 붙이되 필드는 타입만 적습니다. 튜플처럼 `.0`, `.1`로 필드에 접근합니다.

```rust
struct Celsius(f64);
struct Fahrenheit(f64);

fn to_fahrenheit(celsius: Celsius) -> Fahrenheit {
    Fahrenheit(celsius.0 * 9.0 / 5.0 + 32.0)
}
```

`Celsius`와 `Fahrenheit`는 둘 다 `f64` 하나를 감쌌지만 **서로 다른 타입**입니다. 섭씨와 화씨를 모두 맨 `f64`로 다뤘다면 화씨 값을 섭씨 자리에 넘기는 실수를 컴파일러가 잡을 수 없지만, 타입을 나누면 ``expected `Celsius`, found `Fahrenheit` ``(E0308)로 거부됩니다. 감싼다고 크기가 늘지는 않습니다. `size_of::<Celsius>()`는 `f64`와 같은 8바이트입니다.

필드 이름이 없어도 의미가 분명할 때(값 하나를 감싸 타입만 구분할 때, RGB처럼 순서가 관례로 정해진 값) 튜플 구조체가 어울립니다. 구조 분해할 때는 `let Color(r, g, b) = orange;`처럼 구조체 이름까지 적습니다.

**유닛 구조체(unit-like struct)** 는 필드가 하나도 없는 구조체입니다. 원소 없는 튜플 `()`(유닛, ch3)과 비슷하게 동작한다는 뜻에서 이렇게 부릅니다.

```rust
struct AlwaysEqual;          // 정의: 세미콜론으로 끝납니다.
let marker = AlwaysEqual;    // 생성: 이름만 씁니다.
```

크기가 0바이트이고 담는 데이터가 없습니다. 데이터 없이 타입에 기능(트레이트)만 붙이고 싶을 때 쓰는데, 그 쓰임은 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 보게 됩니다.

| 종류 | 정의 | 생성 | 필드 접근 |
| --- | --- | --- | --- |
| 이름 있는 필드 | `struct User { name: String }` | `User { name: ... }` | `user.name` |
| 튜플 구조체 | `struct Celsius(f64);` | `Celsius(36.5)` | `c.0` |
| 유닛 구조체 | `struct AlwaysEqual;` | `AlwaysEqual` | (필드 없음) |

### 2.8 사각형 넓이 프로그램 다듬기

[`05_rectangle_area.rs`](examples/05_rectangle_area.rs)는 같은 계산을 네 단계로 고쳐 씁니다. 결과는 매번 1500으로 같고, 달라지는 것은 **시그니처가 무엇을 말해 주는가**입니다.

| 단계 | 시그니처 | 무엇이 나아졌나 / 남은 문제 |
| --- | --- | --- |
| 1. 변수 두 개 | `fn area_from_values(width: u32, height: u32) -> u32` | 두 값이 한 사각형의 것이라는 사실이 코드에 없습니다. |
| 2. 튜플 | `fn area_from_tuple(dimensions: (u32, u32)) -> u32` | 하나로 묶였지만 `.0`이 가로인지 세로인지 기억해야 합니다. |
| 3. 구조체 | `fn area_from_struct(rectangle: &Rectangle) -> u32` | 묶음과 각 값에 이름이 생겼습니다. 읽기만 하므로 불변 참조로 빌립니다. |
| 4. 메서드 | `impl Rectangle { fn area(&self) -> u32 }` | 넓이 계산이 `Rectangle`의 기능으로 묶였습니다. `rect1.area()`로 호출합니다. |

3단계에서 `rectangle`은 참조인데도 `rectangle.width`처럼 점으로 필드를 읽습니다. 필드에 접근할 때도 Rust가 참조를 자동으로 따라가 주므로 `(*rectangle).width`라고 쓸 필요가 없습니다. 참조로 필드를 **읽기만** 하는 것은 이동이 아니므로 `main`은 `rect1`을 계속 쓸 수 있습니다.

3단계의 `area_from_struct`는 `Rectangle`에만 쓸 수 있는데도 `Rectangle`과 떨어진 곳에 있습니다. 4단계처럼 `impl Rectangle` 블록에 넣으면 "이 타입으로 할 수 있는 일"이 한곳에 모이고, 쓰는 쪽에서도 `rect1.`까지 입력하면 무엇을 할 수 있는지 찾기 쉽습니다.

### 2.9 메서드와 `self`

`impl 타입 { ... }` 블록 안에 정의한 함수 중 첫 매개변수가 `self`인 것을 **메서드(method)** 라고 부릅니다. `self`는 메서드를 호출한 인스턴스이고, `Self`는 `impl` 대상 타입(여기서는 `Rectangle`)의 별칭입니다.

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}
```

ch4의 `impl Drop for Noisy`도 `impl` 블록이었지만, 그것은 "`Drop`이라는 트레이트를 `Noisy`에 구현한다"는 다른 형태입니다(트레이트는 ch10, `Drop`은 ch14). 이 챕터의 `impl Rectangle`은 트레이트 없이 타입에 직접 함수를 붙이는 블록입니다.

첫 매개변수의 모양은 세 가지이고, ch4에서 배운 "빌리기 / 가변으로 빌리기 / 소유권 넘기기"와 정확히 대응합니다.

| 첫 매개변수 | 풀어 쓰면 | 인스턴스를 | 호출한 뒤 원래 변수 | 언제 쓰나 |
| --- | --- | --- | --- | --- |
| `&self` | `self: &Self` | 불변으로 빌림 | 그대로 사용 | 읽기만 할 때 (가장 흔함) |
| `&mut self` | `self: &mut Self` | 가변으로 빌림 | 그대로 사용 (바뀐 상태) | 필드를 바꿀 때. 변수가 `mut`여야 합니다. |
| `self` | `self: Self` | 소유권을 가져감 | 사용 불가 (이동) | 인스턴스를 다른 값으로 바꾸거나, 호출 뒤에 다시 쓰면 안 될 때 (드묾) |

**선택 기준은 "필요한 만큼만 요구한다"입니다.** 읽기만 하는 메서드가 `&mut self`를 요구하면 호출하는 쪽이 쓸데없이 `mut`를 붙여야 하고, 다른 불변 참조가 살아 있는 동안에는 호출할 수 없습니다(ch4의 대여 규칙). `self`를 요구하면 호출한 뒤 인스턴스를 잃습니다. 그래서 `&self`에서 시작해, 필드를 바꿔야 하면 `&mut self`, 인스턴스를 소비하는 것이 맞는 동작일 때만 `self`를 씁니다.

[`06_method_receivers.rs`](examples/06_method_receivers.rs)의 `Account`는 세 가지를 모두 씁니다. 잔액 조회 `balance(&self)`, 입금 `deposit(&mut self, amount)`, 해지 `close(self) -> String`입니다. 해지한 계좌로 다시 입금할 수 있으면 안 되므로 `close`는 `self`를 받아 계좌를 소비하고 영수증 문자열만 돌려줍니다. 해지 뒤에 `account.balance()`를 호출하면 컴파일러가 막습니다.

```text
error[E0382]: borrow of moved value: `account`
...
85 |     let receipt = account.close(); // account가 close의 self로 이동합니다.
   |                           ------- `account` moved due to this method call
...
note: `Account::close` takes ownership of the receiver `self`, which moves `account`
```

메서드 이름은 필드 이름과 같아도 됩니다. `account.balance()`처럼 괄호가 있으면 메서드, `account.balance`처럼 없으면 필드입니다. 필드 값을 돌려주기만 하는 메서드를 게터(getter)라고 부르며, 필드는 숨기고 읽기만 허용할 때 쓸모가 있습니다(공개 범위는 ch7에서 다룹니다).

### 2.10 자동 참조와 역참조

`account.deposit(100)`의 `deposit`은 `&mut self`를 받는데, 호출하는 쪽에는 `&mut`가 보이지 않습니다. 점(.)으로 메서드를 호출하면 **Rust가 메서드의 `self` 모양을 보고 `&`, `&mut`, `*`를 알아서 붙이기** 때문입니다. 메서드는 `타입::메서드(인스턴스, 인자)` 형태로도 부를 수 있는데, 이렇게 쓰면 실제로 무엇이 넘어가는지 보입니다.

| 점 표기 | 실제 호출 | 일어나는 일 |
| --- | --- | --- |
| `account.balance()` | `Account::balance(&account)` | `&`를 붙임 |
| `account.deposit(100)` | `Account::deposit(&mut account, 100)` | `&mut`를 붙임 |
| `account.close()` | `Account::close(account)` | 그대로 넘김 (이동) |
| `r.balance()` (`r: &mut Account`) | `Account::balance(&*r)` | `*`로 따라간 뒤 `&`를 붙임 |
| `shared.balance()` (`shared: &Account`) | `Account::balance(shared)` | 이미 원하는 타입이라 그대로 넘김 |

C/C++에서는 값이면 `.`, 포인터면 `->`로 구분해 써야 하지만, Rust는 점 하나로 충분합니다. ch4에서 참조로 받은 `some_string.push_str(...)`에 `*`를 붙이지 않았던 것도 이 기능 덕분입니다. 메서드의 `self` 모양이 "어떻게 빌리는지"를 이미 정확히 말해 주므로, 호출할 때마다 반복해서 적을 필요가 없는 것입니다.

자동으로 붙여 줄 뿐, 규칙이 느슨해지는 것은 아닙니다. `&mut`가 자동으로 붙어도 변수가 `mut`가 아니면 E0596이 나고(4.4절), `self` 메서드는 여전히 인스턴스를 이동시킵니다.

### 2.11 연관 함수와 `Self`

`impl` 블록 안의 함수를 모두 **연관 함수(associated function)** 라고 부르고, 그중 `self`를 받는 것이 메서드입니다. `self`가 없는 연관 함수는 인스턴스 없이 `타입::함수()`로 부릅니다. 지금까지 써 온 `String::from("...")`이 바로 `String` 타입의 연관 함수입니다.

```rust
impl Rectangle {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    fn square(size: u32) -> Self {
        Self { width: size, height: size }
    }
}

let rect = Rectangle::new(30, 50);
let sq = Rectangle::square(3);
```

- **`Self`**: `impl` 대상 타입의 별칭입니다. 반환 타입과 구조체를 만드는 식, `Self::new(...)` 같은 호출에 모두 쓸 수 있고, 나중에 타입 이름을 바꿔도 `impl` 안은 고칠 필요가 없습니다.
- **생성자 관례 `new`**: Rust에는 생성자 문법이 따로 없습니다. 인스턴스를 만드는 방법은 결국 `Rectangle { ... }` 구조체 식 하나이고, `new`는 그것을 감싼 평범한 연관 함수에 관례로 붙이는 이름일 뿐입니다. 그래서 만드는 방법이 여러 가지라면 `square`처럼 이름을 달리해 여러 개 둘 수 있습니다.

`self`가 없는 함수를 점으로 부르면 컴파일러가 차이를 알려 줍니다.

```text
error[E0599]: no method named `square` found for struct `Rectangle` in the current scope
...
63 |     let wrong = sq.square(4);
   |                    ^^^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
```

### 2.12 `impl` 블록 여러 개

한 타입에 `impl` 블록을 여러 개 둘 수 있고, 모두 합쳐 하나의 타입 기능이 됩니다. [`07_associated_functions.rs`](examples/07_associated_functions.rs)는 생성자(`new`, `square`)와 메서드(`area`, `is_square`, `scaled`)를 두 블록으로 나눴지만, 쓰는 쪽에서는 어느 블록에 있는지 구분되지 않습니다. 지금은 나눌 이유가 크지 않지만, 제네릭과 트레이트를 배우는 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 블록을 나누는 것이 필요한 경우를 만나게 됩니다.

## 3. 예제 안내

모든 명령은 저장소 루트에서 실행합니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_define_and_instantiate.rs`](examples/01_define_and_instantiate.rs) | 정의, 인스턴스 생성, 필드 읽기·쓰기, 인스턴스 단위 가변성, 필드 초기화 축약 | 1 |
| [`02_struct_update.rs`](examples/02_struct_update.rs) | 구조체 갱신 문법과 필드 단위의 이동·복사, 부분 이동 | 1 |
| [`03_debug_print.rs`](examples/03_debug_print.rs) | `#[derive(Debug)]`, `{:?}`, `{:#?}`, `dbg!` | 2 |
| [`04_tuple_and_unit_structs.rs`](examples/04_tuple_and_unit_structs.rs) | 튜플 구조체, 구조 분해, 유닛 구조체, 크기 비교 | 3 |
| [`05_rectangle_area.rs`](examples/05_rectangle_area.rs) | 변수 → 튜플 → 구조체 → 메서드로 다듬는 사각형 넓이 프로그램 | 4 |
| [`06_method_receivers.rs`](examples/06_method_receivers.rs) | `&self`/`&mut self`/`self`, 게터, 자동 참조·역참조 | 4 |
| [`07_associated_functions.rs`](examples/07_associated_functions.rs) | 연관 함수, `new` 관례, `Self`, 여러 `impl` 블록 | 5 |
| [`compile_fail/`](compile_fail/) | 컴파일러가 거부하는 대표 코드 5개 (4절) | 1, 2, 4 |

### 01_define_and_instantiate — 정의와 인스턴스

3번 구역이 핵심입니다. 불변으로 만든 `user1`도 `mut` 변수로 옮기면 바꿀 수 있습니다. 가변성이 값이 아니라 변수에 붙는다는 뜻입니다.

```sh
cargo run -p p0-ch5-structs --example 01_define_and_instantiate
```

```text
1. 인스턴스를 만들고 점(.)으로 필드를 읽습니다
    ferris <ferris@example.com>, 활성: true, 로그인 1회
2. 필드를 바꾸려면 인스턴스 전체를 mut로 선언합니다
    crab <new@example.com>, 로그인 2회
3. 불변 인스턴스도 mut 변수로 옮기면 바꿀 수 있습니다
    ferris, 활성: false
4. 함수가 구조체를 만들어 돌려줄 수 있습니다
    verbose <verbose@example.com>
5. 필드 초기화 축약을 써도 결과는 같습니다
    short <short@example.com>, 활성: true, 로그인 1회
    local <local@example.com>, 활성: false, 로그인 0회
```

### 02_struct_update — 구조체 갱신 문법

2번 구역의 `true`는 `username`의 글자가 복사되지 않고 소유자만 `user2`로 바뀌었다는 뜻입니다(주소 값 자체는 실행마다 달라서 출력하지 않습니다). 3번 구역 아래에 주석 처리한 두 줄의 주석을 하나씩 풀어 보면, 이동한 필드를 쓸 때와 부분 이동한 인스턴스 전체를 쓸 때의 E0382를 각각 볼 수 있습니다.

```sh
cargo run -p p0-ch5-structs --example 02_struct_update
```

```text
1. ..user1은 직접 적지 않은 나머지 필드를 user1에서 가져옵니다
    user2: ferris <another@example.com>, 활성: true, 로그인 3회
2. 가져온 String 필드는 복사가 아니라 이동합니다
    user2.username이 원래 user1.username의 힙 데이터를 가리키는가? true
3. 이동하지 않은 필드는 user1에서 계속 쓸 수 있습니다
    user1.email = ferris@example.com
    user1.active = true, user1.sign_in_count = 3
4. String 필드를 모두 새로 주면 원본은 온전히 남습니다
    user3: crab <crab@example.com>, 활성: false, 로그인 10회
    base:  base <base@example.com>, 활성: false, 로그인 10회
```

### 03_debug_print — 디버그 형식 출력

4번 구역의 `[...]`로 시작하는 줄은 `dbg!`가 표준 에러로 출력한 것입니다. 터미널에서는 표준 출력과 섞여 순서대로 보입니다.

```sh
cargo run -p p0-ch5-structs --example 03_debug_print
```

```text
1. {}(기본 형식)에는 구조체를 바로 넣을 수 없어 필드를 하나씩 꺼내 씁니다
    페리스의 위치는 (3, -2), 체력은 100입니다
2. {:?}는 구조체 이름과 필드를 한 줄로 보여 줍니다
    Player { name: "페리스", position: Point { x: 3, y: -2 }, hp: 100 }
3. {:#?}는 필드마다 줄을 바꿔 들여 씁니다
Player {
    name: "페리스",
    position: Point {
        x: 3,
        y: -2,
    },
    hp: 100,
}
4. dbg!는 파일 위치, 식, 값을 표준 에러로 출력하고 값을 그대로 돌려줍니다
[part0-getting-started/ch5-structs/examples/03_debug_print.rs:63:13] player.hp + bonus = 120
[part0-getting-started/ch5-structs/examples/03_debug_print.rs:67:5] &boosted = Player {
    name: "강해진 페리스",
    position: Point {
        x: 0,
        y: 0,
    },
    hp: 120,
}
    강해진 페리스의 체력: 120
```

표준 에러를 버리고 실행하면(`2>/dev/null`) `[...]`로 시작하는 줄이 모두 사라지고, 4번 구역에는 마지막 줄만 남습니다.

```sh
cargo run -p p0-ch5-structs --example 03_debug_print 2>/dev/null
```

```text
...
4. dbg!는 파일 위치, 식, 값을 표준 에러로 출력하고 값을 그대로 돌려줍니다
    강해진 페리스의 체력: 120
```

### 04_tuple_and_unit_structs — 튜플 구조체와 유닛 구조체

2번 구역의 주석 처리된 `to_fahrenheit(converted)`의 주석을 풀면 E0308(``expected `Celsius`, found `Fahrenheit` ``)을 볼 수 있습니다. 5번 구역은 이름을 붙여 감싸도 크기가 늘지 않는다는 것을 보여 줍니다. `Color`는 `u8` 세 개라서 3바이트입니다.

```sh
cargo run -p p0-ch5-structs --example 04_tuple_and_unit_structs
```

```text
1. 튜플 구조체는 이름(타입)이 붙은 튜플입니다
    body.0 = 36.5, Celsius(36.5)
2. 필드 타입이 같아도 구조체 이름이 다르면 다른 타입입니다
    converted.0 = 97.7, Fahrenheit(97.7)
3. 구조 분해할 때는 구조체 이름까지 적습니다
    r = 255, g = 165, b = 0
    Color(255, 165, 0)
4. 유닛 구조체는 필드 없이 이름만 있습니다
    AlwaysEqual
5. 이름을 붙여 감싸도 크기는 늘지 않습니다
    f64: 8바이트
    Celsius: 8바이트
    Color: 3바이트
    AlwaysEqual: 0바이트
```

### 05_rectangle_area — 사각형 넓이 프로그램

파일 위쪽에 1~4단계의 함수가 차례로 정의되어 있습니다. 출력보다는 단계마다 시그니처가 어떻게 바뀌는지를 코드에서 비교해 보세요(2.8절).

```sh
cargo run -p p0-ch5-structs --example 05_rectangle_area
```

```text
1단계: 변수 두 개
    넓이: 1500
2단계: 튜플
    넓이: 1500
3단계: 구조체
    넓이: 1500
    rect1 = Rectangle { width: 30, height: 50 }
4단계: 메서드
    넓이: 1500
    rect1에 rect2가 들어가는가? true
    rect1에 rect3가 들어가는가? false
```

### 06_method_receivers — `self`의 세 가지 모양

3번 구역은 점 표기와 `타입::메서드(...)` 표기가 같은 결과를 낸다는 것을, 4번 구역은 참조를 통해 호출해도 `*`를 쓰지 않는다는 것을 보여 줍니다. 마지막 줄의 주석을 풀면 `close` 뒤에 `account`를 쓸 수 없다는 E0382를 볼 수 있습니다.

```sh
cargo run -p p0-ch5-structs --example 06_method_receivers
```

```text
1. &self 메서드는 읽기만 합니다
    메서드: 1000, 필드: 1000
2. &mut self 메서드는 인스턴스를 바꿉니다
    Account { owner: "페리스", balance: 1500 }
3. 점(.)으로 호출하면 Rust가 &나 &mut를 자동으로 붙입니다
    Account::balance(&account) = 1600, account.balance() = 1600
4. 참조를 통해 호출할 때도 *를 쓰지 않습니다
    이자 지급 후: 1760
5. self 메서드는 인스턴스의 소유권을 가져갑니다
    페리스님 계좌 해지, 1760원 반환
```

### 07_associated_functions — 연관 함수와 여러 `impl` 블록

`scaled`는 `&self`만 빌리고 결과를 새 인스턴스로 돌려주므로 `rect`가 그대로 남습니다. 2번 구역의 주석 처리된 `sq.square(4)`의 주석을 풀면 E0599를 볼 수 있습니다.

```sh
cargo run -p p0-ch5-structs --example 07_associated_functions
```

```text
1. 연관 함수는 `타입::함수()`로 부릅니다
    rect = Rectangle { width: 30, height: 50 }
2. 생성자 역할의 연관 함수를 여러 개 둘 수 있습니다
    sq = Rectangle { width: 3, height: 3 }, 정사각형인가? true
3. 메서드가 Self::new로 새 인스턴스를 만들어 돌려줄 수 있습니다
    big = Rectangle { width: 60, height: 100 }
    rect는 그대로: Rectangle { width: 30, height: 50 }
4. 서로 다른 impl 블록의 함수도 같은 타입의 것입니다
    넓이: rect = 1500, sq = 9, big = 6000
```

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다.

```sh
cd part0-getting-started/ch5-structs
rustc --edition 2024 compile_fail/e0277_debug_not_derived.rs
```

컴파일에 실패하면 아무 파일도 생기지 않습니다. 코드를 고쳐서 컴파일에 성공하면 현재 디렉터리에 실행 파일이 생기므로, 확인한 뒤 지우거나 `--out-dir`로 다른 디렉터리를 지정하세요. 에러 코드의 설명은 `rustc --explain E0277`처럼 볼 수 있습니다.

### 4.1 `Debug` 없이 `{:?}`로 출력 — E0277

파일: [`compile_fail/e0277_debug_not_derived.rs`](compile_fail/e0277_debug_not_derived.rs)

```text
error[E0277]: `Rectangle` doesn't implement `Debug`
  --> compile_fail/e0277_debug_not_derived.rs:19:22
   |
19 |     println!("rect = {rect:?}");
   |                      ^^^^^^^^ `Rectangle` cannot be formatted using `{:?}` because it doesn't implement `Debug`
   |
   = help: the trait `Debug` is not implemented for `Rectangle`
help: consider annotating `Rectangle` with `#[derive(Debug)]`
   |
 7 + #[derive(Debug)]
 8 | struct Rectangle {
   |
```

- **읽는 법**: "`Rectangle`은 `Debug`를 구현하지 않았다"는 첫 줄이 원인이고, 마지막 `help:`가 해결책입니다. `+`가 붙은 줄은 "여기에 이 줄을 추가하라"는 뜻입니다.
- **원인**: `{:?}`는 디버그 형식 기능(`Debug`)을 가진 타입만 출력할 수 있는데, 직접 만든 구조체는 처음에는 이 기능이 없습니다.
- **해결**: `help:`대로 구조체 정의 바로 위에 `#[derive(Debug)]`를 붙입니다.
- 같은 E0277이 `{}`로 구조체를 출력할 때도 납니다. 메시지는 ``doesn't implement `std::fmt::Display` ``이고, `#[derive(Display)]`라는 것은 없습니다. 개발 중 확인이 목적이면 `{:?}`로 바꾸고, 사용자에게 보여 줄 출력이면 필드를 꺼내 직접 문장을 만듭니다(2.6절).

### 4.2 갱신 문법으로 이동한 뒤 원본 전체를 사용 — E0382

파일: [`compile_fail/e0382_use_after_struct_update.rs`](compile_fail/e0382_use_after_struct_update.rs)

```text
error[E0382]: borrow of partially moved value: `user1`
  --> compile_fail/e0382_use_after_struct_update.rs:30:16
   |
24 |       let user2 = User {
   |  _________________-
25 | |         email: String::from("another@example.com"),
26 | |         ..user1
27 | |     };
   | |_____- value partially moved here
...
30 |       println!("{user1:?}");
   |                  ^^^^^ value borrowed here after partial move
   |
   = note: partial move occurs because `user1.username` has type `String`, which does not implement the `Copy` trait
```

- **읽는 법**: `partially moved`는 부분 이동, 즉 일부 필드만 빠져나갔다는 뜻입니다. `note:`가 빠져나간 필드(`user1.username`)와 이유(`String`은 `Copy`가 아님)를 정확히 짚어 줍니다.
- **원인**: `..user1`이 `username`을 `user2`로 옮겼습니다. `user1`을 통째로 출력하려면 모든 필드가 있어야 하는데 `username`이 없습니다.
- **해결**: 상황에 따라 고릅니다.
  1. 원본에서는 남은 필드만 씁니다. `user1.email`, `user1.active`는 여전히 쓸 수 있습니다.
  2. `username`에도 새 값을 줘서 원본에서 `String`을 가져가지 않게 합니다(2.4절).
  3. 원본이 통째로 필요하면 복제본에서 가져옵니다. 구조체에 `#[derive(Debug, Clone)]`처럼 `Clone`도 derive하면 `.clone()`을 쓸 수 있으므로 `..user1.clone()`으로 씁니다. 다만 ch4에서 본 것처럼 힙 데이터를 통째로 복사하는 비용이 듭니다.

### 4.3 불변 인스턴스의 필드 수정 — E0594

파일: [`compile_fail/e0594_assign_field_of_immutable.rs`](compile_fail/e0594_assign_field_of_immutable.rs)

```text
error[E0594]: cannot assign to `user.email`, as `user` is not declared as mutable
  --> compile_fail/e0594_assign_field_of_immutable.rs:18:5
   |
18 |     user.email = String::from("new@example.com");
   |     ^^^^^^^^^^ cannot assign
   |
help: consider changing this to be mutable
   |
13 |     let mut user = User {
   |         +++
```

- **원인**: 메시지가 필드(`user.email`)가 아니라 변수(`user`)가 `mut`가 아니라고 말하는 것에 주목하세요. 가변성은 인스턴스 전체, 정확히는 인스턴스를 담은 변수에 붙습니다(2.2절).
- **해결**: `help:`대로 `let mut user`로 선언합니다. 필드 정의에 `mut`를 붙이는 문법은 없습니다.

### 4.4 불변 인스턴스에서 `&mut self` 메서드 호출 — E0596

파일: [`compile_fail/e0596_mut_method_on_immutable.rs`](compile_fail/e0596_mut_method_on_immutable.rs)

```text
error[E0596]: cannot borrow `account` as mutable, as it is not declared as mutable
  --> compile_fail/e0596_mut_method_on_immutable.rs:20:5
   |
20 |     account.deposit(500);
   |     ^^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
18 |     let mut account = Account { balance: 1000 };
   |         +++
```

- **원인**: 코드에는 `&mut`가 보이지 않지만, `deposit`이 `&mut self`를 받으므로 `account.deposit(500)`은 `Account::deposit(&mut account, 500)`입니다(2.10절). 불변 변수는 가변으로 빌려줄 수 없습니다.
- **해결**: `let mut account`로 선언합니다. 만약 메서드가 사실은 필드를 바꾸지 않는데 `&mut self`로 잘못 선언된 것이라면, 반대로 메서드를 `&self`로 고치는 것이 맞습니다(2.9절의 선택 기준).

### 4.5 참조 필드에 라이프타임이 없음 — E0106

파일: [`compile_fail/e0106_reference_field_without_lifetime.rs`](compile_fail/e0106_reference_field_without_lifetime.rs)

```text
error[E0106]: missing lifetime specifier
  --> compile_fail/e0106_reference_field_without_lifetime.rs:10:15
   |
10 |     username: &str,
   |               ^ expected named lifetime parameter
   |
help: consider introducing a named lifetime parameter
   |
 8 ~ struct User<'a> {
 9 |     active: bool,
10 ~     username: &'a str,
   |
```

`email: &str`에 대해서도 같은 에러가 한 번 더 나옵니다.

- **원인**: 참조 필드를 가진 구조체는 참조가 가리키는 값보다 오래 살면 안 됩니다. 컴파일러가 이 관계를 확인하려면 라이프타임 표기(`'a`)가 필요합니다(2.5절).
- **해결**: 지금 단계에서는 필드를 `String`으로 바꾸고, 인스턴스를 만들 때 `String::from("ferris")`처럼 소유한 값을 넣습니다. `help:`가 제안하는 `User<'a>`, `&'a str` 방식은 [ch10](../ch10-generics-traits-lifetimes/README.md)에서 배웁니다.

## 5. 연습 문제

### 연습 1. 사각형에 메서드 두 개 추가하기

[`05_rectangle_area.rs`](examples/05_rectangle_area.rs)의 `impl Rectangle`에 다음 두 메서드를 추가하세요. 각각 첫 매개변수로 `&self`, `&mut self`, `self` 중 무엇을 받아야 하는지 먼저 정하고 이유를 설명해 보세요.

- `perimeter`: 둘레(가로 x 2 + 세로 x 2)를 돌려줍니다. `rect1`(30 x 50)이면 `160`입니다.
- `rotate`: 가로와 세로를 맞바꿉니다. 호출한 뒤 `{rect1:?}`는 `Rectangle { width: 50, height: 30 }`이어야 합니다.

<details>
<summary>힌트</summary>

`perimeter`는 읽기만 하므로 `&self`, `rotate`는 필드를 바꾸므로 `&mut self`입니다. `rotate`를 호출하려면 `main`의 `rect1`을 `let mut rect1`로 바꿔야 합니다. 두 필드를 맞바꿀 때는 임시 변수를 써도 되고, ch3의 피보나치 예제처럼 튜플로 한꺼번에 대입해도 됩니다: `(self.width, self.height) = (self.height, self.width);`

</details>

### 연습 2. 어느 줄이 컴파일될까

아래 코드에서 `println!` 세 줄 중 컴파일 에러가 나는 줄은 어느 것일까요? 컴파일하기 **전에** 예측하고, 이유를 2.4절의 표처럼 필드별로 설명해 보세요.

```rust
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

fn main() {
    let a = User {
        active: true,
        username: String::from("a"),
        email: String::from("a@example.com"),
        sign_in_count: 1,
    };
    let b = User { username: String::from("b"), ..a };
    println!("{}", a.username);
    println!("{}", a.email);
    println!("{}", a.active);
    println!("{} {} {} {}", b.username, b.email, b.active, b.sign_in_count);
}
```

<details>
<summary>힌트</summary>

이번에는 `username`에 새 값을 줬으므로 `a`에서 가져가는 필드는 `email`, `active`, `sign_in_count`입니다. 그중 `Copy`가 아닌 것은 `email`뿐입니다. 따라서 `a.email`을 출력하는 줄만 ``error[E0382]: borrow of moved value: `a.email` ``로 거부되고, `a.username`(가져가지 않음)과 `a.active`(복사됨)는 문제없습니다.

</details>

### 연습 3. 생성자와 출금 메서드

[`06_method_receivers.rs`](examples/06_method_receivers.rs)의 `Account`에 다음을 추가하세요.

- 연관 함수 `new(owner: &str) -> Self`: 잔액 0인 계좌를 만듭니다.
- 메서드 `withdraw(...) -> bool`: 잔액이 충분하면 금액만큼 빼고 `true`를, 모자라면 잔액을 바꾸지 않고 `false`를 돌려줍니다.

`Account::new("페리스")`로 만든 계좌에 1000을 입금한 뒤 300을 출금하면 `true`, 이어서 5000을 출금하면 `false`이고, 마지막 잔액은 700이어야 합니다.

<details>
<summary>힌트</summary>

`new`는 `self`가 없는 연관 함수이고, 매개변수 `owner`는 `&str`이므로 `String::from(owner)`로 소유한 `String`을 만들어 필드에 넣습니다. 본문은 `Self { owner: String::from(owner), balance: 0 }`입니다. `withdraw`는 잔액을 바꿀 수 있으므로 `&mut self`와 금액을 받습니다. `if amount > self.balance { return false; }`로 먼저 걸러 내면, `u64`에서 큰 수를 빼서 생기는 오버플로(ch3)도 함께 막을 수 있습니다.

</details>

### 연습 4. `dbg!`로 버그 찾기

누군가 `can_hold`를 다음처럼 작성했더니, 30 x 50인 `rect1`에 60 x 45인 `rect3`가 "들어간다"(`true`)는 잘못된 답이 나옵니다. 비교식을 고치기 전에, `dbg!`를 써서 두 비교 각각의 값을 출력해 원인을 찾아보세요.

```rust
fn can_hold(&self, other: &Rectangle) -> bool {
    self.width > other.width || self.height > other.height
}
```

<details>
<summary>힌트</summary>

`dbg!`는 값을 그대로 돌려주므로 식의 일부를 감싸도 됩니다: `dbg!(self.width > other.width) || dbg!(self.height > other.height)`. 실행하면 표준 에러에 `self.width > other.width = false`, `self.height > other.height = true`가 찍힙니다. 가로는 들어가지 않는데 세로가 들어간다는 이유만으로 `true`가 된 것입니다. "둘 다" 들어가야 하므로 `||`(또는)를 `&&`(그리고)로 고칩니다. 확인이 끝나면 `dbg!`는 지웁니다.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [5. Using Structs to Structure Related Data](https://doc.rust-lang.org/book/ch05-00-structs.html)
  - [5.1 Defining and Instantiating Structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html)
  - [5.2 An Example Program Using Structs](https://doc.rust-lang.org/book/ch05-02-example-structs.html)
  - [5.3 Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
  - [Custom Types > Structures](https://doc.rust-lang.org/rust-by-example/custom_types/structs.html): 세 가지 구조체, 필드 초기화 축약, 갱신 문법, 구조 분해. 맨 위의 `#![allow(dead_code)]`는 2.6절의 "읽지 않은 필드" 같은 경고를 파일 전체에서 끄는 속성입니다. `let Point { x: left_edge, y: top_edge } = point;`는 이름 있는 필드의 구조 분해로, 패턴은 [ch18](../ch18-patterns/README.md)에서 자세히 다룹니다. 끝의 활동 과제(`rect_area`, `square` 함수 만들기)는 이 챕터의 복습으로 좋습니다.
  - [Functions > Methods](https://doc.rust-lang.org/rust-by-example/fn/methods.html): 연관 함수와 `&self`/`&mut self`/`self` 메서드. `Pair(Box<i32>, Box<i32>)`가 나오는데, `Box<i32>`는 `String`처럼 힙 데이터를 소유하는 타입(ch14)으로 보고 읽으면 됩니다.
  - [Formatted print > Debug](https://doc.rust-lang.org/rust-by-example/hello/print/print_debug.html): `derive(Debug)`와 `{:#?}`. 마지막 예제의 `Person<'a>`와 `&'a str`가 4.5절에서 미뤄 둔 라이프타임 표기입니다(ch10).
- Rustlings [`07_structs`](https://github.com/rust-lang/rustlings/tree/main/exercises/07_structs) (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - `structs1.rs`: 이름 있는 필드의 구조체, 튜플 구조체, 유닛 구조체 만들기
  - `structs2.rs`: 구조체 갱신 문법으로 주문서 템플릿에서 새 주문 만들기
  - `structs3.rs`: 따로 떨어진 함수를 연관 함수(`new`)와 메서드로 옮기기. `#![deny(clippy::use_self)]`가 걸려 있어 `Self`를 쓰도록 연습시킵니다.
  - 세 문제 모두 `#[cfg(test)]`와 `assert_eq!`로 된 테스트(ch11)로 채점합니다. 테스트 코드는 고치지 말고, "이 값들이 같아야 한다"는 조건으로 읽으면 됩니다.
- 에러 설명: `rustc --explain E0594`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
