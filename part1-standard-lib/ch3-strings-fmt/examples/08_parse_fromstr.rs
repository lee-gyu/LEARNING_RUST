// 이 예제가 보여 주는 것: parse::<T>()가 FromStr::from_str을 부른다는 것, 표준 타입의 파싱 규칙과 에러,
//                        직접 만든 타입에 FromStr을 구현해 Display와 왕복(문자열 → 값 → 문자열)되게 만들기
// 실행: cargo run -p p1-ch3-strings-fmt --example 08_parse_fromstr
//
// str::parse의 시그니처는 `fn parse<F: FromStr>(&self) -> Result<F, F::Err>`입니다.
// 즉 "FromStr을 구현한 타입이면 무엇이든" 만들 수 있고, 실패하면 그 타입이 정한 에러(F::Err)를 돌려줍니다.

use std::fmt;
use std::str::FromStr;

// 필드 순서대로 비교하는 PartialOrd/Ord를 derive하므로 major → minor → patch 순으로 크기를 비교합니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

// 사람이 읽고, 동시에 FromStr이 다시 읽을 수 있는 모양으로 출력합니다: 1.98.1
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// 실패 이유를 구분하는 에러 타입입니다. Debug는 derive하고, 사용자에게 보여 줄 Display만 직접 구현합니다.
// (std::error::Error 트레이트 구현은 ch8-errors-panic에서 다룹니다.)
#[derive(Debug, PartialEq)]
enum ParseVersionError {
    PartCount(usize),  // 점으로 나눈 부분이 3개가 아님
    BadNumber(String), // u32로 바꿀 수 없는 부분
}

impl fmt::Display for ParseVersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PartCount(n) => write!(f, "점으로 나눈 부분이 3개여야 하는데 {n}개입니다"),
            Self::BadNumber(part) => write!(f, "숫자가 아닌 부분이 있습니다: {part:?}"),
        }
    }
}

