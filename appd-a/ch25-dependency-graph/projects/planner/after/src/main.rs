//! planner 0.2.0: 중간 크레이트를 tiny-date 2.x를 쓰는 버전으로 올려 사본을 하나로 줄였습니다.

fn main() {
    let deadline = tiny_date::Date::new(2026, 11, 22).expect("있는 날짜여야 합니다");
    println!("마감일: {deadline}");
    println!("{}", remind_kit::reminder(2026, 11, 22, "원고 마감"));
    // cal-grid 0.10.0에서 반환 타입이 Option<String>으로 바뀌어 이 줄을 고쳐야 했습니다.
    let title = cal_grid::month_title(2026, 11).expect("11월은 있는 달입니다");
    println!("{title}");

    // 각 크레이트가 링크한 tiny-date 사본의 버전입니다. 값이 하나면 사본도 하나입니다.
    println!("tiny-date 사본");
    println!("  planner가 쓰는 것:    {}", tiny_date::VERSION);
    println!("  remind-kit이 쓰는 것: {}", remind_kit::TINY_DATE_VERSION);
    println!("  cal-grid가 쓰는 것:   {}", cal_grid::TINY_DATE_VERSION);
}
