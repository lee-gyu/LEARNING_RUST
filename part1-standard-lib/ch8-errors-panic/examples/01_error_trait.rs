// std::error::Error 트레이트의 생김새를 확인합니다.
//
// - Error는 Debug와 Display를 슈퍼트레이트로 가집니다. 두 트레이트를 먼저 구현해야 합니다.
// - Error의 메서드는 모두 기본 구현이 있으므로 `impl Error for T {}`처럼 빈 블록이면 충분합니다.
// - source()의 기본 구현은 None을 돌려줍니다("이 에러를 일으킨 더 낮은 수준의 에러가 없다").
// - Error를 구현하면 &dyn Error, Box<dyn Error> 같은 트레이트 객체로 다른 에러들과 함께 다룰 수 있습니다.
//
// 실행: cargo run -p p1-ch8-errors-panic --example 01_error_trait

use std::error::Error;
use std::fmt;
use std::io;
use std::str::FromStr;

// ── 1. 가장 작은 에러 타입 ──────────────────────────────────────────
// Debug는 개발자가 볼 내부 구조, Display는 사용자에게 보여 줄 문장입니다.
// Error 트레이트는 "에러라면 두 가지 모두로 자신을 설명할 수 있어야 한다"고 요구합니다.
#[derive(Debug)]
struct EmptyInput;

impl fmt::Display for EmptyInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 표준 라이브러리 문서의 관례: 짧게 쓰고 끝에 마침표를 붙이지 않습니다.
        // 이 메시지는 다른 메시지 안에 끼워 넣어지거나 "원인:" 뒤에 붙어 출력되기 때문입니다.
        write!(f, "입력이 비어 있습니다")
    }
}

// Debug와 Display가 이미 있으므로 빈 블록으로 Error를 구현합니다.
impl Error for EmptyInput {}

// ── 2. 어떤 에러든 받는 함수 ────────────────────────────────────────
// &dyn Error는 "Error를 구현한 어떤 타입의 값에 대한 참조"입니다(Part 0 ch17의 트레이트 객체).
// Error가 Debug + Display를 요구하므로, 구체 타입을 몰라도 두 형식으로 출력할 수 있습니다.
fn describe(label: &str, err: &dyn Error) {
    println!("    [{label}]");
    println!("      Display: {err}");
    println!("      Debug:   {err:?}");
    println!(
        "      source:  {}",
        if err.source().is_some() {
            "있음"
        } else {
            "없음"
        }
    );
}

// ── 3. ch3의 FromStr 에러 타입에 Error 구현하기 ─────────────────────
// Part 1 ch3의 08 예제에 있던 Version과 ParseVersionError를 그대로 가져왔습니다.
// ch3에서는 Display까지만 구현했습니다.
#[derive(Debug, PartialEq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug, PartialEq)]
enum ParseVersionError {
    PartCount(usize),
    BadNumber(String),
}

impl fmt::Display for ParseVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PartCount(n) => write!(f, "점으로 나눈 부분이 3개여야 하는데 {n}개입니다"),
            Self::BadNumber(part) => write!(f, "숫자가 아닌 부분이 있습니다: {part:?}"),
        }
    }
}

// ch3에서 미뤄 둔 한 줄입니다. 이 줄이 없으면 아래 parse_request의
// `version.parse::<Version>()?`가 컴파일되지 않습니다. ?가 ParseVersionError를
// Box<dyn Error>로 바꾸려면 ParseVersionError가 Error를 구현해야 하기 때문입니다.
impl Error for ParseVersionError {}

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let parts: Vec<&str> = s.split('.').collect();
        let [major, minor, patch] = parts[..] else {
            return Err(ParseVersionError::PartCount(parts.len()));
        };
        let num = |part: &str| {
            part.parse::<u32>()
                .map_err(|_| ParseVersionError::BadNumber(part.to_string()))
        };
        Ok(Version {
            major: num(major)?,
            minor: num(minor)?,
            patch: num(patch)?,
        })
    }
}

// "버전@개수" 모양의 요청을 읽습니다. 실패할 수 있는 곳이 세 군데이고 에러 타입이 모두 다릅니다.
//   split_once 실패 → &str 메시지
//   버전 파싱 실패   → ParseVersionError (직접 만든 에러)
//   개수 파싱 실패   → ParseIntError (표준 에러)
// 셋 다 Box<dyn Error>로 바뀌므로 ? 하나로 전파합니다.
fn parse_request(text: &str) -> Result<(Version, u32), Box<dyn Error>> {
    let (version, count) = text.split_once('@').ok_or("'@'가 없습니다")?;
    let version: Version = version.parse()?;
    let count: u32 = count.parse()?;
    Ok((version, count))
}

fn main() {
    println!("1. Error는 Debug + Display를 요구하고, 나머지는 기본 구현이 있습니다");
    let err = EmptyInput;
    println!("    {{}}   → {err}");
    println!("    {{:?}} → {err:?}");
    // 출력:
    //     {}   → 입력이 비어 있습니다
    //     {:?} → EmptyInput

    println!("2. 표준 에러도 직접 만든 에러도 &dyn Error로 똑같이 다룹니다");
    describe("직접 만든 에러", &EmptyInput);
    let parse_err = "12a".parse::<i32>().unwrap_err();
    describe("ParseIntError", &parse_err);
    let float_err = "3,14".parse::<f64>().unwrap_err();
    describe("ParseFloatError", &float_err);
    let io_err = io::Error::from(io::ErrorKind::NotFound);
    describe("io::Error", &io_err);
    // 출력:
    //     [직접 만든 에러]
    //       Display: 입력이 비어 있습니다
    //       Debug:   EmptyInput
    //       source:  없음
    //     [ParseIntError]
    //       Display: invalid digit found in string
    //       Debug:   ParseIntError { kind: InvalidDigit }
    //       source:  없음
    //     [ParseFloatError]
    //       Display: invalid float literal
    //       Debug:   ParseFloatError { kind: Invalid }
    //       source:  없음
    //     [io::Error]
    //       Display: entity not found
    //       Debug:   Kind(NotFound)
    //       source:  없음
    // 표준 라이브러리의 메시지는 모두 소문자로 시작하고 마침표가 없습니다.

    println!("3. Error를 구현한 ParseVersionError는 Box<dyn Error>로 ?를 쓸 수 있습니다");
    for text in [
        "1.2.3@4",
        "v2.0.1@10",
        "1.2.3",
        "1.2@4",
        "1.x.3@4",
        "1.2.3@-1",
    ] {
        match parse_request(text) {
            Ok((v, n)) => println!(
                "    {text:<12} → {}.{}.{} 버전 {n}개",
                v.major, v.minor, v.patch
            ),
            Err(e) => println!("    {text:<12} → 에러: {e}"),
        }
    }
    // 출력:
    //     1.2.3@4      → 1.2.3 버전 4개
    //     v2.0.1@10    → 2.0.1 버전 10개
    //     1.2.3        → 에러: '@'가 없습니다
    //     1.2@4        → 에러: 점으로 나눈 부분이 3개여야 하는데 2개입니다
    //     1.x.3@4      → 에러: 숫자가 아닌 부분이 있습니다: "x"
    //     1.2.3@-1     → 에러: invalid digit found in string
    // 마지막 줄은 개수 자리의 "-1"을 u32로 읽지 못한 ParseIntError의 메시지입니다.
}
