//! 연습용 remind-kit 0.4.3: 공개 API는 0.4.2와 같고, 안에서 쓰는 tiny-date만 2.x로 올렸습니다.
//! tiny-date의 타입이 공개 API에 없으므로 이 변경은 호환되는 변경이라 0.4.3으로 냈습니다.

/// 이 크레이트가 링크한 tiny-date 사본의 버전입니다.
pub const TINY_DATE_VERSION: &str = tiny_date::VERSION;

/// 알림 문구를 만듭니다. 0.4.3부터 요일을 붙입니다.
pub fn reminder(year: i32, month: u32, day: u32, what: &str) -> String {
    match tiny_date::Date::new(year, month, day) {
        Some(date) => format!("[알림] {date}({}) {what}", date.weekday()),
        None => format!("[알림] (없는 날짜) {what}"),
    }
}
