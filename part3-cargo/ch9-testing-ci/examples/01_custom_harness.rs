//! `harness = false`인 테스트 타깃: libtest 없이 main이 직접 검사하고 종료 상태로 결과를 알립니다.
//!
//! 실행:
//!   cargo test -q -p p3-ch9-testing-ci --example 01_custom_harness
//!   cargo test -q -p p3-ch9-testing-ci --example 01_custom_harness -- 실패      (일부러 실패, 종료 상태 1)
//!   cargo run  -q -p p3-ch9-testing-ci --example 01_custom_harness            (같은 main을 예제로 실행)

use std::process::ExitCode;

/// 검사할 함수: 섭씨를 화씨로 바꿉니다.
fn c_to_f(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn main() -> ExitCode {
    // libtest가 없으므로 인자는 그대로 main에 들어옵니다. `--test-threads` 같은 옵션도 직접 해석해야 합니다.
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("받은 인자: {args:?}");

    let mut cases: Vec<(&str, f64, f64)> = vec![("어는점", 0.0, 32.0), ("끓는점", 100.0, 212.0)];
    if args.iter().any(|a| a == "실패") {
        cases.push(("틀린 기대값", 37.0, 100.0));
    }

    let mut failed = 0;
    for (name, c, expected) in &cases {
        let got = c_to_f(*c);
        if (got - expected).abs() < 1e-9 {
            println!("{name} ... ok");
        } else {
            println!("{name} ... 실패: {c}°C → {got}°F, 기대 {expected}°F");
            failed += 1;
        }
    }
    println!("{}개 중 {failed}개 실패", cases.len());
    // 종료 상태 0이면 Cargo는 통과로, 0이 아니면 실패로 봅니다.
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
    // 출력(cargo test --example 01_custom_harness):
    // 출력: 받은 인자: []
    // 출력: 어는점 ... ok
    // 출력: 끓는점 ... ok
    // 출력: 2개 중 0개 실패
    // 출력(-- 실패를 붙임, 종료 상태 1):
    // 출력: 받은 인자: ["실패"]
    // 출력: 어는점 ... ok
    // 출력: 끓는점 ... ok
    // 출력: 틀린 기대값 ... 실패: 37°C → 98.6°F, 기대 100°F
    // 출력: 3개 중 1개 실패
}
