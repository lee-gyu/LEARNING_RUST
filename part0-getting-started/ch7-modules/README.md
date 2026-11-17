# ch7. 패키지, 크레이트, 모듈

프로그램이 커지면 코드를 나누고, 묶고, 이름을 붙여 정리해야 합니다. 이 챕터에서는 Rust의 모듈 시스템(module system)을 이루는 패키지(package), 크레이트(crate), 모듈(module), 경로(path)와 키워드 `pub`, `use`를 다룹니다. 앞부분은 파일 하나 안의 `mod` 블록으로 개념을 하나씩 확인하고, 마지막에는 같은 구조를 여러 파일로 나눈 프로젝트 `projects/restaurant`로 라이브러리와 바이너리를 함께 가진 패키지를 만들어 봅니다.

- 패키지: `p0-ch7-modules`
- 기준 문서: [The Book 7장 Packages, Crates, and Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- 확인한 툴체인: rustc 1.98.1, cargo 1.98.1 (Edition 2024)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. 패키지, 크레이트, 크레이트 루트의 관계를 설명하고, 어떤 파일이 어떤 크레이트의 루트인지 짚을 수 있다.
2. `mod`로 모듈 트리를 만들고, 절대 경로(`crate::`)와 상대 경로(`self::`, `super::`)로 원하는 항목을 가리킬 수 있다.
3. "기본은 비공개"라는 규칙과 `pub`의 효과를 설명하고, 구조체 필드와 열거형 배리언트의 공개 규칙 차이를 고려해 공개 범위를 정할 수 있다.
4. `use`로 경로를 스코프에 가져오고(관용적인 형태, `as`, 중첩 경로, 글롭), `use`가 유효한 범위와 `pub use` 재내보내기를 설명할 수 있다.
5. 모듈을 여러 파일로 나누는 규칙에 따라, 라이브러리 크레이트와 그것을 쓰는 바이너리 크레이트를 함께 가진 패키지를 구성할 수 있다.

## 2. 핵심 개념

### 2.1 모듈 시스템이 푸는 문제

지금까지의 예제는 파일 하나에 함수 몇 개면 충분했습니다. 코드가 수천 줄로 늘어나면 세 가지 문제가 생깁니다.

- **찾기**: 관련 있는 코드가 흩어져 있으면 무엇이 어디 있는지 알기 어렵습니다.
- **이름 충돌**: `clean`, `new`처럼 흔한 이름은 여러 곳에서 쓰고 싶어집니다.
- **약속의 범위**: 바깥에서 써도 되는 코드와, 내부 사정이라 마음대로 바꿔도 되는 코드를 구분해야 합니다. 내부 구현을 숨기고 정해진 창구만 공개하는 것을 캡슐화(encapsulation)라고 합니다.

Rust는 이 문제들을 다음 도구로 풉니다.

| 도구 | 하는 일 |
| --- | --- |
| 패키지 | 크레이트를 빌드·테스트·공유하는 Cargo의 단위입니다. `Cargo.toml`을 가집니다. |
| 크레이트 | 컴파일러가 한 번에 컴파일하는 단위입니다. 모듈 트리 하나로 이뤄지며 실행 파일이나 라이브러리가 됩니다. |
| 모듈과 `use` | 크레이트 안에서 코드를 묶고, 이름이 보이는 범위와 공개 여부를 정합니다. |
| 경로 | 모듈 트리 안의 항목(함수, 구조체 등)을 가리키는 이름입니다. |

### 2.2 패키지와 크레이트

**크레이트**는 Rust 컴파일러가 한 번에 다루는 가장 작은 코드 단위입니다. ch1에서 Cargo 없이 `rustc`로 `main.rs` 하나를 컴파일했을 때도 그 파일 하나가 크레이트였습니다. 크레이트는 두 종류입니다.

- **바이너리 크레이트(binary crate)**: 실행 파일로 컴파일됩니다. 실행이 시작되는 `main` 함수가 반드시 있어야 합니다. 지금까지 만든 것은 모두 바이너리 크레이트입니다.
- **라이브러리 크레이트(library crate)**: `main`이 없고 실행 파일이 되지 않습니다. 다른 크레이트가 가져다 쓸 기능을 제공합니다. ch2에서 쓴 `rand`가 라이브러리 크레이트입니다. Rust 개발자가 그냥 "크레이트"라고 하면 대개 라이브러리 크레이트를 뜻합니다.

**크레이트 루트(crate root)** 는 컴파일러가 크레이트를 컴파일할 때 출발하는 소스 파일입니다. 크레이트 루트의 내용이 그 크레이트의 맨 위 모듈(이름은 `crate`)이 되고, 다른 모듈은 모두 그 아래에 매달립니다.

**패키지**는 하나 이상의 크레이트를 묶은 것이며, 그 크레이트들을 어떻게 빌드할지 적은 `Cargo.toml`을 가집니다. 규칙은 두 가지입니다.

- 라이브러리 크레이트는 **최대 하나**, 바이너리 크레이트는 **몇 개든** 가질 수 있습니다.
- 크레이트가 **최소 하나**는 있어야 합니다.

`Cargo.toml`에는 크레이트 루트가 어디인지 적혀 있지 않습니다. Cargo가 다음 관례로 찾아서 `rustc`에 넘기기 때문입니다.

| 파일 | 만들어지는 크레이트 |
| --- | --- |
| `src/main.rs` | 패키지 이름과 같은 이름의 바이너리 크레이트 (이 파일이 크레이트 루트) |
| `src/lib.rs` | 패키지 이름과 같은 이름의 라이브러리 크레이트 (이 파일이 크레이트 루트) |
| `src/bin/*.rs` | 파일마다 바이너리 크레이트 하나 (파일 이름이 크레이트 이름) |
| `examples/*.rs` | 파일마다 예제 하나. 예제도 `main`을 가진 바이너리 크레이트로 컴파일됩니다. |

`src/main.rs`와 `src/lib.rs`가 둘 다 있으면 이름이 같은 크레이트 두 개(바이너리 하나, 라이브러리 하나)를 가진 패키지가 됩니다. 이 챕터의 `projects/restaurant`가 그런 패키지입니다. 그 디렉터리에서 `cargo build -v`를 실행해 Cargo가 `rustc`를 어떻게 부르는지 보면 크레이트 루트 두 개가 따로 컴파일됩니다(옵션은 대부분 생략했습니다).

```text
rustc --crate-name restaurant --edition=2024 src/lib.rs --crate-type lib ...
rustc --crate-name restaurant --edition=2024 src/main.rs --crate-type bin ... --extern restaurant=…/librestaurant-….rlib
```

- `rustc`에는 크레이트 루트만 넘어갑니다. 다른 `.rs` 파일은 크레이트 루트의 `mod` 선언을 따라 컴파일러가 직접 찾습니다(2.11절).
- 바이너리 쪽의 `--extern restaurant=...`는 "방금 만든 라이브러리 크레이트를 `restaurant`라는 이름으로 쓸 수 있게 하라"는 뜻입니다(2.12절).

**이 저장소의 경우**: 챕터 패키지(예: `p0-ch7-modules`)에는 `src/`가 없고 `examples/`만 있습니다. Cargo는 예제 파일을 하나하나 따로 컴파일하므로 **예제 파일 하나가 크레이트 하나이고, 그 파일이 크레이트 루트**입니다. 이 챕터의 예제를 `cargo build -v`로 빌드해 보면 `--crate-name 01_module_tree ... --crate-type bin`처럼 예제 파일 이름이 크레이트 이름이 됩니다. 빌드할 대상이 하나도 없는 패키지는 Cargo가 `no targets specified in the manifest` 에러로 거부합니다.

### 2.3 모듈과 모듈 트리

`mod` 키워드 뒤에 이름을 쓰고 중괄호 안에 내용을 넣으면 모듈이 됩니다. 모듈 안에는 함수, 구조체, 열거형, 상수와 다른 모듈을 넣을 수 있습니다.

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
        pub fn seat_at_table() {}
    }

    pub mod serving {
        pub fn take_order() {}
        pub fn clean() {}
    }
}

mod back_of_house {
    pub fn cook_order() {}
    pub fn clean() {}
}
```

`pub`은 2.5절에서 설명합니다. 이 코드는 다음 모듈 트리(module tree)를 만듭니다.

```text
crate
 ├── front_of_house
 │   ├── hosting
 │   │   ├── add_to_waitlist
 │   │   └── seat_at_table
 │   └── serving
 │       ├── take_order
 │       └── clean
 └── back_of_house
     ├── cook_order
     └── clean
