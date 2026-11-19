//! 주간 보고서의 제목 줄을 만드는 크레이트. datefmt의 기능은 하나도 켜지 않습니다.

pub use datefmt::Date;

/// `[주간 보고] 2026-11-19` 모양의 제목을 만듭니다.
pub fn title(d: &Date) -> String {
    format!("[주간 보고] {}", datefmt::iso(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_title() {
        let d = Date {
            year: 2026,
            month: 11,
            day: 19,
        };
        assert_eq!(title(&d), "[주간 보고] 2026-11-19");
    }
}
