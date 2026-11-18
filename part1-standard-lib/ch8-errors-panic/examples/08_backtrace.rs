// std::backtrace::Backtrace로 "지금 이 지점까지 어떤 함수들을 거쳐 왔는가"를 값으로 담습니다.
//
//   1) Backtrace::capture()는 환경 변수가 허락할 때만 실제로 기록합니다.
//      RUST_LIB_BACKTRACE가 있으면 그 값을, 없으면 RUST_BACKTRACE를 봅니다. "0"이거나 둘 다 없으면 기록하지 않습니다.
//   2) Backtrace::force_capture()는 환경 변수와 상관없이 늘 기록합니다.
//   3) 에러가 만들어지는 순간(From::from 안)에 백트레이스를 담아 두면, 나중에 보고할 때
//      그 에러가 어디서 생겼는지 알 수 있습니다.
//
// 실행: 환경 변수에 따라 결과가 달라지므로 값을 직접 정해 두 번 실행해 봅니다.
//   RUST_LIB_BACKTRACE=0 cargo run -p p1-ch8-errors-panic --example 08_backtrace
//   RUST_LIB_BACKTRACE=1 cargo run -p p1-ch8-errors-panic --example 08_backtrace
//
// 백트레이스의 전체 내용(프레임 번호, 표준 라이브러리 경로 등)은 환경마다 다르므로 출력하지 않고,
// 상태(status)와 이 예제의 함수 이름이 들어 있는지만 확인합니다.

use std::backtrace::{Backtrace, BacktraceStatus};
use std::env;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// ── 에러 타입에 백트레이스 담기 ─────────────────────────────────────
#[derive(Debug)]
struct LimitError {
    source: ParseIntError,
    backtrace: Backtrace,
}

impl LimitError {
    // Error 트레이트에는 백트레이스를 꺼내는 안정 메서드가 없으므로(provide는 nightly 전용)
    // 에러 타입이 직접 게터를 제공합니다.
    fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for LimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "한도 값을 읽지 못했습니다")
    }
}

impl Error for LimitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

// ?가 ParseIntError를 LimitError로 바꾸는 순간, 즉 에러가 처음 이 타입이 되는 지점에서 기록합니다.
// capture()를 쓰므로 환경 변수가 꺼져 있으면 거의 비용 없이 "비어 있는 백트레이스"만 만듭니다.
impl From<ParseIntError> for LimitError {
    fn from(source: ParseIntError) -> Self {
        Self {
            source,
            backtrace: Backtrace::capture(),
        }
    }
}

fn parse_limit(text: &str) -> Result<u32, LimitError> {
    Ok(text.trim().parse()?)
}

fn load_settings(config: &str) -> Result<u32, LimitError> {
    let limit = parse_limit(config)?;
    Ok(limit)
}

// 백트레이스를 문자열로 바꿔(Display) 이 예제 크레이트(08_backtrace)의 함수 이름이 나오는 줄만 고릅니다.
// 줄 앞의 프레임 번호와 공백은 환경마다 다르므로 떼어 냅니다.
fn my_frames(bt: &Backtrace) -> Vec<String> {
    bt.to_string()
        .lines()
        .map(str::trim)
        .filter_map(|line| line.split_once(": "))
        .map(|(_, name)| name)
        .filter(|name| name.contains("08_backtrace::"))
        .map(str::to_string)
        .collect()
}

fn env_value(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| "(설정 안 됨)".to_string())
}

fn main() {
    println!("1. 환경 변수");
    println!(
        "    RUST_LIB_BACKTRACE = {}",
        env_value("RUST_LIB_BACKTRACE")
    );
    println!("    RUST_BACKTRACE     = {}", env_value("RUST_BACKTRACE"));

    println!("2. 세 가지 생성 함수의 상태");
    println!("    capture()       → {:?}", Backtrace::capture().status());
    println!(
        "    force_capture() → {:?}",
        Backtrace::force_capture().status()
    );
    println!("    disabled()      → {:?}", Backtrace::disabled().status());
    // 출력 (RUST_LIB_BACKTRACE=0):        출력 (RUST_LIB_BACKTRACE=1):
    //     capture()       → Disabled          capture()       → Captured
    //     force_capture() → Captured          force_capture() → Captured
    //     disabled()      → Disabled          disabled()      → Disabled

    println!("3. 에러에 담아 둔 백트레이스");
    let err = load_settings("12a").unwrap_err();
    println!("    에러: {err}");
    if let Some(source) = err.source() {
        println!("      원인: {source}");
    }
    // BacktraceStatus는 배리언트가 늘어날 수 있는 열거형(non_exhaustive)이라 _ 갈래가 필요합니다.
    match err.backtrace().status() {
        BacktraceStatus::Captured => {
            println!("    백트레이스가 기록되어 있습니다. 이 예제의 함수만 골라 보면:");
            for name in my_frames(err.backtrace()) {
                println!("      {name}");
            }
        }
        BacktraceStatus::Disabled => {
            println!("    백트레이스가 꺼져 있습니다 (RUST_LIB_BACKTRACE=1로 실행해 보세요)")
        }
        BacktraceStatus::Unsupported => println!("    이 플랫폼은 백트레이스를 지원하지 않습니다"),
        _ => println!("    알 수 없는 상태입니다"),
    }
    // 출력 (RUST_LIB_BACKTRACE=0):
    //     에러: 한도 값을 읽지 못했습니다
    //       원인: invalid digit found in string
    //     백트레이스가 꺼져 있습니다 (RUST_LIB_BACKTRACE=1로 실행해 보세요)
    // 출력 (RUST_LIB_BACKTRACE=1, macOS 디버그 빌드에서 확인):
    //     에러: 한도 값을 읽지 못했습니다
    //       원인: invalid digit found in string
    //     백트레이스가 기록되어 있습니다. 이 예제의 함수만 골라 보면:
    //       <08_backtrace::LimitError as core::convert::From<core::num::error::ParseIntError>>::from
    //       <core::result::Result<u32, 08_backtrace::LimitError> as core::ops::try_trait::FromResidual<core::result::Result<!, core::num::error::ParseIntError>>>::from_residual
    //       08_backtrace::parse_limit
    //       08_backtrace::load_settings
    //       08_backtrace::main
    // 위에서 아래로 "안쪽(가장 최근) → 바깥쪽" 순서입니다. parse_limit의 ?가 from_residual을 거쳐
    // From::from을 불렀고, 그 안에서 capture()가 실행됐다는 것을 읽을 수 있습니다.
    // RUST_LIB_BACKTRACE 없이 RUST_BACKTRACE=1로 실행해도 3번 구역은 같습니다(파일 맨 위 설명의 1번 참고).
}
