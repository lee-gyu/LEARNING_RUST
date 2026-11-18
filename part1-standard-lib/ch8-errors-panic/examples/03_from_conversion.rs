// From 구현으로 ?가 에러를 변환하게 만드는 방법과, From만으로는 부족한 경우를 봅니다.
//
//   1) Part 1 ch2의 parse_age: map_err로 감싸던 코드가 From 구현 뒤에 ?만으로 줄어듭니다.
//   2) 같은 원인 타입(ParseIntError)이 두 군데서 올 때: From은 한 번만 구현할 수 있으므로
//      어느 쪽에서 실패했는지 남기려면 map_err로 맥락을 붙입니다.
//   3) 감싸기와 대신하기: 원인 에러를 품고 source()로 내줄지, 버리고 새 에러로 대신할지.
//
// 실행: cargo run -p p1-ch8-errors-panic --example 03_from_conversion

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// ── 1. map_err에서 From으로 ─────────────────────────────────────────
// ch2 02 예제의 AgeError입니다. ch2에서는 derive(Debug)만 붙였고 변환은 map_err로 했습니다.
mod ch2_style {
    use std::num::ParseIntError;

    #[derive(Debug)]
    pub enum AgeError {
        NotNumber(ParseIntError),
        OutOfRange(i32),
    }

    fn check_range(n: i32) -> Result<i32, AgeError> {
        if (0..=150).contains(&n) {
            Ok(n)
        } else {
            Err(AgeError::OutOfRange(n))
        }
    }

    // ParseIntError를 AgeError로 바꾸는 방법이 정의되어 있지 않으므로 직접 map_err로 감쌉니다.
    pub fn parse_age(input: &str) -> Result<i32, AgeError> {
        input
            .trim()
            .parse::<i32>()
            .map_err(AgeError::NotNumber)
            .and_then(check_range)
    }

    // AgeError에 Display가 없으므로, 메시지는 에러를 받은 쪽이 배리언트마다 만들어야 했습니다.
    pub fn describe(result: &Result<i32, AgeError>) -> String {
        match result {
            Ok(age) => format!("{age}살"),
            Err(AgeError::NotNumber(e)) => format!("숫자가 아님 ({e})"),
            Err(AgeError::OutOfRange(n)) => format!("범위 밖 ({n})"),
        }
    }
}

// 같은 열거형에 Display, Error, From을 갖췄습니다.
mod with_from {
    use std::error::Error;
    use std::fmt;
    use std::num::ParseIntError;

    #[derive(Debug)]
    pub enum AgeError {
        NotNumber(ParseIntError),
        OutOfRange(i32),
    }

    impl fmt::Display for AgeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::NotNumber(_) => write!(f, "나이가 숫자가 아닙니다"),
                Self::OutOfRange(n) => write!(f, "나이는 0~150이어야 합니다 (받은 값: {n})"),
            }
        }
    }

    impl Error for AgeError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::NotNumber(e) => Some(e),
                Self::OutOfRange(_) => None,
            }
        }
    }

    // "ParseIntError는 AgeError::NotNumber로 바뀐다"를 한 곳에 적어 둡니다.
    impl From<ParseIntError> for AgeError {
        fn from(e: ParseIntError) -> Self {
            Self::NotNumber(e)
        }
    }

    fn check_range(n: i32) -> Result<i32, AgeError> {
        if (0..=150).contains(&n) {
            Ok(n)
        } else {
            Err(AgeError::OutOfRange(n))
        }
    }

    // map_err가 사라졌습니다. parse()?의 결과 타입은 check_range의 매개변수(i32)에서 추론됩니다.
    pub fn parse_age(input: &str) -> Result<i32, AgeError> {
        check_range(input.trim().parse()?)
    }
}

// ── 2. 같은 원인 타입이 두 군데서 올 때 ─────────────────────────────
// "가로x세로" 문자열을 읽습니다. 가로와 세로 모두 parse에서 ParseIntError로 실패할 수 있습니다.

// 2-a. From만 쓴 판: From<ParseIntError>는 하나밖에 구현할 수 없으므로(compile_fail/e0119_duplicate_from.rs)
//      배리언트도 하나뿐이고, 가로와 세로 중 어디가 틀렸는지 잃어버립니다.
#[derive(Debug)]
enum SizeErrorA {
    MissingX,
    BadNumber(ParseIntError),
}

impl fmt::Display for SizeErrorA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingX => write!(f, "'x'가 없습니다"),
            Self::BadNumber(_) => write!(f, "숫자가 아닌 값이 있습니다"),
        }
    }
}

impl Error for SizeErrorA {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingX => None,
            Self::BadNumber(e) => Some(e),
        }
    }
}

impl From<ParseIntError> for SizeErrorA {
    fn from(e: ParseIntError) -> Self {
        Self::BadNumber(e)
    }
}

fn parse_size_a(text: &str) -> Result<(u32, u32), SizeErrorA> {
    let (w, h) = text.split_once('x').ok_or(SizeErrorA::MissingX)?;
    Ok((w.parse()?, h.parse()?))
}

// 2-b. map_err로 맥락을 붙인 판: 어느 칸인지(field)를 원인 에러와 함께 담습니다.
//      From을 구현하지 않으므로 ?만으로는 변환되지 않고, 실패할 수 있는 곳마다 map_err를 씁니다.
#[derive(Debug)]
enum SizeErrorB {
    MissingX,
    BadNumber {
        field: &'static str,
        source: ParseIntError,
    },
}

