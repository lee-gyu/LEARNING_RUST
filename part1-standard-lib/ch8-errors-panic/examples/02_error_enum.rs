// 실패 종류를 배리언트로 가진 에러 열거형을 설계합니다.
// Part 0 ch9 07 예제의 read_count는 io::Error와 ParseIntError를 Box<dyn Error> 하나로 돌려줬습니다.
// 이 예제는 같은 함수를 에러 열거형 판으로 다시 씁니다.
//
//   1) 실패 종류마다 배리언트를 하나씩 둡니다.
//   2) Display에는 "이 단계에서 무엇이 실패했는지"만 쓰고, 원인 에러는 필드로 품어 source()로 내줍니다.
//   3) From을 구현해 두면 함수 본문은 Box<dyn Error> 판과 똑같이 ?만 써도 됩니다.
//   4) 호출한 쪽은 match로 실패 종류마다 다르게 대응할 수 있습니다.
//
// 실행: cargo run -p p1-ch8-errors-panic --example 02_error_enum
//
// 파일은 임시 디렉터리(std::env::temp_dir())에 만들고 끝나기 전에 지웁니다.
// io::Error의 메시지는 macOS에서 확인한 것으로, 운영체제에 따라 다릅니다.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;

const MAX_COUNT: u32 = 100;

// ── 1. 실패 종류마다 배리언트 하나 ──────────────────────────────────
#[derive(Debug)]
enum ReadCountError {
    // 파일을 읽지 못함. 원인인 io::Error를 그대로 품습니다.
    Io(io::Error),
    // 내용이 숫자가 아님. 원인인 ParseIntError를 품습니다.
    Parse(ParseIntError),
    // 숫자는 맞지만 허용 범위를 넘음. 이 실패는 우리 코드가 판단한 것이라 원인 에러가 없고,
    // 대신 판단에 쓴 값을 필드로 담아 호출한 쪽이 읽을 수 있게 합니다.
    TooMany { count: u32, max: u32 },
}

// ── 2. Display: 이 단계의 실패만 설명 ───────────────────────────────
// 원인 에러의 메시지는 여기에 넣지 않습니다. 원인은 source()로 내주고,
// 원인까지 출력할지는 에러를 보고하는 쪽이 정합니다(05 예제).
impl fmt::Display for ReadCountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => write!(f, "개수 파일을 읽지 못했습니다"),
            Self::Parse(_) => write!(f, "개수 파일의 내용이 숫자가 아닙니다"),
            Self::TooMany { count, max } => write!(f, "개수가 최대값을 넘습니다 ({count} > {max})"),
        }
    }
}

// ── 3. Error: source()로 원인 에러를 내주기 ─────────────────────────
// 반환 타입 Option<&(dyn Error + 'static)>은 "원인 에러에 대한 참조, 없으면 None"입니다.
// 'static은 원인 에러 타입이 빌린 참조를 품고 있지 않다는 조건으로, 이 조건이 있어야
// 받은 쪽이 원인 에러를 구체 타입으로 다운캐스트할 수 있습니다(04 예제).
impl Error for ReadCountError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
            Self::TooMany { .. } => None,
        }
    }
}

// ── 4. From: ?가 원인 에러를 이 열거형으로 바꾸게 하기 ──────────────
// ?는 Err(e)를 만나면 From::from(e)로 함수의 에러 타입으로 바꿔 돌려줍니다(ch6-core-traits).
impl From<io::Error> for ReadCountError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ParseIntError> for ReadCountError {
    fn from(e: ParseIntError) -> Self {
        Self::Parse(e)
    }
}

// Part 0 ch9의 read_count와 본문이 거의 같습니다. 바뀐 것은 반환 타입과 범위 검사뿐입니다.
fn read_count(path: &Path) -> Result<u32, ReadCountError> {
    let text = fs::read_to_string(path)?; // io::Error → ReadCountError::Io
    let count: u32 = text.trim().parse()?; // ParseIntError → ReadCountError::Parse
    if count > MAX_COUNT {
        return Err(ReadCountError::TooMany {
            count,
            max: MAX_COUNT,
        });
    }
    Ok(count)
}

