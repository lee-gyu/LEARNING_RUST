//! 연습용 레지스트리의 가상 크레이트 `tint` 1.1.0입니다. 1.0.0에 `yellow`를 더했습니다.

/// 이 사본의 버전입니다. 어느 버전이 골라졌는지 실행 결과로 보려고 둡니다.
pub const VERSION: &str = "1.1.0";

/// 색 이름을 16진 코드로 바꿉니다.
pub fn hex(name: &str) -> Option<&'static str> {
    match name {
        "red" => Some("#ff0000"),
        "green" => Some("#00ff00"),
        "blue" => Some("#0000ff"),
        // 일부러 넣은 버그입니다. 노랑은 #ffff00이어야 합니다(patched/ 실습에서 고칩니다).
        "yellow" => Some("#ff00ff"),
        _ => None,
    }
}
