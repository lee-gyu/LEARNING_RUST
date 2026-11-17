// 기대하는 에러: E0277 — `?` couldn't convert the error to `std::io::Error`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_question_mark_error_conversion.rs
//
// 함수의 에러 타입은 io::Error인데, parse가 돌려주는 에러는 ParseIntError입니다.
// ?는 에러를 함수의 에러 타입으로 바꿔서 돌려주려고 하지만,
// ParseIntError를 io::Error로 바꾸는 방법은 정의되어 있지 않습니다.

use std::fs;
use std::io;

fn read_count(path: &str) -> Result<u32, io::Error> {
    let text = fs::read_to_string(path)?;
    let count: u32 = text.trim().parse()?;
    Ok(count)
}

fn main() {
    println!("{:?}", read_count("count.txt"));
}
