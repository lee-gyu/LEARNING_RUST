//! 연습용 레지스트리의 가상 크레이트 `tint` 2.0.0입니다.
//!
//! 1.x와 호환되지 않는 변경: `hex`가 `Option` 대신 `Result`를 돌려줍니다.

/// 이 사본의 버전입니다. 어느 버전이 골라졌는지 실행 결과로 보려고 둡니다.
pub const VERSION: &str = "2.0.0";

/// 모르는 색 이름을 받았을 때의 에러입니다.
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownColor(pub String);

/// 색 이름을 16진 코드로 바꿉니다.
pub fn hex(name: &str) -> Result<&'static str, UnknownColor> {
    match name {
        "red" => Ok("#ff0000"),
        "green" => Ok("#00ff00"),
        "blue" => Ok("#0000ff"),
        "yellow" => Ok("#ffff00"),
        _ => Err(UnknownColor(name.to_string())),
    }
}

/// 아는 색 이름의 목록입니다.
pub fn names() -> &'static [&'static str] {
    &["red", "green", "blue", "yellow"]
}
