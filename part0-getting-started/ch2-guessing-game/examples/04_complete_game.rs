// 4단계: 맞힐 때까지 반복하고, 숫자가 아닌 입력은 건너뛰는 완성판입니다.
//
// 보여 주는 것
// - `loop`로 반복하고 `break`로 빠져나오는 방법, `continue`로 다음 반복으로 넘어가는 방법
// - `expect` 대신 `match`로 `parse`의 성공(`Ok`)과 실패(`Err`)를 나눠 처리하는 방법
// - 반복마다 값이 바뀌는 `let mut` 변수(시도 횟수)
// - 입력이 끝나면(EOF) `read_line`이 0을 돌려준다는 점을 이용해 게임을 끝내는 방법
//
// 실행 (대화형): cargo run -p p0-ch2-guessing-game --example 04_complete_game
// 실행 (파이프): seq 1 100 | cargo run -q -p p0-ch2-guessing-game --example 04_complete_game
//                printf '50\nabc\n' | cargo run -q -p p0-ch2-guessing-game --example 04_complete_game
//
// 정답이 실행마다 달라지므로 "너무 작아요!"/"너무 커요!"가 몇 번 나오는지, 몇 번 만에
// 맞히는지는 실행할 때마다 다릅니다.

use std::cmp::Ordering;
use std::io;

use rand::prelude::*;

fn main() {
    println!("숫자를 맞혀 보세요! (1~100)");

    // 이제 정답을 출력하지 않습니다. 정답을 알려 주면 게임이 되지 않으니까요.
    let secret_number = rand::rng().random_range(1..=100);

    // 반복할 때마다 1씩 늘려야 하므로 가변 변수로 선언합니다.
    let mut attempts = 0;

    // `loop`는 조건 없이 끝없이 반복합니다. 빠져나오려면 `break`를 써야 합니다.
    loop {
        println!("추측한 숫자를 입력하세요.");

        // `read_line`은 기존 내용 뒤에 입력을 덧붙입니다. 반복마다 새 빈 문자열을
        // 만들어야 이전에 입력한 값이 섞이지 않습니다.
        let mut guess = String::new();

        let bytes = io::stdin()
            .read_line(&mut guess)
            .expect("입력을 읽지 못했습니다");

        // 더 읽을 입력이 없으면(파이프 입력이 끝났거나 터미널에서 Ctrl+D를 누름)
        // `read_line`은 0바이트를 읽었다고 알려 줍니다. 이 검사가 없으면 빈 문자열을
        // 숫자로 바꾸지 못해 `continue`만 끝없이 반복합니다.
        // `if`는 ch3-common-concepts에서 자세히 다룹니다.
        if bytes == 0 {
            println!("입력이 끝나 게임을 종료합니다. (정답: {secret_number})");
            break;
        }

        // `parse()`가 돌려주는 `Result`에는 성공을 뜻하는 `Ok(숫자)`와 실패를 뜻하는
        // `Err(에러)` 두 배리언트가 있습니다. `match`로 둘을 나눠서
        // - `Ok(num)`이면 안의 숫자를 `num`이라는 이름으로 꺼내 `guess`에 넣고,
        // - `Err(_)`이면 다음 반복으로 넘어갑니다. `_`는 "값은 상관없다"는 뜻입니다.
        // `match`도 값을 만들어 내는 표현식이므로 결과를 `let`으로 받을 수 있습니다.
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("숫자가 아닙니다: {:?}", guess.trim());
                continue;
            }
        };

        // 숫자로 바뀐 입력만 시도 횟수에 셉니다.
        attempts += 1;
        println!("입력한 값: {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("너무 작아요!"),
            Ordering::Greater => println!("너무 커요!"),
            // 갈래에서 여러 줄을 실행하려면 중괄호로 묶습니다.
            Ordering::Equal => {
                println!("정답입니다! {attempts}번 만에 맞혔습니다.");
                break;
            }
        }
    }
    // `loop`가 `main`의 마지막이므로 `break`로 빠져나오면 프로그램도 끝납니다.
}
