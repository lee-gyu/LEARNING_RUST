//! 지침 1: 라이브러리는 패닉 대신 `Result`를 돌려준다.
//!
//! 체스 칸 이름("e4")을 읽는 같은 파서를 패닉판(`board_v1`)과 `Result`판(`board_v2`)으로
//! 만들고, 사용자가 손으로 적은 기보를 읽는 부르는 쪽 코드를 비교합니다.
//!
//! 실행: cargo run -p er-ch18-dont-panic --example 01_result_not_panic
//!
//! 1번 구역은 패닉판을 부르므로 일부러 두 번 패닉합니다. main이 `catch_unwind`로 붙잡아
//! 종료 코드 0으로 끝나지만, 기본 패닉 훅이 표준 에러에 패닉 메시지를 두 번 출력합니다.

use std::any::Any;
use std::error::Error;
use std::fmt;
use std::panic;

/// 체스판의 한 칸. `file`은 열(a~h → 0~7), `rank`는 행(1~8 → 0~7)입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    file: u8,
    rank: u8,
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", char::from(b'a' + self.file), self.rank + 1)
    }
}

/// 비권장: 패닉판만 있는 라이브러리. 잘못된 이름을 받으면 부르는 쪽 스레드를 끝냅니다.
mod board_v1 {
    use super::Square;

    /// 칸 이름을 읽습니다. 열은 대소문자를 가리지 않습니다.
    ///
    /// # Panics
    ///
    /// 이름이 "열 a~h + 행 1~8"의 두 글자가 아니면 패닉합니다.
    pub fn square(name: &str) -> Square {
        let mut chars = name.chars();
        let (Some(file), Some(rank), None) = (chars.next(), chars.next(), chars.next()) else {
            panic!("칸 이름은 두 글자여야 합니다: {name:?}");
        };
        let file = file.to_ascii_lowercase();
        if !('a'..='h').contains(&file) {
            panic!("열은 a~h여야 합니다: {name:?}");
        }
        if !('1'..='8').contains(&rank) {
            panic!("행은 1~8이어야 합니다: {name:?}");
        }
        // 두 글자 모두 ASCII임을 위에서 확인했으므로 as u8은 값을 잃지 않습니다.
        Square {
            file: file as u8 - b'a',
            rank: rank as u8 - b'1',
        }
    }
}

/// 권장: 같은 규칙을 `Result`로 알리는 라이브러리. 실패를 어떻게 다룰지는 부르는 쪽이 정합니다.
mod board_v2 {
    use super::Square;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SquareError {
        Length(usize),
        File(char),
        Rank(char),
    }

    impl fmt::Display for SquareError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Length(n) => write!(f, "칸 이름은 두 글자여야 합니다 (받은 글자 수 {n})"),
                Self::File(c) => write!(f, "열은 a~h여야 합니다: {c:?}"),
                Self::Rank(c) => write!(f, "행은 1~8이어야 합니다: {c:?}"),
            }
        }
    }

    impl Error for SquareError {}

    /// 칸 이름을 읽습니다. 열은 대소문자를 가리지 않습니다.
    ///
    /// # Errors
    ///
    /// 이름이 "열 a~h + 행 1~8"의 두 글자가 아니면 그 이유를 담은 `SquareError`를 돌려줍니다.
    pub fn parse_square(name: &str) -> Result<Square, SquareError> {
        let mut chars = name.chars();
        let (Some(file), Some(rank), None) = (chars.next(), chars.next(), chars.next()) else {
            return Err(SquareError::Length(name.chars().count()));
        };
        let file = file.to_ascii_lowercase();
        if !('a'..='h').contains(&file) {
            return Err(SquareError::File(file));
        }
        if !('1'..='8').contains(&rank) {
            return Err(SquareError::Rank(rank));
        }
        Ok(Square {
            file: file as u8 - b'a',
            rank: rank as u8 - b'1',
        })
    }
}

/// 사용자가 손으로 적은 기보. "G8"은 대문자지만 라이브러리가 받아 주는 이름이고,
/// "b9"는 행이 판 밖인 잘못된 이름입니다.
const RECORD: [&str; 6] = ["e2", "e4", "G8", "f6", "b9", "c3"];

