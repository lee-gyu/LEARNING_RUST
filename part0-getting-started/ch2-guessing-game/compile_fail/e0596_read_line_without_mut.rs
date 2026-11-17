// 기대하는 에러: error[E0596]: cannot borrow `guess` as mutable, as it is not declared as mutable
// 핵심 메시지:   cannot borrow as mutable / help: consider changing this to be mutable
//
// 확인 (챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0596_read_line_without_mut.rs
//
// `mut` 없이 선언한 변수를 `read_line`에 넘기는 실수입니다.

use std::io;

fn main() {
    // `mut`가 없으므로 이 변수는 불변입니다. 선언한 뒤로는 내용을 바꿀 수 없습니다.
    let guess = String::new();

    // `read_line`은 넘겨받은 문자열에 입력을 덧붙여야 하므로 가변 참조(`&mut`)를 요구합니다.
    // 불변 변수에서 가변 참조를 만들 수는 없으므로 컴파일러가 거부합니다.
    // 고치는 방법: `let mut guess = String::new();`
    io::stdin()
        .read_line(&mut guess)
        .expect("입력을 읽지 못했습니다");

    println!("입력한 값: {guess}");
}
