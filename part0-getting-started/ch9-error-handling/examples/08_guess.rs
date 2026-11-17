// 검증 타입: 값을 만들 때 한 번만 검사하고, 그 뒤로는 타입 자체가 "검사를 통과한 값"임을 보장합니다.
// The Book 9.3절의 Guess(1~100 사이의 추측값)를 따라 합니다.
//
// 같은 검사를 두 가지 방식으로 제공해, 패닉과 Result 중 무엇을 고를지 비교합니다.
//   Guess::new   — 1~100이 아니면 패닉합니다. 범위를 지키는 것은 호출하는 코드의 약속이고,
//                  어겼다면 호출한 쪽의 버그이기 때문입니다.
//   Guess::parse — 사용자가 입력한 문자열을 받아 Result로 돌려줍니다.
//                  잘못된 입력은 버그가 아니라 늘 일어나는 일이기 때문입니다.
//
// 실행: cargo run -p p0-ch9-error-handling --example 08_guess

use std::cmp::Ordering;

mod guessing {
    // 필드 value는 pub이 아니므로 비공개입니다(ch7). 그래서 이 모듈 밖에서는
    // Guess { value: 500 }처럼 직접 만들 수 없고, 반드시 new나 parse를 거쳐야 합니다.
    // 두 함수가 모두 범위를 검사하므로, 프로그램 어디에 있는 Guess든 값은 1~100입니다.
    #[derive(Debug)]
    pub struct Guess {
        value: i32,
    }

    // parse가 실패한 이유입니다. 배리언트마다 필요한 정보를 담습니다(ch6).
    #[derive(Debug)]
    pub enum GuessError {
        NotANumber(String),
        OutOfRange(i32),
    }

    impl Guess {
        pub fn new(value: i32) -> Guess {
            // (1..=100).contains(&value)는 "value가 1 이상 100 이하인가"입니다.
            // value < 1 || value > 100이라고 써도 같지만, Clippy는 범위로 쓰기를 권합니다.
            if !(1..=100).contains(&value) {
                panic!("Guess 값은 1~100이어야 합니다. 받은 값: {value}");
            }
            Guess { value }
        }

        pub fn parse(input: &str) -> Result<Guess, GuessError> {
            // i32로 읽으므로 "-5"도 숫자로 읽힌 뒤 범위 검사에서 걸립니다.
            // (ch2처럼 u32로 읽었다면 "-5"는 숫자가 아니라는 에러가 됩니다.)
            let value: i32 = match input.trim().parse() {
                Ok(n) => n,
                Err(_) => return Err(GuessError::NotANumber(input.to_string())),
            };
            if !(1..=100).contains(&value) {
                return Err(GuessError::OutOfRange(value));
            }
            // 범위를 방금 확인했으므로 여기서 new는 패닉하지 않습니다.
            Ok(Guess::new(value))
        }

        // 게터(ch5): 비공개 필드를 읽기 전용으로 내줍니다. 값을 바꾸는 메서드는 만들지 않았으므로
        // 한번 만들어진 Guess가 범위를 벗어날 방법이 없습니다.
        pub fn value(&self) -> i32 {
            self.value
        }
    }
}

use guessing::{Guess, GuessError};

// Guess를 받는 함수는 범위를 다시 검사하지 않습니다. 타입이 이미 보장하기 때문입니다.
fn compare(guess: &Guess, secret: i32) {
    let value = guess.value();
    match value.cmp(&secret) {
        Ordering::Less => println!("    {value}: 너무 작습니다"),
        Ordering::Greater => println!("    {value}: 너무 큽니다"),
        Ordering::Equal => println!("    {value}: 정답입니다!"),
    }
}

fn main() {
    let secret = 42;

    println!("1. 사용자 입력은 parse로 받습니다. 잘못된 입력은 Err로 알려 다시 묻게 합니다");
    for input in ["50", " 7 ", "abc", "150", "-5", "42"] {
        match Guess::parse(input) {
            Ok(guess) => compare(&guess, secret),
            Err(GuessError::NotANumber(text)) => println!("    {text:?}: 숫자가 아닙니다"),
            Err(GuessError::OutOfRange(n)) => println!("    {n}: 1~100 사이로 입력하세요"),
        }
    }
    // 출력:
    //     50: 너무 큽니다
    //     7: 너무 작습니다
    //     "abc": 숫자가 아닙니다
    //     150: 1~100 사이로 입력하세요
    //     -5: 1~100 사이로 입력하세요
    //     42: 정답입니다!

    println!("2. 코드에 직접 적은 값은 new로 만듭니다");
    let first_try = Guess::new(50);
    println!("    {first_try:?}");
    compare(&first_try, secret);
    // 출력:
    //     Guess { value: 50 }
    //     50: 너무 큽니다

    // 아래 주석을 풀면 약속(1~100)을 어긴 호출이므로 패닉하고 종료 코드 101로 끝납니다.
    //     Guess 값은 1~100이어야 합니다. 받은 값: 200
    // let _wrong = Guess::new(200);

    // 아래 주석을 풀면 검사를 건너뛰고 직접 만들려는 코드이므로 컴파일되지 않습니다.
    //     error[E0451]: field `value` of struct `Guess` is private
    // let _fake = Guess { value: 500 };
}
