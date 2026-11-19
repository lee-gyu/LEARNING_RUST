//! 공개 API에 드러난 의존성을 재내보낼 때, 타입만 내보낸 판과 크레이트 전체를 내보낸 판에서
//! 사용자가 할 수 있는 일이 어떻게 다른지 비교합니다.
//!
//! 실행: cargo run -p er-ch24-re-export-dependencies --example 02_re_export_type_or_crate
//!
//! 모듈 `money`는 receipt가 의존하는 money 0.3이라고 칩니다. 사용자 코드(`with_types_only`,
//! `with_whole_crate`)는 자기 `money`로 다른 버전(0.4)을 쓴다고 보므로, money 0.3에는
//! receipt가 재내보낸 경로로만 닿습니다. 이 파일에서 `crate::money`를 직접 쓰는 것은 receipt뿐입니다.

use std::fmt;

/// 의존 크레이트 money 0.3이라고 칩니다.
pub mod money {
    use std::fmt;

    /// 원 단위 금액. 필드가 비공개라서 money 밖에서는 `won` 함수로만 만들 수 있습니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Money {
        won: i64,
    }

    /// 생성자가 `Money`의 연관 함수가 아니라 크레이트 수준 함수입니다.
    pub fn won(amount: i64) -> Money {
        Money { won: amount }
    }

    impl Money {
        pub fn as_won(self) -> i64 {
            self.won
        }
    }

    impl fmt::Display for Money {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}원", self.won)
        }
    }

    /// 가격을 매길 수 있는 것
    pub trait Priced {
        fn price(&self) -> Money;
    }
}

/// 두 판이 함께 쓰는 receipt의 구현입니다. 공개 API에 money의 `Money`와 `Priced`가 드러납니다.
pub mod receipt_impl {
    use crate::money::{self, Money, Priced};
    use std::fmt::Display;

    #[derive(Debug, Default)]
    pub struct Receipt {
        lines: Vec<(String, Money)>,
    }

    impl Receipt {
        pub fn new() -> Receipt {
            Receipt::default()
        }
        pub fn add_won(&mut self, name: &str, won: i64) {
            self.add(name, money::won(won));
        }
        pub fn add(&mut self, name: &str, price: Money) {
            self.lines.push((name.to_string(), price));
        }
        pub fn add_item<T: Priced + Display>(&mut self, item: &T) {
            self.add(&item.to_string(), item.price());
        }
        pub fn total(&self) -> Money {
            money::won(self.lines.iter().map(|(_, p)| p.as_won()).sum())
        }
    }
}

/// receipt의 공개 경로, 타입만 재내보낸 판
pub mod receipt_types_only {
    pub use crate::money::{Money, Priced};
    pub use crate::receipt_impl::Receipt;
}

/// receipt의 공개 경로, 크레이트 전체를 재내보낸 판
pub mod receipt_whole_crate {
    pub use crate::money;
    pub use crate::receipt_impl::Receipt;
}

/// 타입만 재내보낸 판의 사용자: 이름은 적을 수 있지만 값은 만들 수 없습니다.
fn with_types_only() {
    use receipt_types_only::{Money, Receipt};

    // `Money`라는 이름을 적을 수 있으므로, receipt에게서 받은 값을 다루는 함수를 쓸 수 있습니다.
    fn vat_included(total: Money) -> String {
        format!("{total} (부가세 {}원 포함)", total.as_won() / 11)
    }

    let mut receipt = Receipt::new();
    receipt.add_won("아메리카노", 4500);
    receipt.add_won("마들렌", 3200);
    println!("  {}", vat_included(receipt.total()));
    // 출력:   7700원 (부가세 700원 포함)

    // money의 생성자 `won`은 재내보내지 않았으므로 receipt를 거쳐서는 부를 수 없습니다.
    // receipt.add("카페라테", receipt_types_only::money::won(5000)); // E0433, README 4.3절
    // `Priced`라는 이름은 보이지만 구현할 수 없습니다. `price`가 돌려줄 `Money`를 만들 방법이 없습니다.
}

/// 크레이트 전체를 재내보낸 판의 사용자: money 0.3으로 하던 일을 그대로 할 수 있습니다.
fn with_whole_crate() {
    use receipt_whole_crate::{Receipt, money};

    struct Latte;
    impl fmt::Display for Latte {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("카페라테")
        }
    }
    // 재내보낸 경로의 트레이트를 구현하므로 receipt가 요구하는 바로 그 `Priced`입니다.
    impl money::Priced for Latte {
        fn price(&self) -> money::Money {
            money::won(5000)
        }
    }

    let mut receipt = Receipt::new();
    receipt.add_won("아메리카노", 4500);
    receipt.add("마들렌", money::won(3200)); // 생성자 함수를 쓸 수 있음
    receipt.add_item(&Latte); // 트레이트를 구현해 넘길 수 있음
    println!("  합계 {}", receipt.total());
    // 출력:   합계 12700원
}

fn main() {
    println!("타입만 재내보낸 receipt");
    with_types_only();
    println!("크레이트 전체를 재내보낸 receipt");
    with_whole_crate();
}
