// 지침: 예제 프로그램은 사용자가 그대로 옮겨 쓸 코드처럼 씁니다.
//       `main`이 `Result<(), Box<dyn Error>>`를 돌려주고 `unwrap()` 대신 `?`로 잇습니다.
//
// 실행(성공, 종료 코드 0):
//   cargo run -p er-ch30-beyond-unit-tests --example 03_example_program_style
// 실행(일부러 실패, ? 판, 종료 코드 1):
//   cargo run -p er-ch30-beyond-unit-tests --example 03_example_program_style -- 1h30m 2x
// 실행(일부러 실패, unwrap 판, 종료 코드 101):
//   cargo run -p er-ch30-beyond-unit-tests --example 03_example_program_style -- --unwrap 1h30m 2x
//
// `duration` 모듈은 라이브러리를 흉내 냅니다. 아래의 두 함수가 같은 일을 하는 "예제 프로그램"의
// 두 판이고, 첫 인자가 `--unwrap`이면 비권장 판을 실행합니다.

use std::env;
use std::error::Error;

mod duration {
    use std::fmt;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ParseError {
        Empty,
        UnknownUnit { unit: char },
        MissingNumber { unit: char },
        MissingUnit,
        TooLarge,
    }

    impl fmt::Display for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Empty => write!(f, "빈 문자열입니다"),
                Self::UnknownUnit { unit } => write!(f, "알 수 없는 단위 '{unit}'"),
                Self::MissingNumber { unit } => write!(f, "단위 '{unit}' 앞에 숫자가 없습니다"),
                Self::MissingUnit => write!(f, "마지막 숫자에 단위가 없습니다"),
                Self::TooLarge => write!(f, "값이 너무 큽니다"),
            }
        }
    }

    impl std::error::Error for ParseError {}

    /// `"1h30m"`, `"90s"`처럼 숫자와 단위(h, m, s)를 이어 쓴 시간을 초로 바꿉니다.
    pub fn parse_seconds(text: &str) -> Result<u64, ParseError> {
        if text.is_empty() {
            return Err(ParseError::Empty);
        }
        let mut total: u64 = 0;
        let mut number: Option<u64> = None;
        for ch in text.chars() {
            if let Some(d) = ch.to_digit(10) {
                let n = number.unwrap_or(0);
                number = Some(
                    n.checked_mul(10)
                        .and_then(|n| n.checked_add(u64::from(d)))
                        .ok_or(ParseError::TooLarge)?,
                );
                continue;
            }
            let scale = match ch {
                'h' => 3600,
                'm' => 60,
                's' => 1,
                _ => return Err(ParseError::UnknownUnit { unit: ch }),
            };
            let n = number
                .take()
                .ok_or(ParseError::MissingNumber { unit: ch })?;
            total = n
                .checked_mul(scale)
                .and_then(|secs| total.checked_add(secs))
                .ok_or(ParseError::TooLarge)?;
        }
        if number.is_some() {
            return Err(ParseError::MissingUnit);
        }
        Ok(total)
    }
}

const DEFAULT_INPUT: [&str; 3] = ["1h30m", "45m", "90s"];

/// 비권장 판: 실패하면 패닉합니다. 사용자가 이 코드를 자기 함수에 옮기면 패닉도 함께 옮겨 갑니다.
fn total_with_unwrap(args: &[String]) {
    let mut total = 0;
    for arg in args {
        let secs = duration::parse_seconds(arg).unwrap();
        println!("{arg} = {secs}초");
        total += secs;
    }
    println!("합계 {total}초");
}

/// 권장 판: 실패를 돌려줍니다. `Result`를 돌려주는 사용자 함수에 그대로 옮겨 쓸 수 있습니다.
fn total_with_question_mark(args: &[String]) -> Result<(), Box<dyn Error>> {
    let mut total: u64 = 0;
    for arg in args {
        let secs = duration::parse_seconds(arg)?;
        println!("{arg} = {secs}초");
        total = total.checked_add(secs).ok_or("합계가 너무 큽니다")?;
    }
    println!("합계 {total}초");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let use_unwrap = args.first().is_some_and(|a| a == "--unwrap");
    if use_unwrap {
        args.remove(0);
    }
    if args.is_empty() {
        args = DEFAULT_INPUT.iter().map(|s| s.to_string()).collect();
    }

    if use_unwrap {
        total_with_unwrap(&args);
        Ok(())
    } else {
        total_with_question_mark(&args)
    }
    // 출력(인자 없음, 두 판이 같음):
    // 1h30m = 5400초
    // 45m = 2700초
    // 90s = 90초
    // 합계 8190초
    //
    // 출력(`-- 1h30m 2x`, 표준 출력과 표준 에러, 종료 코드 1):
    // 1h30m = 5400초
    // Error: UnknownUnit { unit: 'x' }
    //
    // 출력(`-- --unwrap 1h30m 2x`, 종료 코드 101, 스레드 번호는 실행마다 다름):
    // 1h30m = 5400초
    // thread 'main' (…) panicked at appd-a/ch30-beyond-unit-tests/examples/03_example_program_style.rs:87:49:
    // called `Result::unwrap()` on an `Err` value: UnknownUnit { unit: 'x' }
    // note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
}

// 예제 프로그램에는 경계 사례 테스트를 넣지 않습니다. 이런 검사는 라이브러리의 단위 테스트가 맡습니다.
// 여기서는 `cargo test --examples`가 예제 안의 테스트도 실행한다는 것만 보입니다.
#[cfg(test)]
mod tests {
    use super::duration::{ParseError, parse_seconds};

    #[test]
    fn library_rejects_unknown_unit() {
        assert_eq!(
            parse_seconds("2x"),
            Err(ParseError::UnknownUnit { unit: 'x' })
        );
        assert_eq!(parse_seconds("1h30m"), Ok(5400));
    }
}
