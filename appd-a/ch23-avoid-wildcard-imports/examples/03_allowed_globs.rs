// 지침을 따르지 않아도 되는 글롭 세 가지: 내 크레이트 안 모듈의 재내보내기, 프렐류드, 테스트의 use super::*
//
// 실행:   cargo run -p er-ch23-avoid-wildcard-imports --example 03_allowed_globs
// 테스트: cargo test -p er-ch23-avoid-wildcard-imports --example 03_allowed_globs
//
// 세 경우의 공통점은 "글롭이 가리키는 모듈에 무엇이 더해질지를 내가 정하거나,
// 글롭으로 가져가라고 골라 둔 목록이라는 것"입니다.
//
// clippy의 wildcard_imports 린트는 pedantic 그룹이라 기본으로 꺼져 있습니다.
// 이 파일에서만 켜 두었는데, 아래 글롭은 모두 린트가 정한 예외에 해당해 경고가 나지 않습니다.
#![warn(clippy::wildcard_imports)]

// ---- 1. 내 크레이트의 모듈을 나눠 쓰고 한곳에서 재내보내기 ----

mod units {
    mod celsius {
        pub fn to_kelvin(celsius: f64) -> f64 {
            celsius + 273.15
        }
    }

    mod fahrenheit {
        pub fn to_fahrenheit(celsius: f64) -> f64 {
            celsius * 1.8 + 32.0
        }
    }

    // 바깥에서는 하위 모듈 구조를 모른 채 units::to_kelvin처럼 씁니다.
    // 두 하위 모듈에 같은 이름이 생기면 그 이름을 쓰는 곳에서 내 빌드가 바로 멈추므로(E0659),
    // 문제가 남이 배포한 업데이트가 아니라 내가 고친 커밋에서 드러납니다.
    pub use self::celsius::*;
    pub use self::fahrenheit::*;
}

// ---- 2. 프렐류드: 글롭으로 가져가라고 골라 둔 목록 ----

/// 의존 크레이트 templog라고 생각합니다.
mod templog {
    #[derive(Debug, Default)]
    pub struct Log {
        readings: Vec<f64>,
    }

    impl Log {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn record(&mut self, celsius: f64) {
            self.readings.push(celsius);
        }

        pub fn readings(&self) -> &[f64] {
            &self.readings
        }
    }

    pub trait Stats {
        /// 최고 온도와 최저 온도의 차(일교차)
        fn spread(&self) -> f64;
    }

    impl Stats for Log {
        fn spread(&self) -> f64 {
            let max = self.readings.iter().copied().fold(f64::MIN, f64::max);
            let min = self.readings.iter().copied().fold(f64::MAX, f64::min);
            max - min
        }
    }

    pub fn mean(values: &[f64]) -> f64 {
        values.iter().sum::<f64>() / values.len() as f64
    }

    /// 거의 모든 사용자가 함께 쓰는 핵심 타입과 트레이트만 골라 둡니다. mean은 넣지 않았습니다.
    pub mod prelude {
        pub use super::{Log, Stats};
    }
}

use std::io::prelude::*; // BufRead, Read, Seek, Write 네 트레이트뿐입니다.
use templog::prelude::*; // Log, Stats

fn load(text: &str) -> Log {
    let mut log = Log::new();
    // &[u8]에 lines()를 부를 수 있는 것은 io 프렐류드가 BufRead를 들여왔기 때문입니다.
    for line in text.as_bytes().lines() {
        let line = line.expect("메모리 안의 바이트를 읽으므로 입출력 에러가 나지 않습니다");
        if let Ok(celsius) = line.trim().parse() {
            log.record(celsius);
        }
    }
    log
}

fn summary(log: &Log) -> String {
    // 프렐류드에 없는 mean은 경로로 부릅니다.
    format!(
        "기록 {}개, 평균 {:.2}도, 일교차 {:.1}도",
        log.readings().len(),
        templog::mean(log.readings()),
        log.spread()
    )
}

fn main() -> std::io::Result<()> {
    let log = load("18.0\n21.5\n24.0\n22.5\n");
    let mut out = std::io::stdout().lock();

    // writeln!이 out.write_fmt(..)를 부르려면 Write가 스코프에 있어야 합니다(io 프렐류드).
    writeln!(out, "{}", summary(&log))?;
    // 출력: 기록 4개, 평균 21.50도, 일교차 6.0도

    let c = 21.5;
    writeln!(
        out,
        "{c}도 = {:.2}K = {:.1}F",
        units::to_kelvin(c),
        units::to_fahrenheit(c)
    )?;
    // 출력: 21.5도 = 294.65K = 70.7F
    Ok(())
}

// ---- 3. 테스트 모듈의 use super::* ----

#[cfg(test)]
mod tests {
    // 부모 모듈의 항목을 비공개 함수(load, summary)까지 한꺼번에 가져옵니다.
    // 부모가 가져온 이름(Log, Stats, BufRead 등)도 함께 들어옵니다.
    use super::*;

    #[test]
    fn load_skips_bad_lines() {
        let log = load("20.0\n잘못된 줄\n22.0\n");
        assert_eq!(log.readings(), &[20.0, 22.0]);
    }

    #[test]
    fn summary_mentions_spread() {
        let text = summary(&load("18.0\n24.0\n"));
        assert!(text.contains("일교차 6.0도"), "{text}");
    }
}