impl fmt::Display for SizeErrorB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingX => write!(f, "'x'가 없습니다"),
            Self::BadNumber { field, .. } => write!(f, "{field} 값이 숫자가 아닙니다"),
        }
    }
}

impl Error for SizeErrorB {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingX => None,
            Self::BadNumber { source, .. } => Some(source),
        }
    }
}

fn parse_size_b(text: &str) -> Result<(u32, u32), SizeErrorB> {
    let (w, h) = text.split_once('x').ok_or(SizeErrorB::MissingX)?;
    // 클로저 매개변수 이름을 필드 이름과 같은 source로 지어 필드 초기화 축약을 씁니다.
    let w = w.parse().map_err(|source| SizeErrorB::BadNumber {
        field: "가로",
        source,
    })?;
    let h = h.parse().map_err(|source| SizeErrorB::BadNumber {
        field: "세로",
        source,
    })?;
    Ok((w, h))
}

// ── 3. 감싸기와 대신하기 ────────────────────────────────────────────
// 3-a. 감싸기: 원인 에러를 필드로 품고 source()로 내줍니다. 정보가 남습니다.
#[derive(Debug)]
struct WrappedPortError(ParseIntError);

impl fmt::Display for WrappedPortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "포트 번호를 읽지 못했습니다")
    }
}

impl Error for WrappedPortError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<ParseIntError> for WrappedPortError {
    fn from(e: ParseIntError) -> Self {
        Self(e)
    }
}

// 3-b. 대신하기: from에서 원인 에러를 버리고 내 에러만 만듭니다.
//      ParseIntError가 공개 타입에 드러나지 않는 대신, 왜 실패했는지(숫자가 아님인지, 너무 큰지)를 잃습니다.
#[derive(Debug)]
struct OpaquePortError;

impl fmt::Display for OpaquePortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "포트 번호를 읽지 못했습니다")
    }
}

impl Error for OpaquePortError {}

impl From<ParseIntError> for OpaquePortError {
    fn from(_: ParseIntError) -> Self {
        Self
    }
}

fn port_wrapped(text: &str) -> Result<u16, WrappedPortError> {
    Ok(text.parse()?)
}

fn port_opaque(text: &str) -> Result<u16, OpaquePortError> {
    Ok(text.parse()?)
}

// 에러와 그 원인 한 단계를 한 줄로 보여 줍니다.
fn one_line(e: &dyn Error) -> String {
    match e.source() {
        Some(source) => format!("{e} (원인: {source})"),
        None => format!("{e} (원인 없음)"),
    }
}

fn main() {
    println!("1. map_err로 감싸던 코드가 From 구현 뒤에는 ?만으로 줄어듭니다");
    for input in ["42", "abc", "200"] {
        let before = ch2_style::describe(&ch2_style::parse_age(input));
        let after = match with_from::parse_age(input) {
            Ok(age) => format!("{age}살"),
            Err(e) => one_line(&e), // 메시지는 에러 타입의 Display가 만듭니다.
        };
        println!("    {input:>3}: ch2 판  {before}");
        println!("         From 판 {after}");
    }
    // 출력:
    //      42: ch2 판  42살
    //          From 판 42살
    //     abc: ch2 판  숫자가 아님 (invalid digit found in string)
    //          From 판 나이가 숫자가 아닙니다 (원인: invalid digit found in string)
    //     200: ch2 판  범위 밖 (200)
    //          From 판 나이는 0~150이어야 합니다 (받은 값: 200) (원인 없음)

    println!("2. 같은 원인 타입이 두 군데서 오면 From만으로는 어디서 실패했는지 잃습니다");
    for input in ["12x34", "12xab", "abx34", "1234"] {
        let a = parse_size_a(input).map_err(|e| one_line(&e));
        let b = parse_size_b(input).map_err(|e| one_line(&e));
        println!("    {input:>5}: From만 {a:?}");
        println!("           map_err {b:?}");
    }
    // 출력:
    //     12x34: From만 Ok((12, 34))
    //            map_err Ok((12, 34))
    //     12xab: From만 Err("숫자가 아닌 값이 있습니다 (원인: invalid digit found in string)")
    //            map_err Err("세로 값이 숫자가 아닙니다 (원인: invalid digit found in string)")
    //     abx34: From만 Err("숫자가 아닌 값이 있습니다 (원인: invalid digit found in string)")
    //            map_err Err("가로 값이 숫자가 아닙니다 (원인: invalid digit found in string)")
    //      1234: From만 Err("'x'가 없습니다 (원인 없음)")
    //            map_err Err("'x'가 없습니다 (원인 없음)")

    println!("3. 감싸면 원인이 남고, 대신하면 원인이 사라집니다");
    for input in ["8080", "http", "70000"] {
        let wrapped = port_wrapped(input).map_err(|e| one_line(&e));
        let opaque = port_opaque(input).map_err(|e| one_line(&e));
        println!("    {input:>5}: 감싸기   {wrapped:?}");
        println!("           대신하기 {opaque:?}");
    }
    // 출력:
    //      8080: 감싸기   Ok(8080)
    //            대신하기 Ok(8080)
    //      http: 감싸기   Err("포트 번호를 읽지 못했습니다 (원인: invalid digit found in string)")
    //            대신하기 Err("포트 번호를 읽지 못했습니다 (원인 없음)")
    //     70000: 감싸기   Err("포트 번호를 읽지 못했습니다 (원인: number too large to fit in target type)")
    //            대신하기 Err("포트 번호를 읽지 못했습니다 (원인 없음)")
}
