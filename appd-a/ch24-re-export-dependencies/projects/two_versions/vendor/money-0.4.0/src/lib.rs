//! money 0.4.0 (연습용 레지스트리의 가짜 크레이트)
//!
//! 0.3에서 0.4로 오며 호환되지 않게 바뀐 것(0.x에서는 둘째 자리가 메이저입니다):
//! - `Money`가 통화를 함께 가집니다. `as_won`이 없어지고 `minor_units`와 `currency`가 생겼습니다.
//! - 통화가 다르면 더할 수 없으므로 `Add`와 `Sum` 구현을 떼고 `checked_add`를 두었습니다.
//!
//! 새로 생긴 것: 달러 금액을 만드는 `usd`.
//! 바뀌지 않은 것: `won` 함수와 `Priced` 트레이트는 이름과 모양이 0.3과 같습니다.

use std::fmt;

pub const VERSION: &str = "0.4.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Krw,
    Usd,
}

/// 통화와 최소 단위(원, 센트)의 수로 나타낸 금액입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    minor: i64,
    currency: Currency,
}

/// `amount`원짜리 금액을 만듭니다.
pub fn won(amount: i64) -> Money {
    Money {
        minor: amount,
        currency: Currency::Krw,
    }
}

/// `cents`센트짜리 달러 금액을 만듭니다.
pub fn usd(cents: i64) -> Money {
    Money {
        minor: cents,
        currency: Currency::Usd,
    }
}

impl Money {
    /// 최소 단위(원 또는 센트)의 수
    pub fn minor_units(self) -> i64 {
        self.minor
    }

    pub fn currency(self) -> Currency {
        self.currency
    }

    /// 통화가 같을 때만 더합니다.
    pub fn checked_add(self, other: Money) -> Option<Money> {
        (self.currency == other.currency).then(|| Money {
            minor: self.minor + other.minor,
            currency: self.currency,
        })
    }
}

/// 원은 `4,500원`, 달러는 `$3.50`처럼 씁니다.
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.currency {
            Currency::Krw => write!(f, "{}원", with_commas(self.minor)),
            Currency::Usd => write!(f, "${}.{:02}", self.minor / 100, self.minor % 100),
        }
    }
}

/// 가격을 매길 수 있는 것
pub trait Priced {
    fn price(&self) -> Money;
}

fn with_commas(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}
