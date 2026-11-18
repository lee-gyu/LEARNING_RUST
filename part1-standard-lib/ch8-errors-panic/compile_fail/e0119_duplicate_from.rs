// 기대하는 에러: E0119 — conflicting implementations of trait `From<ParseIntError>` for type `SizeError`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0119_duplicate_from.rs --out-dir /tmp/rust-p1-ch8
//
// 가로와 세로가 모두 ParseIntError로 실패할 수 있어서, 배리언트를 둘로 나누고 From도 두 번 구현했습니다.
// 하지만 한 타입에 같은 트레이트(From<ParseIntError>)의 구현은 하나만 있을 수 있습니다.
// ?는 에러의 "타입"만 보고 From::from을 고르므로, 같은 타입의 에러가 어느 자리에서 왔는지 구분할 방법이 없습니다.
// 해결: From은 구현하지 않고, 실패할 수 있는 자리마다 map_err로 배리언트를 골라 감쌉니다.
//       w.parse().map_err(SizeError::BadWidth)?  /  h.parse().map_err(SizeError::BadHeight)?
//       (examples/03_from_conversion.rs의 parse_size_b)

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum SizeError {
    BadWidth(ParseIntError),
    BadHeight(ParseIntError),
}

impl fmt::Display for SizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadWidth(_) => write!(f, "가로 값이 숫자가 아닙니다"),
            Self::BadHeight(_) => write!(f, "세로 값이 숫자가 아닙니다"),
        }
    }
}

impl Error for SizeError {}

impl From<ParseIntError> for SizeError {
    fn from(e: ParseIntError) -> Self {
        Self::BadWidth(e)
    }
}

impl From<ParseIntError> for SizeError {
    fn from(e: ParseIntError) -> Self {
        Self::BadHeight(e)
    }
}

fn parse_size(text: &str) -> Result<(u32, u32), SizeError> {
    let (w, h) = text.split_once('x').unwrap_or((text, ""));
    Ok((w.parse()?, h.parse()?))
}

fn main() {
    println!("{:?}", parse_size("12x34"));
}
