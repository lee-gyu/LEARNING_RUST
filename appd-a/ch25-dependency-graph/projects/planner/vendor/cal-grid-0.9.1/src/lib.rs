//! 연습용 cal-grid 0.9.1: tiny-date 1.x를 안에서만 씁니다.

/// 이 크레이트가 링크한 tiny-date 사본의 버전입니다.
pub const TINY_DATE_VERSION: &str = tiny_date::VERSION;

/// 달력 위에 붙일 제목을 만듭니다.
pub fn month_title(year: i32, month: u32) -> String {
    format!(
        "{year}년 {month}월 ({}일)",
        tiny_date::days_in_month(year, month)
    )
}
