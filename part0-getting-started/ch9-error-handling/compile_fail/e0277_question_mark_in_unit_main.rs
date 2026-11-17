// 기대하는 에러: E0277 — the `?` operator can only be used in a function that returns `Result` or `Option`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_question_mark_in_unit_main.rs
//
// ?는 Err를 만나면 "그 Err를 돌려주며 함수를 끝내는" 연산자입니다.
// 그런데 이 main의 반환 타입은 ()라서 Err를 돌려줄 자리가 없습니다.

use std::fs::File;

fn main() {
    let greeting_file = File::open("hello.txt")?;

    println!("{greeting_file:?}");
}