// 호출한 쪽: 실패 종류마다 다르게 대응합니다.
// Box<dyn Error>였다면 무엇이 들었는지 컴파일러가 알려 주지 않으므로 이렇게 나눌 수 없습니다.
// 열거형은 match의 완전성 검사 덕분에 배리언트를 하나 추가하면 여기서 컴파일 에러가 나서
// 처리를 빠뜨리지 않게 됩니다.
fn count_or_default(path: &Path) -> u32 {
    match read_count(path) {
        Ok(n) => n,
        // 파일이 없으면 처음 실행한 것으로 보고 0에서 시작합니다. 그 밖의 입출력 에러는 알립니다.
        Err(ReadCountError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {
            println!("    (파일이 없어 0에서 시작합니다)");
            0
        }
        // 최대값을 넘으면 배리언트의 필드에서 꺼낸 max를 대신 씁니다.
        Err(ReadCountError::TooMany { count, max }) => {
            println!("    (개수 {count}을 최대값 {max}에 맞춥니다)");
            max
        }
        // 나머지(숫자가 아님, 다른 입출력 에러)는 사용자에게 알리고 0을 씁니다.
        Err(e) => {
            println!(
                "    (에러: {e} / 원인: {})",
                e.source().map_or("없음".to_string(), |s| s.to_string())
            );
            0
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let dir = std::env::temp_dir();
    let good = dir.join("p1_ch8_02_good.txt");
    let bad = dir.join("p1_ch8_02_bad.txt");
    let big = dir.join("p1_ch8_02_big.txt");
    let missing = dir.join("p1_ch8_02_missing.txt");
    fs::write(&good, "12\n")?;
    fs::write(&bad, "열두 개\n")?;
    fs::write(&big, "500\n")?;

    println!("1. read_count는 실패 종류를 배리언트로 구분해 돌려줍니다 ({{:?}})");
    for (name, path) in [
        ("good", &good),
        ("bad", &bad),
        ("big", &big),
        ("missing", &missing),
    ] {
        println!("    {name:<7} → {:?}", read_count(path));
    }
    // 출력:
    //     good    → Ok(12)
    //     bad     → Err(Parse(ParseIntError { kind: InvalidDigit }))
    //     big     → Err(TooMany { count: 500, max: 100 })
    //     missing → Err(Io(Os { code: 2, kind: NotFound, message: "No such file or directory" }))

    println!("2. Display는 이 단계의 실패만, source()는 그 원인을 알려 줍니다");
    for (name, path) in [("bad", &bad), ("big", &big), ("missing", &missing)] {
        if let Err(e) = read_count(path) {
            println!("    {name:<7} → {e}");
            match e.source() {
                Some(source) => println!("              원인: {source}"),
                None => println!("              원인: 없음"),
            }
        }
    }
    // 출력:
    //     bad     → 개수 파일의 내용이 숫자가 아닙니다
    //               원인: invalid digit found in string
    //     big     → 개수가 최대값을 넘습니다 (500 > 100)
    //               원인: 없음
    //     missing → 개수 파일을 읽지 못했습니다
    //               원인: No such file or directory (os error 2)

    println!("3. 호출한 쪽은 match로 종류마다 다르게 대응합니다");
    for (name, path) in [
        ("good", &good),
        ("bad", &bad),
        ("big", &big),
        ("missing", &missing),
    ] {
        println!("  {name}");
        let n = count_or_default(path);
        println!("    → {n}");
    }
    // 출력:
    //   good
    //     → 12
    //   bad
    //     (에러: 개수 파일의 내용이 숫자가 아닙니다 / 원인: invalid digit found in string)
    //     → 0
    //   big
    //     (개수 500을 최대값 100에 맞춥니다)
    //     → 100
    //   missing
    //     (파일이 없어 0에서 시작합니다)
    //     → 0

    for path in [&good, &bad, &big] {
        fs::remove_file(path)?;
    }
    Ok(())
}
