// 기대하는 에러: E0004 — non-exhaustive patterns: `Ticket::Infant` not covered
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0004_missing_match_arm.rs
//
// 04_catch_all.rs의 Ticket에 나중에 Infant(영유아) 배리언트를 추가했다고 가정한 코드입니다.
// 열거형 정의는 고쳤지만 가격을 정하는 match에 새 갈래를 추가하는 것을 잊었습니다.
// match는 모든 경우를 다뤄야 하므로, 컴파일러가 빠진 배리언트를 짚어 줍니다.

enum Ticket {
    Adult,
    Teen,
    Child,
    Senior,
    Infant, // 새로 추가한 배리언트
}

fn price(ticket: Ticket) -> u32 {
    match ticket {
        Ticket::Adult => 15000,
        Ticket::Teen => 12000,
        Ticket::Child => 8000,
        Ticket::Senior => 10000,
    }
}

fn main() {
    let tickets = [
        Ticket::Adult,
        Ticket::Teen,
        Ticket::Child,
        Ticket::Senior,
        Ticket::Infant,
    ];
    for ticket in tickets {
        println!("{}원", price(ticket));
    }
}
