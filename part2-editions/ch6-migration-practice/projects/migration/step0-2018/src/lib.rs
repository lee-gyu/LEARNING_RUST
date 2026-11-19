//! 작은 가계부 라이브러리입니다.
//!
//! ```
//! use ledger::{report, Ledger};
//!
//! let mut book = Ledger::new();
//! book.add_line("식비 12000 점심").unwrap();
//! let style: Box<report::Style> = report::style_for("plain");
//! assert_eq!(
//!     report::render(&book, &*style),
//!     "식비   12000원 100%\n합계 12000원 (알뜰)"
//! );
//! ```

pub mod book;
#[cfg(feature = "csv")]
pub mod csv;
pub mod entry;
pub mod report;

pub use crate::book::{Audit, IdGen, Ledger};
pub use crate::entry::{Category, Entry};

/// 실행 환경에서 읽는 설정
pub struct Config {
    /// 금액 단위. 환경 변수 `LEDGER_UNIT`으로 바꿀 수 있고, 없으면 "원"입니다.
    pub unit: String,
}

impl Config {
    /// ```
    /// std::env::set_var("LEDGER_UNIT", "달러");
    /// assert_eq!(ledger::Config::from_env().unit, "달러");
    /// ```
    pub fn from_env() -> Config {
        Config {
            unit: std::env::var("LEDGER_UNIT").unwrap_or_else(|_| String::from("원")),
        }
    }
}

/// 예산 한도
#[derive(Debug, PartialEq)]
pub enum Budget {
    Limit(i64),
    Unlimited,
}

impl Budget {
    pub fn allows(&self, spent: i64) -> bool {
        match *self {
            Budget::Limit(limit) => spent <= limit,
            Budget::Unlimited => true,
        }
    }
}

/// C 프로그램에서 부르는 함수: 금액을 10원 단위로 반올림합니다.
#[no_mangle]
pub extern "C" fn ledger_round10(amount: i64) -> i64 {
    (amount + 5).div_euclid(10) * 10
}

/// 줄 여러 개로 가계부를 만듭니다. 잘못된 줄이 있으면 그 이유를 메시지로 패닉합니다.
#[macro_export]
macro_rules! book {
    ($($line:expr),* $(,)?) => {{
        let mut book = $crate::Ledger::new();
        $(
            if let Err(e) = book.add_line($line) {
                panic!(e);
            }
        )*
        book
    }};
}

/// 예산 한도: `budget!(30_000)`은 그 금액까지, `budget!(_)`은 한도 없음
#[macro_export]
macro_rules! budget {
    ($limit:expr) => {
        $crate::Budget::Limit($limit)
    };
    (_) => {
        $crate::Budget::Unlimited
    };
}
