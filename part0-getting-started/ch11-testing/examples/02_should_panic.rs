// #[should_panic]으로 "이 코드는 패닉해야 한다"를 검사하고,
// expected로 패닉 메시지까지 확인하는 방법을 보여 줍니다.
//
// main 실행:   cargo run -p p0-ch11-testing --example 02_should_panic
// 테스트 실행: cargo test -p p0-ch11-testing --example 02_should_panic
// 실패 보기:   cargo test -p p0-ch11-testing --example 02_should_panic -- --ignored

// 1~100 사이의 값만 담을 수 있는 타입입니다.
// 범위를 벗어난 값으로 만들려고 하면 new가 패닉하므로, Guess 값이 있다면 그 값은 언제나 1~100입니다.
// "범위를 벗어나면 패닉한다"도 이 타입이 약속하는 동작이므로 테스트로 지켜야 합니다.
struct Guess {
    value: i32,
}

impl Guess {
    fn new(value: i32) -> Guess {
        if value < 1 {
            panic!("추측 값은 1 이상이어야 합니다. 받은 값: {value}");
        } else if value > 100 {
            panic!("추측 값은 100 이하여야 합니다. 받은 값: {value}");
        }

        Guess { value }
    }

    fn value(&self) -> i32 {
        self.value
    }
}

fn main() {
    let guess = Guess::new(50);
    println!("만든 값: {}", guess.value()); // 출력: 만든 값: 50

    // Guess::new(200)을 여기서 부르면 프로그램이 패닉으로 끝납니다.
    // 패닉하는 경우는 아래 테스트에서 #[should_panic]으로 확인합니다.
}

#[cfg(test)]
mod tests {
    use super::*;

    // 패닉하지 않는 경우도 함께 검사해야 "아무 값에나 패닉하는" 잘못된 구현을 잡을 수 있습니다.
    #[test]
    fn accepts_value_in_range() {
        assert_eq!(Guess::new(50).value(), 50);
    }

    // #[should_panic]: 함수 본문이 패닉하면 통과, 패닉 없이 끝나면 실패입니다.
    // 다만 "어떤 이유로든" 패닉하기만 하면 통과하므로, 엉뚱한 이유의 패닉도 통과시킵니다.
    #[test]
    #[should_panic]
    fn greater_than_100() {
        Guess::new(200);
    }

    // expected = "..."를 주면 패닉 메시지에 그 문자열이 들어 있어야 통과합니다.
    // 메시지 전체가 아니라 일부만 적어도 됩니다. 받은 값처럼 바뀌는 부분은 빼고 적습니다.
    #[test]
    #[should_panic(expected = "100 이하여야 합니다")]
    fn greater_than_100_with_message() {
        Guess::new(200);
    }

    #[test]
    #[should_panic(expected = "1 이상이어야 합니다")]
    fn less_than_1_with_message() {
        Guess::new(0);
    }

    // 아래 두 테스트는 should_panic 테스트가 실패하는 두 가지 경우를 보여 줍니다.
    mod failure_demos {
        use super::*;

        // 패닉이 일어나지 않으면 should_panic 테스트는 실패합니다.
        #[test]
        #[ignore = "실패 데모"]
        #[should_panic]
        fn does_not_panic() {
            Guess::new(50);
        }

        // 패닉은 했지만 메시지에 expected 문자열이 없으면 실패합니다.
        // 200은 "100 이하" 쪽 패닉을 일으키므로 "1 이상"이 들어 있지 않습니다.
        #[test]
        #[ignore = "실패 데모"]
        #[should_panic(expected = "1 이상이어야 합니다")]
        fn wrong_panic_message() {
            Guess::new(200);
        }
    }
}
