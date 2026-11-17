// 기대하는 에러: E0308 — mismatched types (expected `bool`, found integer)
// 확인: rustc --edition 2024 compile_fail/e0308_if_condition_not_bool.rs
//
// `if`의 조건은 반드시 `bool`이어야 합니다. Rust는 정수를 `bool`로 자동 변환하지 않으므로
// C, JavaScript, Python처럼 "0이 아니면 참"으로 해석해 주지 않습니다.
// 고치는 방법: 원하는 조건을 비교식으로 직접 씁니다. 예: `if number != 0 { ... }`

fn main() {
    let number = 3;

    if number {
        println!("number는 0이 아닙니다");
    }
}
