// 기대하는 에러: E0277 — the `?` operator can only be used on `Result`s, not `Option`s, in a function that returns `Result`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_option_question_in_result_fn.rs
//
// next()는 Option<&str>을 돌려줍니다. Option에 ?를 쓰면 None일 때 None을 돌려주며 함수를 끝내야 하는데,
// 이 함수의 반환 타입은 Result라서 None을 돌려줄 수 없습니다.
// ?는 Option과 Result를 저절로 바꿔 주지 않습니다.

use std::num::ParseIntError;

fn first_number(text: &str) -> Result<i32, ParseIntError> {
    let first_line = text.lines().next()?;
    let n: i32 = first_line.trim().parse()?;
    Ok(n)
}

fn main() {
    println!("{:?}", first_number("42\n7"));
}
