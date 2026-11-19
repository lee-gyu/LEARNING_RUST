//! 권장: 더하기만 하는 기능을 가진 날짜 서식 라이브러리.
//!
//! broken/의 datefmt는 같은 이름 `render`를 기능마다 다르게 정의했습니다. 여기서는 서식마다
//! 이름이 다른 함수를 두고, 기능은 함수를 **더하기만** 합니다. 그래서 그래프의 누가 어떤 기능을
//! 켜든 `iso`의 결과는 바뀌지 않고, 두 기능이 함께 켜져도 부딪히지 않습니다.

/// 연, 월, 일. 이 타입은 기능과 상관없이 늘 있습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

/// 날짜를 ISO 8601 모양("2026-11-19")으로 씁니다. 기능과 상관없이 늘 있습니다.
pub fn iso(d: &Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

/// 날짜를 요일을 붙인 한국어 모양("2026년 11월 19일 (목)")으로 씁니다.
///
/// `korean` 기능이 켜졌을 때만 있습니다. 이 함수를 부르는 크레이트는 자기 `Cargo.toml`에서
/// 그 기능을 직접 켜야 합니다.
#[cfg(feature = "korean")]
pub fn korean(d: &Date) -> String {
    let w = weekday::korean_short(d.year, d.month, d.day);
    format!("{}년 {}월 {}일 ({w})", d.year, d.month, d.day)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: Date = Date {
        year: 2026,
        month: 11,
        day: 19,
    };

    #[test]
    fn iso_is_always_there() {
        assert_eq!(iso(&DAY), "2026-11-19");
    }

    #[cfg(feature = "korean")]
    #[test]
    fn korean_with_weekday() {
        assert_eq!(korean(&DAY), "2026년 11월 19일 (목)");
    }
}
