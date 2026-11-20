//! `harness = false`인 통합 테스트입니다.
//!
//! 테스트 러너가 없으므로 `#[test]`를 붙여도 아무도 모아 주지 않습니다. 대신 이 `main`이
//! 그대로 실행되고, 종료 코드가 0이 아니면 `cargo test`가 실패로 봅니다.
//! 자기만의 보고 형식이 필요한 테스트 도구나 벤치마크 도구가 이 방식을 씁니다.

fn main() {
    let cases = [(-40.0, -40.0), (0.0, 32.0), (100.0, 212.0)];
    let mut failed = 0;
    for (celsius, expected) in cases {
        let got = units::c_to_f(celsius);
        let ok = (got - expected).abs() < 1e-9;
        println!(
            "{} {celsius}°C -> {got}°F",
            if ok { "통과" } else { "실패" }
        );
        if !ok {
            failed += 1;
        }
    }
    if failed > 0 {
        std::process::exit(1);
    }
}
