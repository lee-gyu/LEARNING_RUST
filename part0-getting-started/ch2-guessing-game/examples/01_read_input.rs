// 1단계: 표준 입력에서 한 줄을 읽어 그대로 출력합니다.
//
// 보여 주는 것
// - `let mut`로 내용을 바꿀 수 있는 변수를 만드는 이유
// - `String::new()`로 빈 문자열을 만들고 `read_line`으로 입력을 채우는 방법
// - `expect`가 성공 값(읽은 바이트 수)을 꺼내 주는 모습
// - 읽어 온 문자열 끝에 줄바꿈 문자가 함께 들어 있다는 사실
//
// 실행 (대화형): cargo run -p p0-ch2-guessing-game --example 01_read_input
// 실행 (파이프): echo 42 | cargo run -q -p p0-ch2-guessing-game --example 01_read_input

// `io`는 표준 라이브러리(std)의 입출력 모듈입니다. prelude(모든 프로그램에 자동으로
// 들어오는 항목 묶음)에 없으므로 `use`로 직접 가져옵니다.
use std::io;

fn main() {
    println!("숫자를 맞혀 보세요!");
    println!("추측한 숫자를 입력하세요.");

    // Rust의 변수는 기본적으로 불변(immutable)입니다. `read_line`이 이 문자열 뒤에
    // 입력을 덧붙여야 하므로 `mut`를 붙여 가변(mutable)으로 선언합니다.
    let mut guess = String::new();

    // `&mut guess`는 "guess를 잠시 빌려 줄 테니 고쳐 써도 된다"는 뜻의 가변 참조입니다.
    // 참조와 대여는 ch4-ownership에서 자세히 다룹니다.
    //
    // `read_line`은 성공했는지 실패했는지를 담은 `Result` 값을 돌려줍니다.
    // `expect`는 성공이면 안에 든 값(읽은 바이트 수)을 꺼내 주고, 실패면 주어진
    // 메시지와 함께 프로그램을 멈춥니다. `Result`는 ch9-error-handling에서 다룹니다.
    let bytes = io::stdin()
        .read_line(&mut guess)
        .expect("입력을 읽지 못했습니다");

    // 입력 끝의 줄바꿈 문자까지 그대로 출력되므로 이 줄 다음에 빈 줄이 하나 생깁니다.
    println!("입력한 값: {guess}");
    // 출력: 입력한 값: 42
    // 출력: (빈 줄)

    // `{guess:?}`는 디버그 형식입니다. 문자열을 따옴표로 감싸고 줄바꿈 같은 특수
    // 문자를 `\n`으로 보여 주므로, 눈에 보이지 않는 문자를 확인할 때 씁니다.
    println!("디버그 형식: {guess:?}");
    // 출력: 디버그 형식: "42\n"

    // "42"와 줄바꿈 문자를 합쳐 3바이트입니다.
    println!("읽은 바이트 수: {bytes}");
    // 출력: 읽은 바이트 수: 3
}
