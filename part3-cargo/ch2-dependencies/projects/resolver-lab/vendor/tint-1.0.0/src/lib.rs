//! 연습용 레지스트리의 가상 크레이트 `tint` 1.0.0입니다.

/// 이 사본의 버전입니다. 어느 버전이 골라졌는지 실행 결과로 보려고 둡니다.
pub const VERSION: &str = "1.0.0";

/// 색 이름을 16진 코드로 바꿉니다.
pub fn hex(name: &str) -> Option<&'static str> {
    match name {
        "red" => Some("#ff0000"),
        "green" => Some("#00ff00"),
        "blue" => Some("#0000ff"),
        _ => None,
    }
}