impl FromStr for Version {
    type Err = ParseVersionError; // parse가 돌려줄 에러 타입을 정합니다.

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 앞에 붙은 'v'는 있어도 되고 없어도 됩니다. 입력은 Display 출력보다 너그러워도 됩니다.
        let s = s.strip_prefix('v').unwrap_or(s);
        let parts: Vec<&str> = s.split('.').collect();
        // 슬라이스 패턴(Part 0 ch18)으로 "정확히 세 조각"인지 확인하면서 이름을 붙입니다.
        let [major, minor, patch] = parts[..] else {
            return Err(ParseVersionError::PartCount(parts.len()));
        };
        // 각 조각은 u32의 FromStr로 바꾸고, 표준 에러(ParseIntError)를 우리 에러로 바꿉니다(map_err, ch2-option-result).
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

// input을 T로 파싱한 결과를 한 줄로 보여 줍니다. T는 FromStr을 구현한 아무 타입이나 됩니다.
// 결과를 {:?}로 출력하려면 에러 타입(T::Err)도 Debug를 구현해야 하므로 where 절에 적었습니다.
fn show<T>(label: &str, input: &str)
where
    T: FromStr + fmt::Debug,
    T::Err: fmt::Debug,
{
    // 문자열의 디버그 형식은 너비를 무시하므로(06 예제), 따옴표를 붙인 String을 먼저 만들고 정렬합니다.
    let quoted = format!("{input:?}");
    println!("{label:<4} {quoted:<8} → {:?}", input.parse::<T>());
}

fn main() {
    // 1. 결과 타입을 알려 주는 세 가지 방법. 모두 같은 FromStr::from_str을 부릅니다.
    println!("== 1. parse와 FromStr ==");
    let a: i32 = "42".parse().unwrap(); // 변수의 타입 표기로
    let b = "42".parse::<i64>().unwrap(); // 터보피시로
    let c = i32::from_str("42").unwrap(); // FromStr::from_str을 직접(use std::str::FromStr 필요)
    println!("{a} {b} {c}"); // 출력: 42 42 42

    // 2. 표준 타입의 규칙: 공백을 걷어 내지 않고, 자릿수 구분 기호도 받지 않습니다.
    println!("\n== 2. 표준 타입의 파싱 규칙 ==");
    show::<i32>("i32", "+42"); // 출력: i32  "+42"    → Ok(42)
    show::<i32>("i32", " 42"); // 출력: i32  " 42"    → Err(ParseIntError { kind: InvalidDigit })
    show::<i32>("i32", "1_000"); // 출력: i32  "1_000"  → Err(ParseIntError { kind: InvalidDigit })
    show::<i32>("i32", ""); // 출력: i32  ""       → Err(ParseIntError { kind: Empty })
    show::<u8>("u8", "300"); // 출력: u8   "300"    → Err(ParseIntError { kind: PosOverflow })
    show::<f64>("f64", "1e3"); // 출력: f64  "1e3"    → Ok(1000.0)
    show::<f64>("f64", "3,14"); // 출력: f64  "3,14"   → Err(ParseFloatError { kind: Invalid })
    show::<bool>("bool", "True"); // 출력: bool "True"   → Err(ParseBoolError)
    show::<char>("char", "가"); // 출력: char "가"      → Ok('가')
    show::<char>("char", "가나"); // 출력: char "가나"     → Err(ParseCharError { kind: TooManyChars })
    // 표준 에러 타입은 Display도 구현하므로 사용자에게 보여 줄 메시지는 {}로 얻습니다.
    if let Err(e) = "300".parse::<u8>() {
        println!("에러 메시지: {e}"); // 출력: 에러 메시지: number too large to fit in target type
    }
    // 사용자 입력은 trim한 뒤에 파싱하는 것이 보통입니다.
    println!("trim 뒤 → {:?}", " 42\n".trim().parse::<i32>()); // 출력: trim 뒤 → Ok(42)

    // 3. 직접 구현한 FromStr: parse::<Version>()이 위의 from_str을 부릅니다.
    println!("\n== 3. 직접 만든 타입 파싱 ==");
    for input in ["1.98.1", "v2.0.10", "1.2", "1.x.3"] {
        let quoted = format!("{input:?}");
        match input.parse::<Version>() {
            Ok(v) => println!("{quoted:<9} → Ok: {v}"),
            Err(e) => println!("{quoted:<9} → Err: {e} ({e:?})"),
        }
    }
    // 출력:
    // "1.98.1"  → Ok: 1.98.1
    // "v2.0.10" → Ok: 2.0.10
    // "1.2"     → Err: 점으로 나눈 부분이 3개여야 하는데 2개입니다 (PartCount(2))
    // "1.x.3"   → Err: 숫자가 아닌 부분이 있습니다: "x" (BadNumber("x"))

    // 4. 왜 파싱하는가: 문자열 비교는 글자 순서라서 "1.10.0" < "1.9.0"이 됩니다.
    println!("\n== 4. 문자열 비교와 값 비교 ==");
    let (s1, s2) = ("1.10.0", "1.9.0");
    println!("문자열: {s1} > {s2} ? {}", s1 > s2); // 출력: 문자열: 1.10.0 > 1.9.0 ? false
    let (v1, v2): (Version, Version) = (s1.parse().unwrap(), s2.parse().unwrap());
    println!("Version: {v1} > {v2} ? {}", v1 > v2); // 출력: Version: 1.10.0 > 1.9.0 ? true

    // 5. 왕복: Display로 만든 문자열을 다시 parse하면 같은 값이 나와야 합니다.
    println!("\n== 5. Display ↔ FromStr 왕복 ==");
    let original = Version {
        major: 1,
        minor: 10,
        patch: 0,
    };
    let text = original.to_string();
    let back = text.parse::<Version>();
    println!("{original:?} → {text:?} → {back:?}"); // 출력: Version { major: 1, minor: 10, patch: 0 } → "1.10.0" → Ok(Version { major: 1, minor: 10, patch: 0 })
    println!("같은 값인가? {}", back == Ok(original)); // 출력: 같은 값인가? true

    // 6. 여러 개를 한꺼번에: Result로 collect하면 하나라도 실패할 때 첫 에러가 나옵니다(ch2-option-result).
    println!("\n== 6. 목록 파싱 ==");
    let good: Result<Vec<Version>, _> = "1.2.3, 1.10.0, 0.9.12"
        .split(", ")
        .map(str::parse)
        .collect();
    println!("{good:?}"); // 출력: Ok([Version { major: 1, minor: 2, patch: 3 }, Version { major: 1, minor: 10, patch: 0 }, Version { major: 0, minor: 9, patch: 12 }])
    if let Ok(list) = &good {
        println!(
            "가장 높은 버전: {:?}",
            list.iter().max().map(|v| v.to_string())
        ); // 출력: 가장 높은 버전: Some("1.10.0")
    }
    let bad: Result<Vec<Version>, _> = "1.2.3, 1.x.0, 2".split(", ").map(str::parse).collect();
    println!("{bad:?}"); // 출력: Err(BadNumber("x"))
}
