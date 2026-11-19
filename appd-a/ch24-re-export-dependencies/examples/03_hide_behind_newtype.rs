//! 의존성의 타입을 공개 API에 드러내지 않고 자기 타입(뉴타입)으로 감싸 두면, 라이브러리가 그 의존성의
//! 메이저 버전을 올려도 사용자 코드가 그대로 컴파일된다는 것을 봅니다(책에 없는 지침, README 2.3절).
//!
//! 실행: cargo run -p er-ch24-re-export-dependencies --example 03_hide_behind_newtype
//!
//! 모듈 `money_0_3`, `money_0_4`는 의존 크레이트 money의 두 버전, `receipt_*`는 라이브러리 receipt의
//! 여러 릴리스라고 칩니다. 같은 사용자 코드를 매크로로 여러 릴리스에 대해 펼쳐 결과를 비교합니다.

/// money 0.3.2라고 칩니다.
pub mod money_0_3 {
    #[derive(Debug, Clone, Copy)]
    pub struct Money {
        won: i64,
    }
    pub fn won(amount: i64) -> Money {
        Money { won: amount }
    }
    impl Money {
        pub fn as_won(self) -> i64 {
            self.won
        }
    }
}

/// money 0.4.0이라고 칩니다. 통화를 함께 갖도록 바뀌면서 `as_won`이 `minor_units`로 바뀌었습니다.
pub mod money_0_4 {
    #[derive(Debug, Clone, Copy)]
    pub enum Currency {
        Krw,
    }
    #[derive(Debug, Clone, Copy)]
    pub struct Money {
        minor: i64,
        currency: Currency,
    }
    pub fn won(amount: i64) -> Money {
        Money {
            minor: amount,
            currency: Currency::Krw,
        }
    }
    impl Money {
        pub fn minor_units(self) -> i64 {
            self.minor
        }
        pub fn currency(self) -> Currency {
            self.currency
        }
    }
}

/// 드러낸 판, receipt 1.1.0: money 0.3을 API에 쓰고 재내보냅니다.
pub mod receipt_exposed_1_1 {
    pub use crate::money_0_3 as money;

    pub fn total(prices: &[money::Money]) -> money::Money {
        money::won(prices.iter().map(|p| p.as_won()).sum())
    }
}

/// 드러낸 판에서 money를 0.4로 올린 receipt. 재내보낸 `money`의 내용이 바뀌었으므로 메이저 변경(2.0.0)입니다.
/// `main`에서 부르는 곳은 주석으로 막아 두었습니다(풀면 E0599).
pub mod receipt_exposed_2_0 {
    pub use crate::money_0_4 as money;

    pub fn total(prices: &[money::Money]) -> money::Money {
        money::won(prices.iter().map(|p| p.minor_units()).sum())
    }
}

/// 감춘 판, money 0.3 위에 구현한 receipt 1.0.0
pub mod receipt_hidden_1_0 {
    use crate::money_0_3 as money;

    /// 금액. money의 `Money`를 감싸지만 필드가 비공개라서 공개 API에는 `Price`만 보입니다.
    #[derive(Debug, Clone, Copy)]
    pub struct Price(money::Money);

    impl Price {
        pub fn won(amount: i64) -> Price {
            Price(money::won(amount))
        }
        pub fn as_won(self) -> i64 {
            self.0.as_won()
        }
    }

    pub fn total(prices: &[Price]) -> Price {
        Price::won(prices.iter().map(|p| p.as_won()).sum())
    }
}

/// 감춘 판, 안에서 money를 0.4로 올린 receipt 1.1.0. 공개 API는 1.0.0과 글자 하나 다르지 않습니다.
pub mod receipt_hidden_1_1 {
    use crate::money_0_4::{self as money, Currency};

    #[derive(Debug, Clone, Copy)]
    pub struct Price(money::Money);

    impl Price {
        pub fn won(amount: i64) -> Price {
            Price(money::won(amount))
        }
        pub fn as_won(self) -> i64 {
            // 0.4의 바뀐 API에 맞추는 일은 receipt 안에서 끝납니다.
            match self.0.currency() {
                Currency::Krw => self.0.minor_units(),
            }
        }
    }

    pub fn total(prices: &[Price]) -> Price {
        Price::won(prices.iter().map(|p| p.as_won()).sum())
    }
}

/// 드러낸 판의 사용자 코드: receipt가 재내보낸 money로 값을 만들고, money의 메서드로 결과를 읽습니다.
macro_rules! exposed_user_code {
    ($receipt:ident) => {{
        let prices = [$receipt::money::won(4500), $receipt::money::won(3200)];
        $receipt::total(&prices).as_won()
    }};
}

/// 감춘 판의 사용자 코드: receipt의 `Price`만 씁니다.
macro_rules! hidden_user_code {
    ($receipt:ident) => {{
        let prices = [$receipt::Price::won(4500), $receipt::Price::won(3200)];
        $receipt::total(&prices).as_won()
    }};
}

fn main() {
    println!("드러낸 판");
    let a = exposed_user_code!(receipt_exposed_1_1);
    println!("  receipt 1.1.0 (money 0.3): {a}원"); // 출력:   receipt 1.1.0 (money 0.3): 7700원
    // 다음 줄의 주석을 풀면 E0599(no method named `as_won` found for struct `money_0_4::Money`)입니다.
    // 사용자는 receipt만 올렸는데, receipt가 내보낸 money의 API가 바뀌어 사용자 코드가 깨집니다.
    // let b = exposed_user_code!(receipt_exposed_2_0);

    println!("감춘 판");
    let c = hidden_user_code!(receipt_hidden_1_0);
    println!("  receipt 1.0.0 (money 0.3): {c}원"); // 출력:   receipt 1.0.0 (money 0.3): 7700원
    let d = hidden_user_code!(receipt_hidden_1_1);
    println!("  receipt 1.1.0 (money 0.4): {d}원"); // 출력:   receipt 1.1.0 (money 0.4): 7700원
}
