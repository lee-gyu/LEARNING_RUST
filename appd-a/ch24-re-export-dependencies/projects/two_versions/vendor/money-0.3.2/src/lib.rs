//! money 0.3.2 (연습용 레지스트리의 가짜 크레이트)
//!
//! 원 단위 금액 `Money`, 금액을 만드는 함수 `won`, 값에 가격을 매기는 트레이트 `Priced`를 제공합니다.

use std::fmt;
use std::iter::Sum;
use std::ops::Add;

pub const VERSION: &str = "0.3.2";

/// 원 단위 금액입니다. 필드가 비공개라서 이 크레이트 밖에서는 `won` 함수로만 만들 수 있습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Money {
    won: i64,
}

/// `amount`원짜리 금액을 만듭니다.
pub fn won(amount: i64) -> Money {
    Money { won: amount }
}

impl Money {
    /// 원 단위 정수로 돌려줍니다.
    pub fn as_won(self) -> i64 {
        self.won
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, other: Money) -> Money {
        Money {
            won: self.won + other.won,
        }
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::default(), Add::add)
    }
}

/// `4,500원`처럼 세 자리마다 쉼표를 넣어 씁니다.
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}원", with_commas(self.won))
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
