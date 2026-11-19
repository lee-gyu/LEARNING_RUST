//! 청구서의 발행일 줄을 만드는 크레이트. datefmt의 korean 기능을 씁니다.

pub use datefmt::Date;

/// `발행일: 2026년 11월 19일` 모양의 줄을 만듭니다.
pub fn issued(d: &Date) -> String {
    format!("발행일: {}", datefmt::render(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn korean_issued() {
        let d = Date {
            year: 2026,
            month: 11,
            day: 19,
        };
        assert_eq!(issued(&d), "발행일: 2026년 11월 19일");
    }
}
