//! 연습용 tiny-date 2.1.0: 2.0.0에 `weekday`를 더했습니다.

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

    /// 요일(2.1.0에서 생김). 사카모토 도모히코의 계산법입니다.
    pub fn weekday(&self) -> &'static str {
        const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if self.month < 3 {
            self.year - 1
        } else {
            self.year
        };
        let m = (self.month - 1) as usize;
        let n = (y + y / 4 - y / 100 + y / 400 + T[m] + self.day as i32).rem_euclid(7);
        ["일", "월", "화", "수", "목", "금", "토"][n as usize]
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
