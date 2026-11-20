// 기대하는 에러: 에러 코드 없음(린트 에러). 메시지로 확인합니다.
//   error: missing documentation for the crate
//   error: missing documentation for a function
//   error: missing documentation for a variant
//
// 확인: rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/deny_missing_docs.rs
//
// `missing_docs`는 rustc 린트라서 rustdoc 없이 `cargo build`에서도 검사됩니다(기본 수준은 allow).
// `deny`로 켜면 문서가 없는 공개 항목 하나가 빌드를 막습니다. 크레이트 자체(`//!`), 공개 함수,
// 공개 열거형의 배리언트마다 문서가 필요하고, 비공개 항목(`helper`)은 검사하지 않습니다.
//
// 이 에러를 없애려고 `/// 적립.`처럼 이름을 되풀이하는 주석을 다는 것이 이 린트의 흔한 부작용입니다.
// 린트는 문서가 "있는지"만 보고 무엇을 적었는지는 보지 않습니다(README 2.7절).

#![deny(missing_docs)]

pub enum Points {
    Earned(u32),
    Spent(u32),
}

pub fn earn(amount: u32) -> Points {
    Points::Earned(helper(amount))
}

fn helper(amount: u32) -> u32 {
    amount / 100
}
