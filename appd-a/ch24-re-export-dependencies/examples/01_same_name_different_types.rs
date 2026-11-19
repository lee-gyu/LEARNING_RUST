//! 같은 크레이트의 두 버전을 모듈 두 개로 흉내 내어 두 가지를 봅니다.
//! (1) 출처(경로)가 다르면 이름과 정의가 같아도 다른 타입입니다.
//! (2) `pub use`는 새 타입을 만들지 않고, 같은 타입에 닿는 경로를 하나 더 만듭니다.
//!
//! 실행: cargo run -p er-ch24-re-export-dependencies --example 01_same_name_different_types
//!
//! 실제로는 Cargo가 money 0.3.2와 0.4.0을 서로 다른 크레이트로 컴파일합니다(README 2.1절).
//! 컴파일러가 두 크레이트를 구별하는 방식은 한 크레이트 안의 두 모듈을 구별하는 방식과 같으므로,
//! 여기서는 모듈 `money_0_3`, `money_0_4`를 두 버전이라고 칩니다.

use std::any::{Any, TypeId, type_name};

/// 의존 크레이트 money 0.3.2라고 칩니다.
pub mod money_0_3 {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Money {
        pub won: i64,
    }
}

/// 같은 크레이트의 0.4.0이라고 칩니다. 차이를 지우려고 정의를 0.3과 글자 하나 다르지 않게 두었습니다.
pub mod money_0_4 {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Money {
        pub won: i64,
    }
}

/// money 0.3에 의존하고, 그것을 `pub use`로 재내보내는 라이브러리 receipt라고 칩니다.
pub mod receipt {
    pub use crate::money_0_3 as money;

    /// 공개 API에 money 0.3의 `Money`가 드러납니다.
    pub fn line(name: &str, price: money::Money) -> String {
        format!("{name}: {}원", price.won)
    }
}

/// 두 타입이 컴파일러에게 같은 타입인지 `TypeId`로 확인합니다.
fn same_type<A: 'static, B: 'static>() -> bool {
    TypeId::of::<A>() == TypeId::of::<B>()
}

fn main() {
    println!("1) 정의가 같아도 출처가 다르면 다른 타입");
    println!("   {}", type_name::<money_0_3::Money>());
    // 출력:    01_same_name_different_types::money_0_3::Money
    println!("   {}", type_name::<money_0_4::Money>());
    // 출력:    01_same_name_different_types::money_0_4::Money
    let same = same_type::<money_0_3::Money, money_0_4::Money>();
    println!("   같은 타입인가? {same}"); // 출력:    같은 타입인가? false

    // 아래 줄의 주석을 풀면 E0308(expected `money_0_3::Money`, found `money_0_4::Money`)입니다.
    // 실제 크레이트 두 버전에서는 두 타입의 경로가 모두 `money::Money`로 보입니다(README 4.1절).
    // receipt::line("카페라테", money_0_4::Money { won: 5000 });

    println!("2) pub use는 같은 타입에 닿는 경로를 하나 더 만든다");
    let via_receipt = same_type::<receipt::money::Money, money_0_3::Money>();
    println!("   receipt::money::Money = money_0_3::Money? {via_receipt}");
    // 출력:    receipt::money::Money = money_0_3::Money? true
    let other = same_type::<receipt::money::Money, money_0_4::Money>();
    println!("   receipt::money::Money = money_0_4::Money? {other}");
    // 출력:    receipt::money::Money = money_0_4::Money? false
    // 재내보낸 경로로 만든 값은 receipt가 기대하는 바로 그 타입이므로 그대로 넘길 수 있습니다.
    let latte = receipt::money::Money { won: 5000 };
    println!("   {}", receipt::line("카페라테", latte)); // 출력:    카페라테: 5000원

    println!("3) 컴파일러가 잡지 못하는 곳: 다운캐스트는 실행 중에 실패");
    // 타입을 지운 `dyn Any`로 경계를 넘기면 컴파일 에러가 없습니다. 꺼낼 때 TypeId를 비교하므로
    // 다른 버전의 타입으로는 꺼내지 못하고 None이 나옵니다.
    let boxed: Box<dyn Any> = Box::new(money_0_4::Money { won: 4500 });
    let as_0_3 = boxed.downcast_ref::<money_0_3::Money>();
    let as_0_4 = boxed.downcast_ref::<money_0_4::Money>();
    println!("   0.3의 Money로 꺼내기: {as_0_3:?}"); // 출력:    0.3의 Money로 꺼내기: None
    println!("   0.4의 Money로 꺼내기: {as_0_4:?}"); // 출력:    0.4의 Money로 꺼내기: Some(Money { won: 4500 })

    println!("4) 경계에서 원시 타입으로 옮겨 담으면 두 버전이 함께 있어도 문제없음");
    let mine = money_0_4::Money { won: 3200 }; // 사용자 자신은 0.4를 씀
    let converted = receipt::money::Money { won: mine.won };
    println!("   {}", receipt::line("마들렌", converted)); // 출력:    마들렌: 3200원
}
