// 기대하는 에러: E0107 — type alias takes 1 generic argument but 2 generic arguments were supplied
//   (함께 나오는 E0308은 컴파일러가 Result<f64, String>을 templog의 Result<f64>로 읽은 결과입니다.)
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0107_glob_shadows_prelude.rs --out-dir /tmp/er-ch23
//
// 상황: 글롭으로 들어온 이름은 프렐류드의 이름을 가립니다(프렐류드가 가장 바깥 스코프이기 때문).
//   - templog 1.0.0에는 Log와 mean만 있었고, 사용자의 Result<f64, String>은 프렐류드의 Result였습니다.
//   - templog 1.1.0이 편의용 별칭 `pub type Result<T> = std::result::Result<T, Error>;`를 더했습니다.
//     많은 크레이트가 두는 모양이고, 항목 추가이므로 마이너 변경입니다.
//   - 글롭이 이 Result를 들여와 프렐류드의 Result를 가리므로, 타입 인자 두 개를 쓴 사용자 코드가 깨집니다.
// 주의: 컴파일러의 help("remove the unnecessary generic argument")를 따르면
//       Result<f64>가 templog의 별칭이 되어 에러 타입이 templog::Error로 바뀌므로 다른 에러가 납니다.
// 해결: 쓰는 이름만 명시적으로 가져옵니다(use templog::{Log, mean};).
//       글롭을 남겨야 한다면 std::result::Result<f64, String>처럼 경로를 적습니다.

/// 의존 크레이트 templog 1.1.0이라고 생각합니다.
mod templog {
    pub struct Log {
        pub readings: Vec<f64>,
    }

    pub fn mean(values: &[f64]) -> f64 {
        values.iter().sum::<f64>() / values.len() as f64
    }

    // ---- 1.1.0에서 추가한 항목 ----
    #[derive(Debug)]
    pub struct Error;

    pub type Result<T> = std::result::Result<T, Error>;
}

use templog::*;

// 아래 시그니처의 Result가 E0107입니다.
fn parse_reading(line: &str) -> Result<f64, String> {
    line.trim()
        .parse::<f64>()
        .map_err(|e| format!("{line:?}: {e}"))
}

fn main() {
    let mut log = Log {
        readings: Vec::new(),
    };
    for line in ["18.0", "21.5", "24.0"] {
        if let Ok(celsius) = parse_reading(line) {
            log.readings.push(celsius);
        }
    }
    println!("평균 {:.2}도", mean(&log.readings));
}
