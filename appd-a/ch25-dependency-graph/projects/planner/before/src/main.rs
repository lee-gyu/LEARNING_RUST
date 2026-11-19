//! planner 0.1.0: tiny-date 2.x를 직접 쓰고, 1.x를 쓰는 중간 크레이트 둘에도 의존합니다.

fn main() {
    let deadline = tiny_date::Date::new(2026, 11, 22).expect("있는 날짜여야 합니다");
    println!("마감일: {deadline}");
    println!("{}", remind_kit::reminder(2026, 11, 22, "원고 마감"));
    println!("{}", cal_grid::month_title(2026, 11));

    // 각 크레이트가 링크한 tiny-date 사본의 버전입니다. 값이 둘이면 사본도 둘입니다.
    println!("tiny-date 사본");
    println!("  planner가 쓰는 것:    {}", tiny_date::VERSION);
    println!("  remind-kit이 쓰는 것: {}", remind_kit::TINY_DATE_VERSION);
    println!("  cal-grid가 쓰는 것:   {}", cal_grid::TINY_DATE_VERSION);
}
