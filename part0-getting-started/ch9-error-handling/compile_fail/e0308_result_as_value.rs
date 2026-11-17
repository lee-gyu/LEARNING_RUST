// 기대하는 에러: E0308 — mismatched types (expected `u32`, found `Result<_, _>`)
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0308_result_as_value.rs
//
// parse는 u32가 아니라 Result<u32, ParseIntError>를 돌려줍니다.
// 실패했을 때 어떻게 할지 정하지 않고 성공한 값처럼 쓸 수는 없습니다.

fn main() {
    let input = String::from("42");

    let guess: u32 = input.trim().parse();

    println!("{guess}");
}
