// 2단계: 외부 크레이트 `rand`로 1~100 사이의 정답을 만듭니다.
//
// 보여 주는 것
// - `Cargo.toml`에 추가한 외부 크레이트를 `use`로 가져와 쓰는 방법
// - `rand::rng()`로 난수 생성기를 얻고 `random_range(1..=100)`으로 범위 안의 수를 뽑는 방법
// - 범위 표현식 `1..=100`(끝 포함)과 `1..100`(끝 제외)의 차이
//
// 실행 (대화형): cargo run -p p0-ch2-guessing-game --example 02_secret_number
// 실행 (파이프): echo 50 | cargo run -q -p p0-ch2-guessing-game --example 02_secret_number
//
// 정답은 실행할 때마다 달라지므로 아래 `출력:` 주석의 숫자 자리는 매번 다릅니다.

use std::io;

// `rand::prelude`에는 `rand`에서 자주 쓰는 항목이 모여 있습니다. 그중 `RngExt`라는
// 트레이트(trait)가 `random_range` 메서드를 제공합니다. 트레이트는 "이 타입이 할 수
// 있는 일의 목록"으로, 해당 트레이트가 스코프에 있어야 그 메서드를 부를 수 있습니다.
// 트레이트는 ch10-generics-traits-lifetimes에서 다룹니다.
use rand::prelude::*;

fn main() {
    println!("숫자를 맞혀 보세요!");

    // `rand::rng()`는 현재 스레드 전용 난수 생성기를 돌려줍니다. 처음 쓸 때 운영체제가
    // 주는 무작위 값으로 초기화되므로 실행할 때마다 다른 수열이 나옵니다.
    //
    // `1..=100`은 1과 100을 모두 포함하는 범위입니다. `1..100`이라고 쓰면 100은
    // 빠지므로 정답이 1~99 중에서만 나옵니다.
    let secret_number = rand::rng().random_range(1..=100);

    // 개발 중에 동작을 확인하려고 정답을 출력합니다. 완성판(04)에서는 지웁니다.
    println!("[개발용] 정답: {secret_number}");
    // 출력: [개발용] 정답: (1~100 중 하나, 실행마다 다름)

    println!("추측한 숫자를 입력하세요.");

    let mut guess = String::new();

    io::stdin()
        .read_line(&mut guess)
        .expect("입력을 읽지 못했습니다");

    // 아직은 문자열 그대로 출력만 합니다. 정답과 비교하는 일은 3단계에서 합니다.
    // `trim()`은 앞뒤 공백과 줄바꿈을 걷어 낸 문자열을 돌려주므로 빈 줄이 생기지 않습니다.
    println!("입력한 값: {}", guess.trim());
    // 출력: 입력한 값: 50
}
