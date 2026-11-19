//! 연습용 remind-kit 0.4.2: tiny-date 1.x를 안에서만 씁니다.
//! 공개 API에 tiny-date의 타입이 드러나지 않으므로, 사용자가 다른 버전의 tiny-date를 써도 컴파일 에러가 나지 않습니다.

/// 이 크레이트가 링크한 tiny-date 사본의 버전입니다.
pub const TINY_DATE_VERSION: &str = tiny_date::VERSION;

/// 알림 문구를 만듭니다.
pub fn reminder(year: i32, month: u32, day: u32, what: &str) -> String {
    let date = tiny_date::Date::new(year, month, day);
    format!("[알림] {date} {what}")
}
