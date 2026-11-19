// 기대하는 에러: E0659 — `mean` is ambiguous
//                (ambiguous because of multiple glob imports of a name in the same module)
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0659_two_glob_imports.rs --out-dir /tmp/er-ch23
//
// 상황: 사용자는 의존 크레이트 두 개(templog, chartkit)를 모두 글롭으로 가져옵니다.
//   - chartkit 1.0.0에는 bar만 있었고, 사용자의 mean 호출은 templog의 mean을 불렀습니다.
//   - chartkit 1.1.0이 공개 함수 mean을 더했습니다(마이너 변경).
//   - 두 글롭이 같은 이름을 들여오는 것만으로는 에러가 아닙니다. 그 이름을 쓰는 곳에서 E0659가 납니다.
//     bar처럼 한쪽에만 있는 이름은 계속 문제없이 쓸 수 있습니다.
// 해결: 쓰는 이름을 명시적으로 가져옵니다. 명시적 가져오기는 글롭보다 우선합니다.
//       use templog::mean;          (글롭을 남겨 둔다면 이 줄 하나로도 풀립니다)
//       use chartkit::bar;          (권장: 글롭을 모두 명시적 가져오기로 바꿉니다)

/// 의존 크레이트 templog 1.0.0이라고 생각합니다.
mod templog {
    pub fn mean(values: &[f64]) -> f64 {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

/// 의존 크레이트 chartkit 1.1.0이라고 생각합니다.
mod chartkit {
    pub fn bar(value: f64) -> String {
        "#".repeat(value as usize)
    }

    /// 1.1.0에서 추가: 막대 그래프의 기준선으로 쓸 평균
    pub fn mean(values: &[f64]) -> f64 {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

use chartkit::*;
use templog::*;

fn main() {
    let readings = [18.0, 21.5, 24.0, 22.5];
    println!("{}", bar(readings[0])); // 한쪽에만 있는 이름: 문제없음
    println!("평균 {:.2}도", mean(&readings)); // E0659
}
