// 기대하는 에러: error[E0308]: mismatched types
// 핵심 메시지:   expected `&String`, found `&{integer}`
//
// 확인 (챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0308_compare_string_number.rs
//
// 입력받은 문자열을 숫자로 바꾸지 않고 그대로 정답과 비교하는 실수입니다.
// 실제 게임에서는 정답을 `rand`로 만들지만, `rustc`만으로 확인할 수 있도록 고정값을 씁니다.

use std::cmp::Ordering;
use std::io;

fn main() {
    let secret_number = 42;

    let mut guess = String::new();
    io::stdin()
        .read_line(&mut guess)
        .expect("입력을 읽지 못했습니다");

    // `guess`는 `String`이므로 `guess.cmp(...)`는 다른 `String`과만 비교할 수 있습니다.
    // 정수를 넘겼으니 타입이 맞지 않습니다.
    // 고치는 방법: 비교하기 전에 숫자로 바꿉니다.
    //     let guess: u32 = guess.trim().parse().expect("숫자를 입력해야 합니다!");
    match guess.cmp(&secret_number) {
        Ordering::Less => println!("너무 작아요!"),
        Ordering::Greater => println!("너무 커요!"),
        Ordering::Equal => println!("정답입니다!"),
    }
}
