//! 연습용 cal-grid 0.10.0: tiny-date 2.x로 올리면서 `month_title`의 반환 타입을 바꿨습니다.
//! 공개 API가 바뀌었으므로 0.9 → 0.10(호환되지 않는 변경)입니다.

/// 이 크레이트가 링크한 tiny-date 사본의 버전입니다.
pub const TINY_DATE_VERSION: &str = tiny_date::VERSION;

/// 달력 위에 붙일 제목을 만듭니다. 없는 달이면 `None`입니다.
pub fn month_title(year: i32, month: u32) -> Option<String> {
    tiny_date::Date::new(year, month, 1)?;
    Some(format!(
        "{year}년 {month}월 ({}일)",
        tiny_date::days_in_month(year, month)
    ))
}