/// 기보를 읽다 멈춘 위치와 이유. 부르는 쪽 애플리케이션의 에러 타입입니다.
#[derive(Debug)]
struct RecordError {
    line: usize,
    source: board_v2::SquareError,
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "기보 {}번째 칸을 읽지 못했습니다", self.line)
    }
}

impl Error for RecordError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// 패닉 페이로드에서 메시지를 꺼냅니다. 서식 인자가 있는 panic!은 String을 담습니다.
fn panic_message(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("(문자열이 아닌 페이로드)")
}

/// 부르는 쪽이 라이브러리의 규칙을 짐작해 다시 쓴 검사. 대문자 열을 모르고, 행 검사를 빠뜨렸습니다.
fn looks_like_square(name: &str) -> bool {
    name.len() == 2 && name.starts_with(|c: char| ('a'..='h').contains(&c))
}

/// `?`로 첫 실패를 위로 넘기는 부르는 쪽 함수. 몇 번째 칸인지는 부르는 쪽만 알므로 여기서 붙입니다.
fn load_record(record: &[&str]) -> Result<Vec<Square>, RecordError> {
    let mut squares = Vec::new();
    for (i, name) in record.iter().enumerate() {
        let square = board_v2::parse_square(name).map_err(|source| RecordError {
            line: i + 1,
            source,
        })?;
        squares.push(square);
    }
    Ok(squares)
}

fn join(squares: &[Square]) -> String {
    squares
        .iter()
        .map(Square::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    println!("1. 비권장: 패닉판만 있는 라이브러리");
    println!("  1-a. 그대로 부르면 잘못된 칸 하나가 기보 전체를 끝냄");
    let result = panic::catch_unwind(|| {
        RECORD
            .iter()
            .map(|name| board_v1::square(name))
            .collect::<Vec<_>>()
    });
    match result {
        Ok(squares) => println!("    읽은 칸: {}", join(&squares)),
        Err(payload) => println!(
            "    패닉, 앞에서 읽은 칸도 모두 잃음 (페이로드: {})",
            panic_message(&*payload)
        ),
    }
    // 출력: 패닉, 앞에서 읽은 칸도 모두 잃음 (페이로드: 행은 1~8이어야 합니다: "b9")

    println!("  1-b. 부르는 쪽이 규칙을 짐작해 미리 거름");
    let result = panic::catch_unwind(|| {
        for name in RECORD {
            if looks_like_square(name) {
                let square = board_v1::square(name);
                println!("    {name:?} 통과 → {square}");
            } else {
                println!("    {name:?} 부르는 쪽 검사에서 걸러 버림");
            }
        }
    });
    if let Err(payload) = result {
        println!("    패닉으로 멈춤: {}", panic_message(&*payload));
    }
    // 출력: "G8"은 라이브러리가 받는 이름인데 버려지고, "b9"는 검사를 통과해 패닉합니다.
    //       "c3"은 읽지 못합니다.

    println!("2. 권장: Result판을 받은 부르는 쪽이 정책을 고름");
    println!("  2-a. 잘못된 칸만 건너뛰고 이유를 알림");
    let mut squares = Vec::new();
    for name in RECORD {
        match board_v2::parse_square(name) {
            Ok(square) => squares.push(square),
            Err(e) => println!("    {name:?} 건너뜀: {e}"),
        }
    }
    println!("    읽은 칸: {}", join(&squares));
    // 출력: "b9" 건너뜀: 행은 1~8이어야 합니다: '9' / 읽은 칸: e2 e4 g8 f6 c3

    println!("  2-b. 첫 실패에서 멈추고 ?로 위에 넘김");
    match load_record(&RECORD) {
        Ok(squares) => println!("    읽은 칸: {}", join(&squares)),
        Err(e) => {
            let cause = e.source().map(ToString::to_string).unwrap_or_default();
            println!("    Err: {e}");
            println!("      원인: {cause}");
        }
    }
    // 출력: Err: 기보 5번째 칸을 읽지 못했습니다 / 원인: 행은 1~8이어야 합니다: '9'

    println!("  2-c. 코드에 적은 값처럼 실패할 리 없으면 부르는 쪽이 expect로 패닉을 고름");
    let king = board_v2::parse_square("e1").expect("코드에 적은 칸 이름 \"e1\"은 올발라야 합니다");
    println!("    백 킹의 시작 칸: {king}");
    // 출력: 백 킹의 시작 칸: e1
}
