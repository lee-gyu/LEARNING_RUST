//! 가계부 예시 프로그램: 내장된 예시 자료로 월말 보고서를 출력합니다.
//!
//! 사용법: `cargo run` 또는 `cargo run -- bar`

use ledger::{book, budget, report, Category, Config};

fn main() {
    let style_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("plain"));
    let config = Config::from_env();

    let mut book = book![
        "식비 32000 장보기",
        "교통 12500 교통카드 충전",
        "식비 -4000 환불",
        "기타 8000 우산",
    ];

    let style = report::style_for(&style_name);
    println!("{}", book.close_month(&*style));
    println!("(금액 단위: {})", config.unit);

    let food = budget!(30_000);
    let etc = budget!(_);
    println!(
        "예산: 식비 {}, 기타 {}",
        food.allows(book.total_of(Category::Food)),
        etc.allows(book.total_of(Category::Etc))
    );

    // 번호 목록을 쥔 채로 새 항목을 더합니다.
    let ids = book.ids();
    let new_id = book.push_line("교통 1250 버스");
    println!(
        "기존 번호 {:?}, 새 번호 {}",
        ids.collect::<Vec<_>>(),
        new_id
    );

    println!("감사 기록:");
    for event in book.audit.events() {
        println!("  {}", event);
    }
}
