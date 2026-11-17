// 테스트 함수가 Result<(), E>를 반환하게 해서 본문에서 ? 연산자를 쓰는 방법을 보여 줍니다.
// Ok(())를 반환하면 통과, Err를 반환하면 실패입니다.
//
// main 실행:   cargo run -p p0-ch11-testing --example 03_result_tests
// 테스트 실행: cargo test -p p0-ch11-testing --example 03_result_tests
// 실패 보기:   cargo test -p p0-ch11-testing --example 03_result_tests -- --ignored

use std::num::ParseIntError;

// 문자열을 정수로 바꾼 뒤 두 배로 만듭니다. 숫자가 아니면 parse가 돌려준 에러를 그대로 전달합니다.
fn parse_and_double(text: &str) -> Result<i32, ParseIntError> {
    let number: i32 = text.trim().parse()?;
    Ok(number * 2)
}

fn main() {
    println!("{:?}", parse_and_double("21")); // 출력: Ok(42)
    println!("{:?}", parse_and_double("스물하나")); // 출력: Err(ParseIntError { kind: InvalidDigit })
}

#[cfg(test)]
mod tests {
    use super::*;

    // 반환 타입이 Result<(), ParseIntError>이므로 본문에서 ParseIntError를 내는 ?를 쓸 수 있습니다.
    // ? 앞의 값이 Err이면 테스트 함수가 그 Err를 반환하며 끝나고, 테스트는 실패합니다.
    // unwrap()과 달리 실패 원인이 되는 에러 값이 그대로 테스트 결과에 전달됩니다.
    #[test]
    fn doubles_number() -> Result<(), ParseIntError> {
        let doubled = parse_and_double(" 21 ")?;
        assert_eq!(doubled, 42);
        Ok(())
    }

    // ?를 쓰지 않고 직접 Ok/Err를 만들어 반환해도 됩니다.
    // 에러 타입이 String이면 실패할 때 보여 줄 설명을 마음대로 적을 수 있습니다.
    #[test]
    fn doubles_negative_number() -> Result<(), String> {
        let result = parse_and_double("-4");
        if result == Ok(-8) {
            Ok(())
        } else {
            Err(format!(
                "-4의 두 배는 -8이어야 하는데 {result:?}가 나왔습니다"
            ))
        }
    }

    // "Err가 나와야 한다"를 검사할 때는 ?를 쓰지 않습니다. ?는 Err를 만나면 테스트를 실패로 끝내기 때문입니다.
    // 결과를 받아서 is_err()로 확인합니다.
    #[test]
    fn rejects_text() {
        let result = parse_and_double("스물하나");
        assert!(
            result.is_err(),
            "에러가 나와야 하는데 {result:?}가 나왔습니다"
        );
    }

    mod failure_demos {
        use super::*;

        // ?가 Err를 만나 테스트가 Err를 반환하는 경우입니다.
        // 출력에 Err 안의 값이 Debug 형식으로 나오지만, 어느 줄의 ?에서 멈췄는지는 나오지 않습니다.
        #[test]
        #[ignore = "실패 데모"]
        fn question_mark_returns_err() -> Result<(), ParseIntError> {
            let doubled = parse_and_double("스물하나")?;
            assert_eq!(doubled, 42);
            Ok(())
        }
    }
}
