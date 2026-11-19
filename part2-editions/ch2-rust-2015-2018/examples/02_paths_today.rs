// 2018 이후의 경로 규칙: crate::, self::, super::, 크레이트 이름, ::크레이트이름
//
// 실행: cargo run -p p2-ch2-rust-2015-2018 --example 02_paths_today
//
// 2018 에디션의 경로 규칙은 한 줄로 요약됩니다. "use 선언 안의 경로와 코드 안의 경로는 같은 규칙을
// 따르고, 어느 모듈에서든 같은 뜻이다." 이 예제는 그 규칙으로 쓸 수 있는 모든 시작점을 보여 줍니다.
// 각 구역의 주석에 2015 에디션에서는 어떻게 달랐는지 적었습니다.
// 이 예제는 2024 에디션으로 컴파일됩니다.

// 크레이트에 다른 이름 붙이기. 2015에서는 `extern crate std as stdlib;`이라고 썼습니다.
// 2018부터는 크레이트 이름도 다른 항목처럼 `use`로 별칭을 붙입니다.
use std as stdlib;

// extern crate가 아직 필요한 곳: Cargo가 외부 프렐류드에 넣어 주지 않는 sysroot 크레이트(alloc 등).
// (Part 1 ch1 참고) 이 줄을 지우면 아래의 alloc:: 경로에서 E0433이 납니다.
extern crate alloc;

mod shop {
    pub fn name() -> &'static str {
        "상점"
    }

    pub mod cart {
        fn label() -> &'static str {
            "장바구니"
        }

        // 1. 절대 경로는 crate::로 시작합니다. 2015 시절 코드에서는 같은 경로를 ::shop::name()으로,
        //    use 안에서는 shop::name으로 썼습니다(지금 툴체인은 2015 에디션에서도 crate::를 받아 줍니다).
        pub fn via_crate() -> &'static str {
            crate::shop::name()
        }

        // 2. 상대 경로: super::는 부모 모듈, self::는 현재 모듈입니다. 2015에서도 같습니다.
        pub fn via_super() -> &'static str {
            super::name()
        }
        pub fn via_self() -> &'static str {
            self::label()
        }

        // 3. use도 현재 모듈에서 출발합니다. 자식 모듈 price를 self:: 없이 바로 씁니다.
        //    2015에서는 use 경로가 크레이트 루트에서 출발했으므로 `use self::price::total;`이어야 했습니다.
        mod price {
            pub fn total() -> u32 {
                4500
            }
        }
        use price::total;
        pub fn checkout() -> u32 {
            total()
        }

        // 4. 열거형 배리언트를 함수 안에서 use로 가져오기. 2015에서는 이 모듈이 크레이트 루트가
        //    아니므로 `use self::Size::*;`라고 써야 했습니다.
        pub enum Size {
            Small,
            Large,
        }
        pub fn describe(size: Size) -> &'static str {
            use Size::*;
            match size {
                Small => "작음",
                Large => "큼",
            }
        }

        // 5. 외부 크레이트 이름(외부 프렐류드)은 어느 깊이의 모듈에서나, use 안에서도 밖에서도 그대로 씁니다.
        //    2015에서는 use 안에서 외부 크레이트를 쓰려면 크레이트 루트에 extern crate가 있어야 했습니다
        //    (std만은 컴파일러가 크레이트 루트에 자동으로 넣어 주었습니다).
        pub fn bigger(a: u32, b: u32) -> u32 {
            std::cmp::max(a, b)
        }
    }
}

// 6. 지역 모듈이 외부 크레이트와 이름이 같으면, 그 이름으로 시작하는 경로는 지역 모듈을 가리킵니다.
//    외부 크레이트를 콕 집어 가리킬 때는 ::를 앞에 붙입니다. 2018부터 ::는 외부 크레이트 전용입니다.
mod core {
    pub fn answer() -> u32 {
        42
    }
}

fn main() {
    use shop::cart;

    println!("1. 절대 경로 crate::");
    println!("    crate::shop::name() = {}", cart::via_crate()); // 출력:     crate::shop::name() = 상점

    println!("2. 상대 경로 super::, self::");
    println!("    super::name() = {}", cart::via_super()); // 출력:     super::name() = 상점
    println!("    self::label() = {}", cart::via_self()); // 출력:     self::label() = 장바구니

    println!("3. use도 현재 모듈 기준");
    println!("    use price::total; total() = {}", cart::checkout()); // 출력:     use price::total; total() = 4500

    println!("4. 함수 안의 use Size::*");
    println!(
        "    Small = {}, Large = {}",
        cart::describe(cart::Size::Small),
        cart::describe(cart::Size::Large)
    ); // 출력:     Small = 작음, Large = 큼

    println!("5. 외부 크레이트 이름은 어디서나");
    println!("    std::cmp::max(3, 8) = {}", cart::bigger(3, 8)); // 출력:     std::cmp::max(3, 8) = 8

    println!("6. 지역 모듈 core와 외부 크레이트 ::core");
    println!("    core::answer()          = {}", core::answer()); // 출력:     core::answer()          = 42
    println!("    ::core::cmp::min(3, 8)  = {}", ::core::cmp::min(3, 8)); // 출력:     ::core::cmp::min(3, 8)  = 3

    println!("7. 별칭을 붙인 크레이트와 extern crate alloc");
    println!("    stdlib::cmp::max(3, 8) = {}", stdlib::cmp::max(3, 8)); // 출력:     stdlib::cmp::max(3, 8) = 8
    let shared = alloc::rc::Rc::new(5);
    println!(
        "    alloc::rc::Rc 강한 참조 수 = {}",
        alloc::rc::Rc::strong_count(&shared)
    ); // 출력:     alloc::rc::Rc 강한 참조 수 = 1
}
