//! 가계부의 동작을 고정하는 통합 테스트입니다.
//! 에디션을 옮길 때 "동작이 그대로인지" 확인하는 안전망 역할을 합니다.

use ledger::{Category, Config, Entry, Ledger, book, budget, report};

// C 프로그램이 하듯 심벌 이름만으로 선언해서 부릅니다.
// SAFETY: 선언은 src/lib.rs의 `ledger_round10`과 시그니처가 같습니다. 그 함수는 어떤 i64로
// 불러도 미정의 동작이 없으므로 `safe fn`으로 선언합니다.
unsafe extern "C" {
    safe fn ledger_round10(amount: i64) -> i64;
}

#[test]
fn parses_a_line() {
    let e = Entry::parse("교통 1250 버스 환승").unwrap();
    assert_eq!(e.category, Category::Transport);
    assert_eq!(e.amount, 1250);
    assert_eq!(e.memo, "버스 환승");
    assert_eq!(
        Entry::parse("간식 500").unwrap_err(),
        "모르는 카테고리: 간식"
    );
}

#[test]
#[should_panic(expected = "잘못된 줄: 식비 abc")]
fn push_line_panics_on_bad_line() {
    Ledger::new().push_line("식비 abc");
}

#[test]
fn report_lines() {
    let book = book!["식비 30000 장보기", "교통 10000 충전"];
    assert_eq!(
        report::render(&book, &report::Plain),
        "식비   30000원  75%\n교통   10000원  25%\n합계 40000원 (알뜰)"
    );
}

#[test]
fn report_hides_odd_percent() {
    // 환불 때문에 식비가 총액의 150%가 됩니다. 이런 비율은 "-"로 표시합니다.
    let book = book!["식비 30000 장보기", "기타 -10000 환불"];
    assert_eq!(
        report::render(&book, &report::Plain),
        "식비   30000원    -\n기타  -10000원    -\n합계 20000원 (알뜰)"
    );
}

#[test]
fn totals_are_cached_and_reset() {
    let mut book = book!["식비 1000 물"];
    assert_eq!(book.total_of(Category::Food), 1000);
    assert_eq!(book.total_of(Category::Food), 1000);
    book.push_line("식비 2000 빵");
    assert_eq!(book.total_of(Category::Food), 3000);
}

#[test]
fn ids_count_up() {
    let mut book = Ledger::new();
    assert_eq!(book.push_line("식비 1000 물"), 1);
    assert_eq!(book.push_line("식비 2000 빵"), 2);
    assert_eq!(book.ids().collect::<Vec<_>>(), [1, 2]);
}

#[test]
fn balance_audit() {
    let book = Ledger::new();
    assert_eq!(book.balance(), 0);
    assert_eq!(
        book.audit.events(),
        ["잔액 시작", "합산 시작", "합산 끝", "잔액 끝"]
    );
}

#[test]
fn close_month_audit() {
    let book = book!["식비 1000 물"];
    book.close_month(&report::Plain);
    assert_eq!(
        book.audit.events(),
        [
            "정산 시작",
            "잔액 시작",
            "합산 시작",
            "합산 끝",
            "잔액 끝",
            "보고서 완성",
            "정산 끝"
        ]
    );
}

#[test]
fn budgets() {
    assert!(budget!(30_000).allows(30_000));
    assert!(!budget!(30_000).allows(30_001));
    assert_eq!(budget!(_), ledger::Budget::Unlimited);
}

#[test]
fn unit_from_lookup() {
    assert_eq!(Config::from_lookup(|_| Some(String::from("엔"))).unit, "엔");
    assert_eq!(Config::from_lookup(|_| None).unit, "원");
}

#[test]
fn round10_by_c_symbol() {
    assert_eq!(ledger_round10(12_345), 12_350);
    assert_eq!(ledger_round10(-12), -10);
}

#[cfg(feature = "csv")]
#[test]
fn csv_export() {
    let book = book!["식비 1000 물"];
    let mut out = Vec::new();
    ledger::csv::write_csv(&book, &mut out).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "category,amount,memo\n식비,1000,물\n"
    );
}
