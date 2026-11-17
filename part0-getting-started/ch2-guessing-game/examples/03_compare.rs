// 3단계: 입력을 숫자로 바꾸고 `match`로 정답과 비교합니다.
//
// 보여 주는 것
// - `trim()`과 `parse()`로 문자열을 숫자로 바꾸는 방법과, 바꿀 타입을 `: u32`로 알려 주는 이유
// - 같은 이름으로 새 변수를 선언해 이전 변수를 가리는 섀도잉(shadowing)
// - `cmp`가 돌려주는 `Ordering` 값을 `match`로 나눠 처리하는 방법
//
// 실행 (대화형): cargo run -p p0-ch2-guessing-game --example 03_compare
// 실행 (파이프): echo 50 | cargo run -q -p p0-ch2-guessing-game --example 03_compare
//
// 정답이 실행마다 달라지므로 마지막 줄이 "너무 작아요!", "너무 커요!", "정답입니다!" 중
// 무엇이 될지는 실행할 때마다 다릅니다. 숫자가 아닌 값을 넣으면 패닉으로 멈춥니다.

// `Ordering`은 두 값을 비교한 결과(작다, 크다, 같다)를 나타내는 열거형(enum)입니다.
use std::cmp::Ordering;
use std::io;

use rand::prelude::*;

fn main() {
    println!("숫자를 맞혀 보세요!");

    let secret_number = rand::rng().random_range(1..=100);
    println!("[개발용] 정답: {secret_number}");
    // 출력: [개발용] 정답: (1~100 중 하나, 실행마다 다름)

    println!("추측한 숫자를 입력하세요.");

    let mut guess = String::new();

    io::stdin()
        .read_line(&mut guess)
        .expect("입력을 읽지 못했습니다");

    // 이 시점의 `guess`는 "50\n" 같은 문자열(`String`)이고 `secret_number`는 정수입니다.
    // Rust는 타입이 다른 두 값을 비교하지 않으므로(compile_fail/e0308_compare_string_number.rs)
    // 먼저 문자열을 숫자로 바꿔야 합니다.
    //
    // - `trim()`: 앞뒤 공백과 줄바꿈("\n", Windows에서는 "\r\n")을 걷어 냅니다.
    //   줄바꿈이 남아 있으면 숫자가 아닌 문자가 섞인 것이므로 `parse`가 실패합니다.
    // - `parse()`: 문자열을 다른 타입으로 바꿉니다. 어떤 타입으로 바꿀지는
    //   `let guess: u32`의 타입 표기를 보고 정합니다. `u32`는 부호 없는 32비트 정수입니다.
    // - `parse()`도 실패할 수 있으므로 `Result`를 돌려주고, 여기서는 `expect`로
    //   실패하면 멈추게 했습니다. 실패를 견디는 방법은 4단계에서 봅니다.
    //
    // 섀도잉: `let guess`를 한 번 더 쓰면 같은 이름의 새 변수가 생기고, 이후로는
    // 새 변수(u32)가 보입니다. `guess_str`, `guess_num`처럼 이름을 둘 만들 필요가 없습니다.
    let guess: u32 = guess.trim().parse().expect("숫자를 입력해야 합니다!");

    println!("입력한 값: {guess}");
    // 출력: 입력한 값: 50

    // `guess`를 `u32`로 정했으므로 비교 상대인 `secret_number`도 `u32`로 추론됩니다.
    // (타입 표기가 없었다면 정수의 기본 타입인 `i32`가 됐을 것입니다.)
    //
    // `cmp`는 비교할 상대를 참조(`&`)로 받아 `Ordering`의 세 배리언트(variant)
    // `Less`, `Greater`, `Equal` 중 하나를 돌려줍니다.
    // `match`는 위에서부터 차례로 패턴과 맞춰 보고, 처음 맞는 갈래(arm)의 코드만 실행합니다.
    // 세 배리언트를 모두 적어야 컴파일됩니다. 하나라도 빠뜨리면 E0004 에러가 납니다.
    match guess.cmp(&secret_number) {
        Ordering::Less => println!("너무 작아요!"),
        Ordering::Greater => println!("너무 커요!"),
        Ordering::Equal => println!("정답입니다!"),
    }
    // 출력: 너무 작아요! / 너무 커요! / 정답입니다! 중 하나 (정답에 따라 다름)
}