```

- 모듈 A가 모듈 B 안에 있으면 A는 B의 **자식(child)**, B는 A의 **부모(parent)** 입니다. `hosting`과 `serving`처럼 같은 부모 아래에 있으면 **형제(sibling)** 입니다.
- 트리 전체의 뿌리는 크레이트 루트 파일이 만드는 암묵적인 모듈 `crate`입니다.
- 모듈은 **이름 공간(namespace)** 이기도 합니다. `front_of_house::serving::clean`과 `back_of_house::clean`은 이름이 같아도 위치가 다르므로 서로 다른 함수입니다.

모듈 트리는 파일 시스템의 디렉터리 트리와 닮았습니다. 디렉터리로 파일을 정리하듯 모듈로 코드를 정리하고, 경로로 원하는 항목을 찾아갑니다.

`module_path!()` 매크로는 그 코드가 모듈 트리의 어디에 있는지를 문자열로 돌려줍니다. 첫 부분은 컴파일 중인 크레이트의 이름입니다. 이 챕터의 예제는 이 매크로로 각 함수의 위치를 출력해서 모듈 트리를 눈으로 확인하게 합니다.

### 2.4 경로: 절대 경로와 상대 경로

함수를 부르려면 그 함수의 경로를 알아야 합니다. 경로는 이름을 `::`로 이어 쓴 것이며 두 가지 형태가 있습니다.

| 형태 | 시작 | 예 | 파일 시스템에 비유하면 |
| --- | --- | --- | --- |
| 절대 경로 | `crate` (같은 크레이트) | `crate::front_of_house::hosting::add_to_waitlist()` | `/front_of_house/hosting/...` |
| 절대 경로 | 크레이트 이름 (다른 크레이트) | `std::cmp::max(3, 7)` | 다른 디스크의 `/` |
| 상대 경로 | 현재 모듈에 있는 이름 | `front_of_house::hosting::add_to_waitlist()` | `front_of_house/hosting/...` |
| 상대 경로 | `self` (현재 모듈) | `self::add_to_waitlist()` | `./add_to_waitlist` |
| 상대 경로 | `super` (부모 모듈) | `super::deliver_order()` | `../deliver_order` |

`super`는 이어 쓸 수 있습니다. `super::super::deliver_order()`는 두 단계 위 모듈에 있는 `deliver_order`입니다.

어느 쪽을 쓸지는 코드가 앞으로 어떻게 옮겨질지에 달려 있습니다.

- 정의하는 코드와 부르는 코드가 **따로** 옮겨질 가능성이 크면 절대 경로가 안전합니다. 부르는 쪽이 어디로 가든 `crate::...`는 같은 곳을 가리키기 때문입니다. The Book은 일반적으로 절대 경로를 권합니다.
- 부모 모듈과 늘 **함께** 움직일 코드라면 `super`가 편합니다. 둘을 통째로 다른 곳으로 옮겨도 경로를 고칠 필요가 없습니다.

코드 안에서는 자기 크레이트를 언제나 `crate`라고 부릅니다. 크레이트 이름(`std`, `rand`, `restaurant` 등)으로 시작하는 경로는 **다른 크레이트**를 가리킬 때 씁니다. 그래서 예제 02에서 `module_path!()`는 `02_paths::back_of_house`를 출력하지만, 코드에서는 `crate::back_of_house`라고 씁니다.

### 2.5 기본은 비공개, 공개는 `pub`

Rust의 모든 항목(함수, 메서드, 구조체, 열거형, 상수, 모듈)은 기본적으로 **부모 모듈에 대해 비공개(private)** 입니다. 규칙을 정리하면 다음과 같습니다.

1. 부모 모듈은 자식 모듈의 비공개 항목을 쓸 수 없습니다.
2. 자식 모듈은 조상 모듈의 항목을 비공개여도 쓸 수 있습니다.
3. 같은 모듈 안에 정의된 항목끼리는 공개 여부와 상관없이 서로 쓸 수 있습니다.
4. `pub`을 붙인 항목은 부모 모듈 바깥에서도 쓸 수 있습니다. 단, 경로의 모든 단계에 접근할 수 있어야 합니다.

식당에 비유하면 비공개 항목은 주방의 내부 사정입니다. 손님(바깥 코드)은 주방 안을 볼 수 없지만, 주방 직원(자식 모듈)은 식당 전체 사정을 압니다. 자식 모듈은 자기가 들어 있는 맥락을 볼 수 있고, 부모는 자식의 구현 세부를 몰라도 되도록 만든 것입니다.

**왜 비공개가 기본일까요?** 공개한 항목은 바깥 코드가 기대는 약속이 됩니다. 비공개 항목은 바깥에서 쓰는 곳이 없다는 것을 컴파일러가 보장하므로, 이름을 바꾸거나 지우거나 동작을 바꿔도 바깥 코드가 깨지지 않습니다. 숨기는 쪽이 기본이어야 공개가 "일부러 내린 결정"이 됩니다.

`pub mod`에는 함정이 하나 있습니다. **모듈에 `pub`을 붙이면 모듈 이름만 공개되고, 안의 항목은 공개되지 않습니다.** 모듈은 그릇일 뿐이라 이름만 공개해서는 할 수 있는 일이 거의 없습니다. 바깥에 내놓을 항목에도 하나하나 `pub`을 붙여야 합니다(4.1절).

```rust
mod kitchen {
    const SAUCE_PER_SERVING: u32 = 120; // 비공개

    pub fn make_pasta(servings: u32) {
        let sauce = prepare_sauce(servings); // 같은 모듈: 비공개도 쓸 수 있음
        // ...
    }

    fn prepare_sauce(servings: u32) -> u32 { // 비공개
        servings * SAUCE_PER_SERVING
    }

    pub mod staff {
        pub fn check_recipe() {
            println!("{}ml", super::SAUCE_PER_SERVING); // 자식 → 조상의 비공개: 쓸 수 있음
        }
    }
}

