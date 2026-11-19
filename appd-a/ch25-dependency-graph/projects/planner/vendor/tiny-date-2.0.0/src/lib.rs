//! 연습용 tiny-date 2.0.0: `Date::new`가 `Option`을 돌려주도록 바뀌었습니다(호환되지 않는 변경).

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
    /// 2.0.0의 호환되지 않는 변경: 없는 날짜면 None을 돌려줍니다.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Date> {
        if day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Date { year, month, day })
    }

    /// 연, 월, 일 게터(1.6.0에서 생김).
    pub fn year(&self) -> i32 {
        self.year
    }
    pub fn month(&self) -> u32 {
        self.month
    }
    pub fn day(&self) -> u32 {
        self.day
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// 그 달의 날 수입니다(1.4.0에서 생김). 달이 1~12가 아니면 0입니다.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    }
}
