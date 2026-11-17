// 여러 통합 테스트 파일이 함께 쓰는 도우미 모듈입니다.
// 테스트 파일에서 `mod common;`으로 선언하면, ch7에서 배운 모듈 파일 규칙(옛 방식 mod.rs)에 따라 이 파일을 찾습니다.
//
// tests/common.rs가 아니라 tests/common/mod.rs에 두는 이유:
// Cargo는 tests/ 바로 아래의 .rs 파일을 하나하나 별도의 통합 테스트 크레이트로 컴파일하고 실행합니다.
// tests/common.rs로 두면 테스트가 하나도 없는 common 구역이 cargo test 출력에 생깁니다.
// 하위 디렉터리 안의 파일은 테스트 크레이트로 취급하지 않으므로 그런 일이 없습니다.

/// 1부터 n까지의 수를 차례로 담은 벡터를 만듭니다.
pub fn one_to(n: u64) -> Vec<u64> {
    let mut numbers = Vec::new();
    for i in 1..=n {
        numbers.push(i);
    }
    numbers
}