fn main() {
    kitchen::make_pasta(3);       // kitchen은 main의 형제, make_pasta는 pub
    // kitchen::prepare_sauce(1); // E0603: function `prepare_sauce` is private
}
```

`main`이 `kitchen`에 접근할 수 있는 것은 `kitchen`이 `pub`이어서가 아니라, `main`과 `kitchen`이 둘 다 크레이트 루트에 정의된 형제이기 때문입니다(규칙 3). 전체 코드는 [`examples/03_privacy.rs`](examples/03_privacy.rs)에 있습니다.

### 2.6 구조체와 열거형의 공개 규칙

구조체와 열거형은 `pub`을 붙였을 때의 효과가 서로 다릅니다.

**`pub struct`는 구조체 이름만 공개하고, 필드는 여전히 비공개입니다.** 필드마다 `pub`을 따로 정합니다.

```rust
mod back_of_house {
    pub struct Breakfast {
        pub toast: String,      // 손님이 고르는 빵: 공개
        seasonal_fruit: String, // 주방이 정하는 과일: 비공개
    }

    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast { /* ... */ } // 공개 생성자
        pub fn fruit(&self) -> &str { &self.seasonal_fruit }  // 읽기 전용 창구
    }
}
```

비공개 필드가 하나라도 있으면 바깥에서는 `Breakfast { toast: ..., seasonal_fruit: ... }`처럼 구조체 리터럴로 값을 만들 수 없습니다. 비공개 필드에 들어갈 값을 바깥에서 정할 수는 없기 때문입니다. 다음은 `main`에서 `Breakfast`를 구조체 리터럴로 만들려고 했을 때의 에러입니다(파일 이름과 줄 번호는 코드에 따라 다릅니다).

```text
error[E0451]: field `seasonal_fruit` of struct `Breakfast` is private
  --> e0451.rs:11:9
   |
 9 |     let meal = back_of_house::Breakfast {
   |                ------------------------ in this type
10 |         toast: String::from("호밀빵"),
11 |         seasonal_fruit: String::from("블루베리"),
   |         ^^^^^^^^^^^^^^ private field
```

그래서 비공개 필드가 있는 공개 구조체는 `summer` 같은 공개 연관 함수(생성자)를 제공해야 합니다. 비공개 필드를 바깥에서 읽기만 하게 하려면 `fruit` 같은 공개 메서드를 둡니다. 이렇게 하면 "과일은 볼 수 있지만 바꿀 수는 없다" 같은 규칙을 타입이 스스로 지킵니다.

**`pub enum`은 열거형을 공개하면 모든 배리언트가 함께 공개됩니다.**

```rust
mod back_of_house {
    pub enum Appetizer {
        Soup, // 따로 pub을 붙이지 않아도 공개
        Salad,
    }
}
```

차이가 나는 이유는 쓰임새입니다. 배리언트를 쓸 수 없는 열거형은 값을 만들 수도, `match`할 수도 없어 쓸모가 없으므로 배리언트는 공개가 기본입니다. 반면 구조체는 필드를 숨긴 채로도 쓸모가 많으므로(위의 `Breakfast`처럼) "기본은 비공개" 규칙을 그대로 따릅니다.

### 2.7 `use`로 경로를 스코프에 가져오기

함수를 부를 때마다 `crate::front_of_house::hosting::add_to_waitlist()`를 쓰기는 번거롭습니다. `use`는 경로에 짧은 이름(바로 가기)을 만들어 줍니다. 파일 시스템의 심볼릭 링크와 비슷합니다.

```rust
use crate::front_of_house::hosting;

fn main() {
    hosting::add_to_waitlist("김민준");
}
```

`use`로 가져오는 경로도 일반 경로와 똑같이 공개 규칙 검사를 받습니다. 비공개 항목은 `use`로도 가져올 수 없습니다.

ch2에서 쓴 `use std::io;`도 같은 문법입니다. 표준 라이브러리 `std`는 Rust와 함께 설치되는 별도의 크레이트입니다. `Cargo.toml`에 적지 않아도 쓸 수 있지만, 다른 크레이트이므로 경로는 크레이트 이름 `std`로 시작합니다.

**관용적인 형태**: 무엇을 가져오느냐에 따라 관례가 다릅니다.

| 가져오는 것 | 관례 | 예 |
| --- | --- | --- |
| 함수 | **부모 모듈까지** 가져와서 `모듈::함수()`로 부릅니다. | `use std::cmp;` → `cmp::max(a, b)` |
| 구조체, 열거형 같은 타입 | **그 타입까지** 전체 경로로 가져옵니다. | `use std::cmp::Ordering;` → `Ordering::Less` |

함수를 부모 모듈까지만 가져오면 호출하는 곳에 `hosting::`이 남아서, 이 함수가 이 파일에 정의된 함수가 아니라는 것이 한눈에 보입니다. 타입을 전체 경로로 가져오는 데에는 특별한 이유가 있다기보다 그렇게 굳어진 관례입니다. 이름이 같은 타입 두 개를 가져올 때는 예외입니다(2.8절).

**`use`는 선언한 스코프에서만 유효합니다.** 크레이트 루트에 쓴 `use`는 크레이트 루트에서만 통하고, 자식 모듈 안에서는 통하지 않습니다. 자식 모듈은 별개의 스코프이기 때문입니다(4.3절).

```rust
use crate::front_of_house::hosting;

mod customer {
    pub fn arrive(name: &str) {
        hosting::add_to_waitlist(name); // E0433: 이 스코프에는 hosting이라는 이름이 없음
    }
}
```

고치는 방법은 두 가지입니다.

```rust
mod customer {
    use crate::front_of_house::hosting; // 방법 1: 자식 모듈 안에서 다시 use

    pub fn arrive(name: &str) {
        hosting::add_to_waitlist(name);
    }
}

mod takeout {
    pub fn call(name: &str) {
        super::hosting::add_to_waitlist(name); // 방법 2: 부모에 use로 만든 이름을 super::로 가리킴
    }
}
```

방법 2가 되는 이유는, `use`로 만든 이름도 그 스코프에 정의된 항목처럼 경로로 찾을 수 있기 때문입니다. 함수 본문 안에도 `use`를 쓸 수 있고, 그 `use`는 그 함수 안에서만 유효합니다. 모두 [`examples/05_use.rs`](examples/05_use.rs)에서 확인합니다.

### 2.8 이름 충돌: 부모 모듈로 구분하거나 `as`로 별칭 붙이기

한 스코프에는 같은 이름이 둘 있을 수 없습니다. 서로 다른 모듈에 있는 같은 이름의 타입 두 개를 모두 `use`로 가져오면 E0252 에러가 납니다(4.4절). 해결 방법은 두 가지이고, 둘 다 관용적인 방법입니다.

```rust
// 방법 1: 부모 모듈까지만 가져와서, 쓸 때 모듈 이름으로 구분합니다.
use crate::menu::dinner;
use crate::menu::lunch;

let a = lunch::Course::Bibimbap;
let b = dinner::Course::Steak;
```

```rust
// 방법 2: as로 이 스코프에서 쓸 새 이름(별칭)을 붙입니다.
use crate::menu::dinner::Course as DinnerCourse;
use crate::menu::lunch::Course as LunchCourse;

let a = LunchCourse::Kimbap;
let b = DinnerCourse::Pasta;
```

`as`는 이름만 바꿀 뿐 타입은 그대로입니다. `LunchCourse`와 `crate::menu::lunch::Course`는 같은 타입입니다.

### 2.9 중첩 경로와 글롭

**중첩 경로(nested path)**: 앞부분이 같은 경로 여러 개는 공통 부분 뒤에 `{ }`를 붙여 한 줄로 씁니다. 공통 부분 자체도 가져오려면 목록에 `self`를 넣습니다.

```rust
use crate::menu::{dinner, drinks, lunch};
// = use crate::menu::dinner;
//   use crate::menu::drinks;
//   use crate::menu::lunch;

use std::cmp::{self, Ordering};
// = use std::cmp;
//   use std::cmp::Ordering;
```

**글롭(glob) `*`**: 경로 뒤에 `::*`를 쓰면 그 모듈의 **공개 항목 전부**를 가져옵니다. 비공개 항목은 가져오지 않습니다.

```rust
use crate::menu::drinks::*;

coffee();
tea();
```

글롭은 편하지만, 코드만 보고는 `coffee`가 어디에서 왔는지 알 수 없고, 가져오는 모듈에 항목이 추가되면 내 코드의 이름과 겹칠 수도 있습니다. 그래서 일반 코드에서는 아껴 쓰고, 주로 다음 두 곳에서 씁니다.

- 프렐류드(prelude) 패턴: 자주 쓰는 항목을 모아 둔 모듈을 한꺼번에 가져올 때입니다. ch2의 `use rand::prelude::*;`가 이 경우입니다.
- 테스트 모듈에서 테스트할 대상을 모두 가져올 때입니다. `ch11-testing`에서 다룹니다.

### 2.10 `pub use`로 재내보내기

`use`로 만든 이름은 그 스코프의 **비공개** 항목입니다. 여기에 `pub`을 붙여 `pub use`라고 쓰면 그 이름을 바깥에 공개합니다. 다른 곳에 정의된 항목을 이 모듈의 항목처럼 다시 내보낸다고 해서 재내보내기(re-exporting)라고 합니다.

```rust
mod restaurant {
    mod front_of_house { // 비공개
        pub mod hosting {
            pub fn add_to_waitlist() { /* ... */ }
        }
    }

    pub use self::front_of_house::hosting; // restaurant::hosting으로 공개
}

fn main() {
    restaurant::hosting::add_to_waitlist();                    // 됨
    // restaurant::front_of_house::hosting::add_to_waitlist(); // E0603: module `front_of_house` is private
}
```

재내보내기는 **코드의 내부 구조와 사용하는 사람이 생각하는 구조가 다를 때** 쓸모가 있습니다. 식당을 운영하는 사람은 "홀(front of house)"과 "주방(back of house)"으로 나눠 생각하지만, 손님은 그런 구분을 모릅니다. `pub use`를 쓰면 내부는 만드는 사람에게 편한 구조로 두고, 바깥에는 쓰는 사람에게 편한 경로를 보여 줄 수 있습니다. `pub use ... as 새_이름`처럼 이름을 바꿔 공개할 수도 있습니다.

`pub use`는 항목을 옮기거나 복사하지 않습니다. 같은 항목을 가리키는 경로를 하나 더 공개할 뿐입니다. 예제 07에서 `restaurant::hosting::add_to_waitlist()`로 불러도 `module_path!()`가 여전히 `07_pub_use::restaurant::front_of_house::hosting`을 출력하는 것으로 확인할 수 있습니다.

`pub` 없이 `use`만 쓰면 바깥에서는 그 이름을 쓸 수 없습니다. 위 코드에서 `pub use`를 `use`로 바꾸면 `main`의 첫 줄에서 다음 에러가 납니다.

```text
error[E0603]: module import `hosting` is private
```

### 2.11 모듈을 파일로 나누기

모듈이 커지면 내용을 별도의 파일로 옮깁니다. `mod 이름 { ... }`의 중괄호 블록 대신 세미콜론으로 끝나는 **선언**만 남기고, 블록의 내용을 정해진 위치의 파일로 옮기면 됩니다.

```rust
// src/lib.rs
mod front_of_house; // 내용은 src/front_of_house.rs에 있습니다.
```

```rust
// src/front_of_house.rs
pub mod hosting; // 내용은 src/front_of_house/hosting.rs에 있습니다.
pub mod serving;
```

```rust
// src/front_of_house/hosting.rs — 파일 전체가 hosting 모듈의 본문이므로 mod hosting { } 블록을 다시 쓰지 않습니다.
pub fn add_to_waitlist(name: &str) { /* ... */ }
```

컴파일러가 모듈의 내용을 찾는 위치는 다음과 같습니다.

| 선언한 곳 | 선언 | 찾는 파일 (권장 방식) | 찾는 파일 (옛 방식) |
| --- | --- | --- | --- |
| 크레이트 루트 (`src/lib.rs`, `src/main.rs`) | `mod front_of_house;` | `src/front_of_house.rs` | `src/front_of_house/mod.rs` |
| `front_of_house` 모듈 | `mod hosting;` | `src/front_of_house/hosting.rs` | `src/front_of_house/hosting/mod.rs` |

하위 모듈의 파일은 **부모 모듈 이름을 딴 디렉터리** 안에 둡니다. 그래서 디렉터리와 파일의 구조가 모듈 트리와 똑같아집니다. `hosting.rs`를 `src/`에 바로 두면 컴파일러는 그 위치를 크레이트 루트의 자식 자리로 보므로, `front_of_house` 안에서 선언한 `hosting`의 파일을 찾지 못합니다(4.5절).

알아 둘 점:

- **`mod`는 include가 아닙니다.** C의 `#include`처럼 파일 내용을 그 자리에 붙여 넣는 명령이 아니라, 모듈 트리에 모듈 하나를 등록하는 선언입니다. 그래서 한 파일은 모듈 트리 전체에서 **딱 한 번만** `mod`로 선언합니다. 다른 곳에서 그 코드를 쓸 때는 `mod`를 또 쓰지 않고 2.4절의 경로(`crate::front_of_house::hosting` 등)나 `use`로 가리킵니다(4.6절).
- **`mod`로 선언하지 않은 `.rs` 파일은 컴파일되지 않습니다.** `src/` 안에 파일을 만들기만 해서는 크레이트에 들어가지 않으며, 에러도 경고도 나지 않습니다. 실험 삼아 Rust 코드가 아닌 내용을 `src/stray.rs`에 넣어도 `cargo build`는 성공합니다.
- **`use`는 어떤 파일이 컴파일될지와 상관이 없습니다.** 파일을 크레이트에 넣는 것은 `mod`이고, `use`는 이미 트리에 있는 항목에 짧은 이름을 붙일 뿐입니다.
- **파일을 나눠도 모듈 트리와 경로는 그대로입니다.** 그래서 블록을 파일로 옮겨도 그 모듈을 쓰는 코드(`pub use crate::front_of_house::hosting;` 등)는 고칠 필요가 없습니다.

**옛 방식 `mod.rs`**: 모듈의 내용을 `src/front_of_house/mod.rs`처럼 모듈 이름의 디렉터리 안 `mod.rs`에 두는 예전 방식도 아직 지원됩니다. 한 프로젝트 안에서 모듈마다 다른 방식을 써도 되지만 읽는 사람이 헷갈리기 쉽습니다. 이 방식의 가장 큰 단점은 프로젝트에 `mod.rs`라는 같은 이름의 파일이 여러 개 생겨서 편집기에서 여러 개를 열어 두면 구분하기 어렵다는 것입니다. 같은 모듈에 두 방식의 파일이 모두 있으면 컴파일러가 어느 쪽을 쓸지 정할 수 없어 에러를 냅니다.

```text
error[E0761]: file for module `front_of_house` found at both "src/front_of_house.rs" and "src/front_of_house/mod.rs"
  --> src/lib.rs:18:1
   |
18 | mod front_of_house;
   | ^^^^^^^^^^^^^^^^^^^
   |
   = help: delete or rename one of them to remove the ambiguity
```

### 2.12 라이브러리와 바이너리를 함께 가진 패키지

라이브러리 크레이트와 바이너리 크레이트를 함께 가진 패키지는 보통 이렇게 나눕니다.

- **모듈 트리와 실제 기능은 `src/lib.rs`(라이브러리)에** 둡니다.
- **`src/main.rs`(바이너리)에는 라이브러리를 불러 실행을 시작할 만큼의 코드만** 둡니다.

바이너리 크레이트는 같은 패키지의 라이브러리 크레이트를 **패키지 이름**으로 시작하는 경로로 씁니다. `Cargo.toml`에 의존성으로 적을 필요는 없습니다. 2.2절의 `--extern restaurant=...`가 Cargo가 알아서 해 주는 그 연결입니다.

```rust
// src/main.rs
use restaurant::back_of_house::{Appetizer, Breakfast};
use restaurant::hosting;
```

이때 바이너리는 **바깥 사용자와 같은 입장**입니다. 크레이트가 다르므로 라이브러리의 공개 API만 쓸 수 있습니다. 라이브러리를 만드는 사람이 스스로 첫 번째 사용자가 되므로 공개 API가 쓰기 편한지 자연스럽게 점검하게 되고, 라이브러리 쪽 코드는 다른 프로젝트에서도 그대로 가져다 쓸 수 있습니다. `ch12-io-project-minigrep`에서 이 구성으로 명령줄 프로그램을 만듭니다.

두 크레이트의 이름이 모두 `restaurant`라서 헷갈리기 쉬운 점이 있습니다. **바이너리 안에서 `crate`는 바이너리 자신(`src/main.rs`)** 입니다. `main.rs`에서 `crate::eat_at_restaurant()`라고 쓰면 라이브러리가 아니라 바이너리의 루트에서 찾기 때문에 실패합니다.

```text
error[E0425]: cannot find function `eat_at_restaurant` in the crate root
  --> src/main.rs:28:12
   |
28 |     crate::eat_at_restaurant();
   |            ^^^^^^^^^^^^^^^^^ not found in the crate root
```

라이브러리의 항목은 `restaurant::eat_at_restaurant()`처럼 언제나 패키지 이름으로 가리킵니다.

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_module_tree.rs`](examples/01_module_tree.rs) | `mod` 블록으로 만든 모듈 트리, 예제 파일이 곧 크레이트 루트라는 것, 이름 공간 | 1, 2 |
| [`examples/02_paths.rs`](examples/02_paths.rs) | 절대 경로 `crate::`, 상대 경로, `self::`, `super::`, `super::super::` | 2 |
| [`examples/03_privacy.rs`](examples/03_privacy.rs) | 기본 비공개, `pub mod`와 안의 항목, 자식이 조상의 비공개 항목을 보는 규칙 | 3 |
| [`examples/04_struct_enum_pub.rs`](examples/04_struct_enum_pub.rs) | 구조체 필드별 공개, 생성자와 읽기 전용 메서드, 열거형 배리언트의 공개 | 3 |
| [`examples/05_use.rs`](examples/05_use.rs) | `use`의 관용(함수는 부모 모듈, 타입은 전체 경로), `use`의 스코프 | 4 |
| [`examples/06_use_variants.rs`](examples/06_use_variants.rs) | 이름 충돌 해결(부모 모듈, `as`), 중첩 경로, 글롭 | 4 |
| [`examples/07_pub_use.rs`](examples/07_pub_use.rs) | `pub use` 재내보내기, `pub use ... as` | 4 |
| [`projects/restaurant`](projects/restaurant) | 여러 파일로 나눈 라이브러리 + 그것을 쓰는 바이너리 | 1, 5 |
| [`compile_fail/`](compile_fail) | 공개 규칙과 `use`에서 자주 나는 컴파일 에러 | 3, 4 |

### 01_module_tree — 모듈 트리와 크레이트 루트

```sh
cargo run -q -p p0-ch7-modules --example 01_module_tree
```

```text
main의 위치: 01_module_tree
손님이 왔습니다.
  대기 명단에 추가합니다    @ 01_module_tree::front_of_house::hosting
  자리로 안내합니다         @ 01_module_tree::front_of_house::hosting
  주문을 받습니다           @ 01_module_tree::front_of_house::serving
  요리합니다                @ 01_module_tree::back_of_house
영업이 끝났습니다.
  테이블을 치웁니다         @ 01_module_tree::front_of_house::serving
  주방을 청소합니다         @ 01_module_tree::back_of_house
```

볼 점:

- `@` 뒤는 `module_path!()`의 결과입니다. 파일 맨 위 주석에 그린 모듈 트리와 그대로 일치합니다.
- `main`의 위치가 크레이트 이름 `01_module_tree` 하나뿐입니다. 이 예제 파일이 크레이트 루트이고, `main`은 그 맨 위 모듈에 정의된 함수이기 때문입니다.
- 이름이 같은 `clean` 함수 두 개가 충돌 없이 공존합니다. 모듈마다 이름 공간이 따로 있기 때문입니다.

### 02_paths — 절대 경로와 상대 경로

```sh
cargo run -q -p p0-ch7-modules --example 02_paths
```

```text
1. 절대 경로: crate::front_of_house::hosting::add_to_waitlist()
    -> 대기 명단에 추가합니다     @ 02_paths::front_of_house::hosting
2. 상대 경로: front_of_house::hosting::add_to_waitlist()
    -> 대기 명단에 추가합니다     @ 02_paths::front_of_house::hosting
3. 같은 모듈 안에서 이름만, 또는 self::로 부르기
    -> 대기 명단에 추가합니다     @ 02_paths::front_of_house::hosting
    -> 대기 명단에 추가합니다     @ 02_paths::front_of_house::hosting
4. super::로 부모 모듈의 함수 부르기
    -> 다시 요리합니다            @ 02_paths::back_of_house
    -> 음식을 손님에게 가져갑니다 @ 02_paths
5. super::super::와 crate::는 같은 곳을 가리킵니다
    -> 설거지합니다               @ 02_paths::back_of_house::dishwashing
    -> 음식을 손님에게 가져갑니다 @ 02_paths
    -> 음식을 손님에게 가져갑니다 @ 02_paths
```

볼 점:

- 1번과 2번은 쓰는 경로는 다르지만 같은 함수를 부릅니다. `@` 뒤의 위치가 같습니다.
- 4번에서 `back_of_house` 안의 `super::deliver_order()`는 크레이트 루트의 `deliver_order`를 부릅니다. 출력 위치가 크레이트 이름 `02_paths`뿐인 것이 그 증거입니다.
- 5번에서 `dishwashing` 안의 `super::super::deliver_order()`와 `crate::deliver_order()`는 같은 함수입니다.

### 03_privacy — 기본 비공개와 `pub`

```sh
cargo run -q -p p0-ch7-modules --example 03_privacy
```

```text
1. 공개 함수 호출
  파스타 3인분 완성 (소스 360ml 사용)
2. 자식 모듈은 조상의 비공개 항목을 볼 수 있습니다
  1인분 소스 양: 120ml
  2인분 소스 양: 240ml
  (메모: 이 레시피는 주방 밖으로 알려지지 않습니다)
```

볼 점:

- `make_pasta`는 공개 함수지만 안에서 비공개 함수 `prepare_sauce`와 비공개 상수를 씁니다. 같은 모듈 안이기 때문입니다.
- `kitchen::staff::check_recipe`는 부모 `kitchen`의 비공개 항목을 `super::`로 씁니다.
- 파일 끝에 주석 처리한 세 줄의 주석을 풀면 각각 E0603이 납니다. 메시지는 차례로 ``function `prepare_sauce` is private``, ``function `private_memo` is private``, ``constant `SAUCE_PER_SERVING` is private``입니다. `private_memo`는 `pub mod staff` 안에 있지만 자신에게 `pub`이 없어서 비공개입니다.

### 04_struct_enum_pub — 구조체와 열거형의 공개 규칙

```sh
cargo run -q -p p0-ch7-modules --example 04_struct_enum_pub
```

```text
1. 공개 필드는 바깥에서 읽고 바꿀 수 있습니다
  주문: 호밀빵 토스트
  변경: 통밀빵 토스트
2. 비공개 필드는 공개 메서드를 통해서만 읽을 수 있습니다
  곁들임 과일: 복숭아
  Breakfast { toast: "통밀빵", seasonal_fruit: "복숭아" }
3. 모든 필드가 공개된 구조체는 리터럴로 만들 수 있습니다
  음료: 아이스티 350ml
4. 공개 열거형의 배리언트는 모두 공개입니다
  Soup: 4000원
  Salad: 5500원
```

볼 점:

- `Breakfast`는 `summer` 생성자로만 만들 수 있고, `Drink`는 모든 필드가 공개라서 구조체 리터럴로 바로 만듭니다.
- `{:?}`(디버그 형식)로 출력하면 비공개 필드도 보입니다. `#[derive(Debug)]`가 만든 출력 코드는 구조체와 같은 모듈 안에 생기므로 비공개 필드를 읽을 수 있기 때문입니다. 공개 규칙은 "다른 모듈의 코드가 이름으로 접근하는 것"을 막는 규칙이지, 값을 화면에 감추는 기능이 아닙니다.
- `Appetizer`에는 `pub`이 한 번만 붙었지만 `main`의 `match`에서 두 배리언트를 모두 씁니다.

### 05_use — `use`의 관용과 스코프

```sh
cargo run -q -p p0-ch7-modules --example 05_use
```

```text
1. 함수는 부모 모듈 이름과 함께 부릅니다
  대기 명단에 추가: 김민준
2. 타입은 전체 경로로 가져와 타입 이름부터 씁니다
  Soup: 4000원
3. 표준 라이브러리 함수도 같은 관용을 따릅니다
  더 비싼 쪽: 5500원
4. 자식 모듈에서는 자기 use를 쓰거나 super::로 가리킵니다
  대기 명단에 추가: 이서연
  대기 명단에 추가: 박지호
5. 함수 안의 use는 그 함수 안에서만 유효합니다
  Soup는 Salad보다 더 쌉니다
```

볼 점:

- `use std::cmp;` 후 `cmp::max(...)`, `use crate::back_of_house::Appetizer;` 후 `Appetizer::Soup`처럼 2.7절의 관용을 따릅니다.
- `customer` 모듈은 자기 안에서 `use`를 다시 쓰고, `takeout` 모듈은 `super::hosting`으로 크레이트 루트의 `use`를 가리킵니다.
- `Ordering`은 `compare_prices` 함수 안에서만 `use`했습니다. `main`에서 `Ordering::Less`라고 쓰면 ``error[E0433]: cannot find type `Ordering` in this scope``가 납니다.

### 06_use_variants — `as`, 중첩 경로, 글롭

```sh
cargo run -q -p p0-ch7-modules --example 06_use_variants
```

```text
1. 이름이 같은 두 타입을 부모 모듈로 구분하기
  점심: Bibimbap / 저녁: Steak
2. as로 별칭 붙이기
  점심: Kimbap / 저녁: Pasta
3. 중첩 경로로 use 묶기
  점심: Kimbap / 저녁: Steak
  차를 우립니다
  cmp::max(3, 7) = 7, 3과 7을 비교하면 Less
  3이 7보다 작습니다
4. 글롭(*)으로 공개 항목 전부 가져오기
  커피를 내립니다 (원두: 하우스 블렌드)
  차를 우립니다
```

볼 점:

- 형태마다 함수를 하나씩 두고 그 안에 `use`를 썼습니다. 함수마다 스코프가 다르므로 `by_parent_module`과 `nested_paths`가 같은 이름 `lunch`를 가져와도 부딪히지 않습니다.
- 4번의 글롭은 `coffee`와 `tea`만 가져오고 비공개 함수 `secret_blend`는 가져오지 않습니다. 주석을 풀어 `secret_blend()`를 부르면 ``error[E0425]: cannot find function `secret_blend` in this scope``와 함께 ``function `crate::menu::drinks::secret_blend` exists but is inaccessible``라는 안내가 나옵니다. 함수는 있지만 비공개라서 접근할 수 없다는 뜻입니다.

### 07_pub_use — 재내보내기

```sh
cargo run -q -p p0-ch7-modules --example 07_pub_use
```

```text
1. 재내보낸 경로 restaurant::hosting으로 부르기
  대기 명단에 추가합니다 @ 07_pub_use::restaurant::front_of_house::hosting
2. 이름을 바꿔 재내보낸 restaurant::order_food 부르기
  요리합니다             @ 07_pub_use::restaurant::back_of_house
3. 모듈 안에서도 pub use로 만든 이름을 씁니다
  대기 명단에 추가합니다 @ 07_pub_use::restaurant::front_of_house::hosting
  요리합니다             @ 07_pub_use::restaurant::back_of_house
```

볼 점:

- `main`은 `restaurant::hosting`, `restaurant::order_food`라는 경로로 부르지만, `@` 뒤의 실제 위치는 여전히 `front_of_house::hosting`과 `back_of_house`입니다. `pub use`는 경로를 하나 더 공개할 뿐 항목을 옮기지 않습니다.
- 원래 경로 `restaurant::front_of_house::hosting::add_to_waitlist()`는 쓸 수 없습니다. 주석을 풀면 ``error[E0603]: module `front_of_house` is private``가 나고, 컴파일러가 `help:`로 재내보낸 경로 `crate::restaurant::hosting::add_to_waitlist()`를 대신 제안합니다.

### projects/restaurant — 파일로 나눈 라이브러리와 바이너리

예제 01~07에서 본 식당 모듈 구조를 여러 파일로 나누고, 라이브러리 크레이트(`src/lib.rs`)와 그것을 쓰는 바이너리 크레이트(`src/main.rs`)를 한 패키지에 담았습니다. 워크스페이스와 분리된 독립 프로젝트이므로 그 디렉터리에서 실행합니다.

```text
projects/restaurant/
├── Cargo.toml
├── Cargo.lock
└── src/
    ├── main.rs               바이너리 크레이트 루트
    ├── lib.rs                라이브러리 크레이트 루트          crate
    ├── front_of_house.rs     mod front_of_house;의 내용         crate::front_of_house
    ├── front_of_house/
    │   ├── hosting.rs        pub mod hosting;의 내용            crate::front_of_house::hosting
    │   └── serving.rs        pub mod serving;의 내용            crate::front_of_house::serving
    └── back_of_house.rs      pub mod back_of_house;의 내용      crate::back_of_house
```

라이브러리 크레이트의 모듈 트리는 다음과 같습니다.

```text
crate (restaurant)          src/lib.rs
 ├── front_of_house         (비공개)
 │   ├── hosting            add_to_waitlist, seat_at_table
 │   └── serving            take_order, serve_order
 ├── back_of_house          (공개) Breakfast, Appetizer
 ├── hosting                ← pub use로 재내보낸 이름
 └── eat_at_restaurant
```

```sh
cd part0-getting-started/ch7-modules/projects/restaurant
cargo run
```

```text
main의 위치: restaurant
1. 재내보낸 경로 restaurant::hosting으로 부르기
  [restaurant::front_of_house::hosting] 김민준님을 대기 명단에 추가했습니다
2. 공개 구조체: 공개 필드는 바꾸고, 비공개 필드는 메서드로 읽기
  통밀빵 토스트, 곁들임: 복숭아
3. 공개 열거형: 배리언트를 모두 쓸 수 있음
  전채 메뉴: [Soup, Salad]
4. 라이브러리의 공개 함수 부르기
  [restaurant] eat_at_restaurant 시작
  [restaurant::front_of_house::hosting] 최유나님을 대기 명단에 추가했습니다
  [restaurant::front_of_house::hosting] 최유나님을 자리로 안내했습니다
  [restaurant::front_of_house::serving] 최유나님의 주문: 호밀빵 토스트
  [restaurant::front_of_house::serving] 호밀빵 토스트를 내갑니다 (곁들임: 복숭아)
```

(`cargo run`이 출력하는 `Compiling`, `Finished`, `Running` 줄은 뺐습니다.)

볼 점:

- 대괄호 안은 `module_path!()`의 결과입니다. `src/front_of_house/hosting.rs`의 코드가 `restaurant::front_of_house::hosting`에 있다고 나옵니다. **파일 위치가 곧 모듈 트리의 위치**입니다.
- `main의 위치: restaurant`와 `[restaurant] eat_at_restaurant 시작`의 `restaurant`는 이름만 같을 뿐 서로 다른 크레이트(바이너리와 라이브러리)입니다.
- `main.rs`는 `restaurant::hosting`(재내보낸 경로)과 `restaurant::back_of_house`(공개 모듈)만 씁니다. 비공개 모듈인 `front_of_house::serving`은 라이브러리 안의 `eat_at_restaurant`만 쓸 수 있습니다. `main.rs` 끝의 주석을 풀면 다음 에러가 납니다.

  ```text
  error[E0603]: module `front_of_house` is private
    --> src/main.rs:32:17
     |
  32 |     restaurant::front_of_house::serving::take_order("김민준", "호밀빵");
     |                 ^^^^^^^^^^^^^^           ---------- function `take_order` is not publicly re-exported
     |                 |
     |                 private module
  ```

- `src/front_of_house/serving.rs`는 다른 파일(`src/back_of_house.rs`)에 있는 `Breakfast`를 `use crate::back_of_house::Breakfast;`로 가져옵니다. 파일 경로가 아니라 모듈 트리의 경로를 씁니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 코드를 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part0-getting-started/ch7-modules
rustc --edition 2024 compile_fail/e0603_private_function.rs --out-dir /tmp/rust-ch7
```

4.5절과 4.6절은 여러 파일이 필요한 실수라서 `projects/restaurant`의 복사본으로 재현합니다. 저장소 밖으로 복사한 뒤 그 안에서 실험하세요.

```sh
cp -R part0-getting-started/ch7-modules/projects/restaurant /tmp/restaurant-try
cd /tmp/restaurant-try
```

### 4.1 `pub mod`만 붙이고 안의 함수는 비공개 — E0603

파일: [`compile_fail/e0603_private_function.rs`](compile_fail/e0603_private_function.rs)

```text
error[E0603]: function `add_to_waitlist` is private
  --> compile_fail/e0603_private_function.rs:14:30
   |
14 |     front_of_house::hosting::add_to_waitlist();
   |                              ^^^^^^^^^^^^^^^ private function
   |
note: the function `add_to_waitlist` is defined here
  --> compile_fail/e0603_private_function.rs:9:9
   |
 9 |         fn add_to_waitlist() {}
   |         ^^^^^^^^^^^^^^^^^^^^
```

- **읽는 법**: `^^^`가 가리키는 경로의 마지막 단계 `add_to_waitlist`가 비공개라는 뜻입니다. `note:`가 그 함수가 정의된 곳을 보여 줍니다. 경로 중간의 모듈이 비공개라면 ``module `hosting` is private``처럼 그 모듈을 가리킵니다.
- **원인**: `hosting` 모듈에는 `pub`을 붙였지만 그 안의 함수에는 붙이지 않았습니다. 모듈의 `pub`은 모듈 이름만 공개합니다.
- **해결**: 바깥에서 쓸 함수에도 `pub fn add_to_waitlist()`처럼 `pub`을 붙입니다. 경로의 모든 단계(`front_of_house` → `hosting` → `add_to_waitlist`)에 접근할 수 있어야 합니다. `front_of_house`는 `main`과 같은 크레이트 루트에 있는 형제라서 `pub`이 없어도 됩니다.

### 4.2 비공개 필드를 바깥에서 사용 — E0616

파일: [`compile_fail/e0616_private_field.rs`](compile_fail/e0616_private_field.rs)

```text
error[E0616]: field `seasonal_fruit` of struct `Breakfast` is private
  --> compile_fail/e0616_private_field.rs:26:10
   |
26 |     meal.seasonal_fruit = String::from("블루베리"); // 비공개 필드라서 에러입니다.
   |          ^^^^^^^^^^^^^^ private field
```

- **원인**: `pub struct`는 구조체 이름만 공개합니다. `seasonal_fruit`에는 `pub`이 없으므로 `back_of_house` 바깥에서는 읽을 수도, 바꿀 수도 없습니다. 바로 위 줄의 `meal.toast = ...`는 `pub` 필드라서 문제없습니다.
- **해결**: 정말 바깥에서 바꿔도 되는 값이라면 필드에 `pub`을 붙입니다. 그렇지 않다면(이 예처럼 주방이 정하는 값이라면) 필드는 비공개로 두고, 읽기용 공개 메서드(`pub fn fruit(&self) -> &str`)나 규칙을 지키며 값을 바꾸는 공개 메서드를 만듭니다.
- 같은 이유로, 비공개 필드가 있는 구조체를 바깥에서 구조체 리터럴로 만들면 E0451(2.6절)이 납니다. 이때는 공개 생성자를 씁니다.

### 4.3 바깥 스코프의 `use`를 자식 모듈에서 사용 — E0433

파일: [`compile_fail/e0433_use_not_in_child_scope.rs`](compile_fail/e0433_use_not_in_child_scope.rs)

```text
error[E0433]: cannot find module or crate `hosting` in this scope
  --> compile_fail/e0433_use_not_in_child_scope.rs:18:9
   |
18 |         hosting::add_to_waitlist();
   |         ^^^^^^^ use of unresolved module or unlinked crate `hosting`
   |
   = help: you might be missing a crate named `hosting`
help: consider importing this module through its public re-export
   |
17 +     use crate::hosting;
   |

warning: unused import: `crate::front_of_house::hosting`
  --> compile_fail/e0433_use_not_in_child_scope.rs:14:5
   |
14 | use crate::front_of_house::hosting;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

- **읽는 법**: `customer` 모듈 안에서 `hosting`이라는 모듈이나 크레이트를 찾을 수 없다는 뜻입니다. 컴파일러는 혹시 `hosting`이라는 외부 크레이트를 빠뜨린 것은 아닌지도 물어봅니다(`= help:`). 함께 나온 경고는 크레이트 루트의 `use`가 한 번도 쓰이지 않았다는 뜻입니다. `use`가 자식 모듈까지 미치지 않는다는 또 하나의 증거입니다.
- **원인**: `use`는 그것을 쓴 스코프(크레이트 루트)에서만 바로 가기를 만듭니다. `customer`는 다른 스코프입니다.
- **해결**: (1) `use crate::front_of_house::hosting;`을 `customer` 모듈 안으로 옮기거나, (2) `super::hosting::add_to_waitlist()`처럼 부모 스코프의 이름을 가리킵니다. 컴파일러가 제안한 `use crate::hosting;`도 동작합니다. 크레이트 루트의 `use`가 `crate::hosting`이라는 이름을 만들었고, `customer`는 크레이트 루트의 자식이라서 그 이름을 볼 수 있기 때문입니다.

### 4.4 같은 이름을 두 번 `use` — E0252

파일: [`compile_fail/e0252_duplicate_use_name.rs`](compile_fail/e0252_duplicate_use_name.rs)

```text
error[E0252]: the name `Course` is defined multiple times
  --> compile_fail/e0252_duplicate_use_name.rs:23:5
   |
22 | use crate::menu::dinner::Course;
   |     --------------------------- previous import of the type `Course` here
23 | use crate::menu::lunch::Course;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ `Course` reimported here
   |
   = note: `Course` must be defined only once in the type namespace of this module
help: you can use `as` to change the binding name of the import
   |
23 | use crate::menu::lunch::Course as OtherCourse;
   |                                ++++++++++++++
```

- **원인**: 한 스코프에 `Course`라는 이름이 두 번 생겼습니다. 이대로라면 `Course::Special`이 점심 특선인지 저녁 특선인지 알 수 없습니다. 이 에러 뒤에 두 번째 `use`가 쓰이지 않았다는 `unused import` 경고도 따라 나옵니다.
- **해결**: 컴파일러의 `help:`처럼 `as`로 한쪽에 별칭을 붙이거나(`use crate::menu::lunch::Course as LunchCourse;`), 타입 대신 부모 모듈을 가져와 `lunch::Course`, `dinner::Course`로 구분합니다(2.8절).

### 4.5 모듈 파일을 엉뚱한 디렉터리에 둠 — E0583

`front_of_house`의 하위 모듈인 `hosting`의 파일을 `src/front_of_house/`가 아니라 `src/`에 둔 경우입니다. 복사본에서 재현합니다.

```sh
mv src/front_of_house/hosting.rs src/hosting.rs
cargo build
```

```text
error[E0583]: file not found for module `hosting`
 --> src/front_of_house.rs:6:1
  |
6 | pub mod hosting;
  | ^^^^^^^^^^^^^^^^
  |
  = help: to create the module `hosting`, create file "src/front_of_house/hosting.rs" or "src/front_of_house/hosting/mod.rs"
  = note: if there is a `mod hosting` elsewhere in the crate already, import it with `use crate::...` instead
```

- **원인**: 하위 모듈의 파일은 부모 모듈 이름의 디렉터리(`src/front_of_house/`) 안에서 찾습니다. `src/hosting.rs`는 크레이트 루트의 자식 `hosting`을 위한 자리이므로 찾아보지 않습니다. 그리고 `src/hosting.rs`는 어디에서도 `mod`로 선언되지 않았으므로 그냥 무시됩니다.
- **해결**: `help:`가 알려 주는 두 위치 중 하나로 파일을 옮깁니다. 보통은 `src/front_of_house/hosting.rs`입니다.

### 4.6 `main.rs`에서 라이브러리의 모듈 파일을 다시 `mod`로 선언 — E0432

다른 언어의 include처럼 생각해서, `main.rs`에서도 `src/front_of_house.rs`를 쓰려고 `mod front_of_house;`를 적는 경우입니다. 4.5절에서 옮긴 파일을 원래 자리로 되돌린 뒤, 복사본의 `src/main.rs` 맨 위에 `mod front_of_house;` 한 줄을 추가하고 `cargo build`를 실행하면 엉뚱해 보이는 곳에서 에러가 납니다.

```text
error[E0432]: unresolved import `crate::back_of_house`
 --> src/front_of_house/serving.rs:5:12
  |
5 | use crate::back_of_house::Breakfast;
  |            ^^^^^^^^^^^^^ could not find `back_of_house` in the crate root

For more information about this error, try `rustc --explain E0432`.
error: could not compile `restaurant` (bin "restaurant") due to 1 previous error
```

- **원인**: `mod front_of_house;`는 "이 크레이트의 모듈 트리에 `front_of_house`를 등록하라"는 선언입니다. 그래서 같은 파일들이 라이브러리 크레이트에 한 벌, 바이너리 크레이트에 또 한 벌 컴파일됩니다. 바이너리 쪽에서 컴파일될 때 `serving.rs`의 `crate::`는 바이너리의 루트(`src/main.rs`)를 가리키는데, 거기에는 `back_of_house`가 없습니다. 에러 끝의 `(bin "restaurant")`가 바이너리 크레이트를 컴파일하다 났다는 표시입니다.
- **해결**: `main.rs`의 `mod front_of_house;`를 지우고, 라이브러리의 공개 API를 `restaurant::...` 경로로 씁니다. 한 파일은 모듈 트리 전체에서 한 번만 `mod`로 선언합니다.

## 5. 연습 문제

### 연습 1. 깊은 모듈에서 세 가지 경로로 부르기

[`examples/02_paths.rs`](examples/02_paths.rs)의 `back_of_house::dishwashing` 모듈에 `pub fn call_next_guest()`를 추가하세요. 이 함수는 `front_of_house::hosting::add_to_waitlist()`를 (가) `crate::`로 시작하는 절대 경로, (나) `super::`로 시작하는 상대 경로, (다) `use`로 가져온 짧은 이름으로 한 번씩, 모두 세 번 부릅니다. `main`에서 `call_next_guest()`를 부르고, 세 번 모두 같은 위치가 출력되는지 확인하세요.

<details>
<summary>힌트</summary>

- `dishwashing`에서 두 단계 올라가면 크레이트 루트입니다. (나)는 `super::super::front_of_house::hosting::add_to_waitlist()`입니다.
- (다)는 `dishwashing` 모듈 안(또는 `call_next_guest` 함수 안)에 `use crate::front_of_house::hosting;`을 쓰고 `hosting::add_to_waitlist()`로 부릅니다. 크레이트 루트에 `use`를 쓰면 `dishwashing` 안에서는 통하지 않습니다(4.3절).
- `front_of_house`에는 `pub`이 없는데도 `dishwashing`에서 접근할 수 있는 이유를 2.5절의 규칙으로 설명해 보세요(규칙 2: 자식은 조상의 항목을 볼 수 있습니다).

</details>

### 연습 2. 잔액을 함부로 바꿀 수 없는 계좌

`bank` 모듈 안에 다음 조건을 만족하는 `Account` 구조체를 만드세요.

- 예금주 `owner: String`은 바깥에서 읽을 수 있습니다.
- 잔액 `balance: u32`은 바깥에서 직접 바꿀 수 없고, `deposit(&mut self, amount: u32)` 메서드로만 늘릴 수 있습니다. 잔액은 `balance(&self) -> u32` 메서드로 읽습니다.
- 계좌는 `Account::open("김민준")`으로 만들며, 잔액 0에서 시작합니다.

`main`에서 계좌를 열고 5000원과 3000원을 입금한 뒤 `김민준: 8000원`을 출력하세요. 그다음 `main`에서 `acc.balance = 1_000_000;`이라고 써 보고 어떤 에러가 나는지 확인하세요.

<details>
<summary>힌트</summary>

- `pub struct Account { pub owner: String, balance: u32 }`처럼 필드마다 공개 여부를 정합니다.
- 비공개 필드가 있으므로 바깥에서는 구조체 리터럴로 만들 수 없습니다. `pub fn open(owner: &str) -> Account`를 `impl` 블록에 둡니다.
- 메서드 이름 `balance`와 필드 이름 `balance`가 같아도 됩니다. `acc.balance()`는 메서드, `acc.balance`는 필드입니다.
- 직접 대입하면 ``error[E0616]: field `balance` of struct `Account` is private``가 납니다.

</details>

### 연습 3. 재내보내기로 공개 API 다듬기

`projects/restaurant`를 복사한 프로젝트에서, `main.rs`가 `use restaurant::{Appetizer, Breakfast};`처럼 짧은 경로로 타입을 쓸 수 있게 바꾸세요. 그리고 `lib.rs`의 `pub mod back_of_house;`를 `mod back_of_house;`(비공개)로 바꿔도 `main.rs`가 빌드되게 하세요.

<details>
<summary>힌트</summary>

- `lib.rs`에 `pub use crate::back_of_house::{Appetizer, Breakfast};`를 추가합니다. 중첩 경로와 `pub use`를 함께 쓴 형태입니다.
- `back_of_house` 모듈은 비공개여도, 안의 `Breakfast`와 `Appetizer`는 `pub`이고 `lib.rs`(그 부모)가 재내보내므로 바깥에서 `restaurant::Breakfast`로 쓸 수 있습니다.
- 이제 바이너리 입장에서는 `back_of_house`라는 내부 구분이 보이지 않습니다. 2.10절의 "내부 구조와 바깥 구조를 다르게" 하는 예입니다.

</details>

### 연습 4. 하위 모듈을 파일로 더 나누기

`projects/restaurant`를 복사한 프로젝트에서 `back_of_house`를 하위 모듈 두 개로 나누세요. `Breakfast`는 `back_of_house::breakfast` 모듈로, `Appetizer`는 `back_of_house::appetizer` 모듈로 옮기고, 각각 별도의 파일에 둡니다. 단, `main.rs`와 `serving.rs`는 한 글자도 고치지 않고 빌드돼야 합니다.

<details>
<summary>힌트</summary>

- 새 파일은 `src/back_of_house/breakfast.rs`와 `src/back_of_house/appetizer.rs`입니다. 부모 모듈 이름의 디렉터리를 만들어야 합니다.
- `src/back_of_house.rs`에는 `mod appetizer;`, `mod breakfast;` 선언과 `pub use self::appetizer::Appetizer;`, `pub use self::breakfast::Breakfast;`만 남깁니다. 재내보내기 덕분에 바깥에서 쓰던 경로 `crate::back_of_house::Breakfast`가 그대로 유지됩니다.
- 다 되면 `src/back_of_house.rs`를 `src/back_of_house/mod.rs`로 옮겨(옛 방식) 그대로 빌드되는지, 두 파일을 모두 두면 E0761이 나는지도 확인해 보세요.

</details>

## 6. 참고 자료

- The Book (기준 문서)
  - [7. Packages, Crates, and Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
  - [7.1 Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
  - [7.2 Control Scope and Privacy with Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html): 맨 앞의 "Modules Cheat Sheet"가 이 챕터 규칙의 요약입니다.
  - [7.3 Paths for Referring to an Item in the Module Tree](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html)
  - [7.4 Bringing Paths into Scope with the `use` Keyword](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html)
  - [7.5 Separating Modules into Different Files](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html)
- Rust By Example: [Modules](https://doc.rust-lang.org/rust-by-example/mod.html)와 [Crates](https://doc.rust-lang.org/rust-by-example/crates.html) 장
  - [Visibility](https://doc.rust-lang.org/rust-by-example/mod/visibility.html): 기본 비공개와 `pub`. 이 절에는 `pub(crate)`, `pub(super)`, `pub(in path)`처럼 공개 범위를 특정 크레이트나 모듈 안으로 제한하는 문법도 나옵니다. 예를 들어 `pub(crate)` 항목은 같은 크레이트 안에서는 어디서나 쓸 수 있지만 다른 크레이트에서는 쓸 수 없습니다. 그래서 `projects/restaurant`의 `fruit` 메서드를 `pub(crate)`로 바꾸면 라이브러리 안의 `serving.rs`는 그대로 빌드되지만, 바이너리 `main.rs`에서는 ``error[E0624]: method `fruit` is private``가 납니다.
  - [Struct visibility](https://doc.rust-lang.org/rust-by-example/mod/struct_visibility.html): 공개·비공개 필드와 공개 생성자. 예제의 `OpenBox<T>`에 붙은 `<T>`는 제네릭(`ch10-generics-traits-lifetimes`)으로, "아무 타입이나 담을 수 있는 상자"라고 보고 읽으면 됩니다.
  - [The use declaration](https://doc.rust-lang.org/rust-by-example/mod/use.html): `use`, `as`, `pub use`. 블록 안의 `use`가 바깥 이름을 가리는(섀도잉) 예도 있습니다.
  - [super and self](https://doc.rust-lang.org/rust-by-example/mod/super.html): 이름이 같은 함수가 여러 모듈에 있을 때 `self::`, `super::`, `crate::`로 구분하는 예입니다.
  - [File hierarchy](https://doc.rust-lang.org/rust-by-example/mod/split.html): 모듈을 `my.rs`와 `my/` 디렉터리로 나누는 예입니다.
  - [Crates](https://doc.rust-lang.org/rust-by-example/crates.html), [Creating a Library](https://doc.rust-lang.org/rust-by-example/crates/lib.html), [Using a Library](https://doc.rust-lang.org/rust-by-example/crates/using_lib.html): Cargo 없이 `rustc --crate-type=lib`로 라이브러리 크레이트를 만들고 `--extern`으로 연결하는 과정입니다. 2.2절에서 본 `cargo build -v`의 출력과 비교해 보면 Cargo가 대신 해 주는 일을 알 수 있습니다.
- Rustlings (설치와 실행 방법은 [Rustlings 저장소](https://github.com/rust-lang/rustlings/)에서 링크하는 공식 사이트에 안내되어 있습니다)
  - [`10_modules`](https://github.com/rust-lang/rustlings/tree/main/exercises/10_modules): `modules1.rs`는 비공개 함수 호출 에러 고치기, `modules2.rs`는 `pub use ... as`로 재내보내며 이름 바꾸기, `modules3.rs`는 `std::time`의 항목 두 개를 (가능하면 중첩 경로로 한 줄에) 가져오기입니다. `modules3.rs`의 `Ok(n)`, `Err(_)`는 `ch9-error-handling`에서 배우는 `Result`의 배리언트인데, 지금은 "성공/실패 두 배리언트를 가진 열거형을 `match`한다"고 보고 읽으면 됩니다.
- API 문서: [`module_path!` 매크로](https://doc.rust-lang.org/std/macro.module_path.html): 결과의 첫 부분이 컴파일 중인 크레이트의 이름이라는 설명이 있습니다.
- 에러 설명: `rustc --explain E0603`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
