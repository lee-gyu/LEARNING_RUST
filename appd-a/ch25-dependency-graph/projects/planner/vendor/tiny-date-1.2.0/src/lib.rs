//! 연습용 tiny-date 1.2.0: 검사하지 않는 `Date::new`와 `Display`만 있습니다.

use std::fmt;

/// 이 사본의 버전입니다. 그래프에 어느 사본이 들어왔는지 실행 결과로 보려고 공개합니다.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 연, 월, 일만 담는 날짜입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl Date {
    /// 1.x의 생성자는 값을 검사하지 않습니다.
    pub fn new(year: i32, month: u32, day: u32) -> Date {
        Date { year, month, day }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
