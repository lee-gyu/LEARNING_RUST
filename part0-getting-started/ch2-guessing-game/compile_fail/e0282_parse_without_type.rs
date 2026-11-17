// 기대하는 에러: error[E0282]: type annotations needed
// 핵심 메시지:   type must be known at this point / help: consider giving `guess` an explicit type
//
// 확인 (챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0282_parse_without_type.rs
//
// `parse()`로 바꿀 타입을 알려 주지 않는 실수입니다.

use std::cmp::Ordering;
use std::io;

fn main() {
    let secret_number: u32 = 42;

    let mut guess = String::new();
    io::stdin()
        .read_line(&mut guess)
        .expect("입력을 읽지 못했습니다");

    // `parse()`는 문자열을 여러 타입(u32, i64, f64 등)으로 바꿀 수 있어서, 결과 타입을
    // 알 수 있는 단서가 있어야 합니다. 여기에는 단서가 없고, 바로 다음 줄에서
    // `guess.cmp(...)`처럼 메서드를 부르려면 그 전에 타입이 정해져 있어야 합니다.
    // 고치는 방법: `let guess: u32 = ...`처럼 타입을 적습니다.
    let guess = guess.trim().parse().expect("숫자를 입력해야 합니다!");

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("너무 작아요!"),
        Ordering::Greater => println!("너무 커요!"),
        Ordering::Equal => println!("정답입니다!"),
    }
}
