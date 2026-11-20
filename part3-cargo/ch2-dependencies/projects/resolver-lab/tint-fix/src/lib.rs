//! `tint` 1.2.0의 버그를 고친 로컬 사본입니다. 고친 버전이 게시되기 전까지 `[patch]`로 씁니다.

/// 이 사본의 버전입니다.
pub const VERSION: &str = "1.2.1";

/// 색 이름을 16진 코드로 바꿉니다.
pub fn hex(name: &str) -> Option<&'static str> {
    match name {
        "red" => Some("#ff0000"),
        "green" => Some("#00ff00"),
        "blue" => Some("#0000ff"),
        // 고친 곳: 1.1.0과 1.2.0은 #ff00ff를 돌려줬습니다.
        "yellow" => Some("#ffff00"),
        _ => None,
    }
}

/// 아는 색 이름의 목록입니다.
pub fn names() -> &'static [&'static str] {
    &["red", "green", "blue", "yellow"]
}
