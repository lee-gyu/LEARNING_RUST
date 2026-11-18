// source 체인을 따라가며 에러 보고서를 출력합니다.
//
//   1) 에러가 여러 층으로 감싸여 있으면 source()를 반복해서 불러 맨 아래 원인까지 내려갈 수 있습니다.
//   2) main이 Err를 돌려줄 때 찍히는 디버그 형식과, 직접 만든 보고서를 비교합니다.
//   3) Display에 원인 메시지까지 넣고 source()로도 내주면, 보고서에 같은 내용이 두 번 나옵니다.
//
// 실행: cargo run -p p1-ch8-errors-panic --example 05_error_report

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// ── 아래층: 설정 파싱 에러 ──────────────────────────────────────────
#[derive(Debug)]
enum ConfigError {
    Missing(&'static str),
    BadValue {
        line: usize,
        key: String,
        source: ParseIntError,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(key) => write!(f, "설정에 {key} 항목이 없습니다"),
            // 원인(ParseIntError)의 메시지는 넣지 않고, 이 층에서 알 수 있는 맥락(줄 번호, 키)만 씁니다.
            Self::BadValue { line, key, .. } => {
                write!(f, "{line}번째 줄의 {key} 값을 읽지 못했습니다")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Missing(_) => None,
            Self::BadValue { source, .. } => Some(source),
        }
    }
}

fn parse_port(text: &str) -> Result<u16, ConfigError> {
    for (i, line) in text.lines().enumerate() {
        if let Some((key, value)) = line.split_once('=')
            && key.trim() == "port"
        {
            return value
                .trim()
                .parse()
                .map_err(|source| ConfigError::BadValue {
                    line: i + 1,
                    key: key.trim().to_string(),
                    source,
                });
        }
    }
    Err(ConfigError::Missing("port"))
}

// ── 위층: 애플리케이션 에러 ─────────────────────────────────────────
#[derive(Debug)]
enum AppError {
    Startup(ConfigError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Startup(_) => write!(f, "서버를 시작하지 못했습니다"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Startup(e) => Some(e),
        }
    }
}

impl From<ConfigError> for AppError {
    fn from(e: ConfigError) -> Self {
        Self::Startup(e)
    }
}

fn start_server(config: &str) -> Result<u16, AppError> {
    let port = parse_port(config)?; // ConfigError → AppError::Startup
    Ok(port)
}

// ── 보고서: source 체인을 끝까지 따라갑니다 ─────────────────────────
// &dyn Error를 받으므로 직접 만든 에러, 표준 에러, Box<dyn Error>(as_ref로) 무엇이든 받습니다.
fn report(err: &dyn Error) {
    println!("    에러: {err}");
    let mut current = err.source();
    let mut depth = 1;
    while let Some(cause) = current {
        println!(
            "    {:indent$}원인 {depth}: {cause}",
            "",
            indent = depth * 2
        );
        current = cause.source();
        depth += 1;
    }
}

// ── 나쁜 예: Display와 source()가 같은 내용을 두 번 내보냄 ──────────
#[derive(Debug)]
struct NoisyPortError(ParseIntError);

impl fmt::Display for NoisyPortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 원인의 메시지를 Display에 붙였습니다.
        write!(f, "포트 값을 읽지 못했습니다: {}", self.0)
    }
}

impl Error for NoisyPortError {
    // 그런데 source()로도 같은 원인을 내줍니다.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

fn main() {
    let good = "host=localhost\nport=8080";
    let bad = "host=localhost\nport=80a0";
    let missing = "host=localhost";

    println!("1. 성공하면 포트를, 실패하면 보고서를 출력합니다");
    for (name, config) in [("good", good), ("bad", bad), ("missing", missing)] {
        println!("  [{name}]");
        match start_server(config) {
            Ok(port) => println!("    포트 {port}에서 시작합니다"),
            Err(e) => report(&e),
        }
    }
    // 출력:
    //   [good]
    //     포트 8080에서 시작합니다
    //   [bad]
    //     에러: 서버를 시작하지 못했습니다
    //       원인 1: 2번째 줄의 port 값을 읽지 못했습니다
    //         원인 2: invalid digit found in string
    //   [missing]
    //     에러: 서버를 시작하지 못했습니다
    //       원인 1: 설정에 port 항목이 없습니다

    println!("2. main이 이 에러를 Err로 돌려줬다면 이렇게 찍힙니다 (디버그 형식)");
    let err = start_server(bad).unwrap_err();
    println!("    Error: {err:?}");
    // 출력: Error: Startup(BadValue { line: 2, key: "port", source: ParseIntError { kind: InvalidDigit } })
    // 개발자에게는 쓸모 있지만 사용자에게 보여 줄 문장은 아닙니다. 그래서 보고서를 직접 출력합니다.

    println!("3. Box<dyn Error>에 담겨도 체인은 그대로 따라갈 수 있습니다");
    let boxed: Box<dyn Error> = Box::new(err);
    report(boxed.as_ref());
    // 출력:
    //     에러: 서버를 시작하지 못했습니다
    //       원인 1: 2번째 줄의 port 값을 읽지 못했습니다
    //         원인 2: invalid digit found in string
    // 체인의 맨 아래 원인을 구체 타입으로 꺼낼 수도 있습니다. source()가 'static 에러를 돌려주기 때문입니다.
    let mut root: &dyn Error = boxed.as_ref();
    while let Some(cause) = root.source() {
        root = cause;
    }
    if let Some(e) = root.downcast_ref::<ParseIntError>() {
        println!("    맨 아래 원인은 ParseIntError, 종류: {:?}", e.kind());
    }
    // 출력: 맨 아래 원인은 ParseIntError, 종류: InvalidDigit

    println!("4. Display와 source()에 같은 원인을 담으면 보고서에 두 번 나옵니다");
    let noisy = NoisyPortError("80a0".parse::<u16>().unwrap_err());
    report(&noisy);
    // 출력:
    //     에러: 포트 값을 읽지 못했습니다: invalid digit found in string
    //       원인 1: invalid digit found in string
}
