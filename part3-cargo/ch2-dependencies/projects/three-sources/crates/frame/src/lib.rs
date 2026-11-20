//! 글 한 줄을 상자로 감싸는 작은 라이브러리입니다.
//! `three-sources`가 path 의존성으로 쓰는, 같은 저장소 안의 하위 크레이트입니다.

use unicode_width::UnicodeWidthStr;

/// 글 한 줄을 상자로 감쌉니다.
///
/// 한글처럼 터미널에서 두 칸을 차지하는 글자가 있어도 테두리가 맞도록
/// `char` 수가 아니라 화면 폭을 셉니다.
///
/// ```
/// assert_eq!(frame::boxed("hi"), "+----+\n| hi |\n+----+");
/// assert_eq!(frame::boxed("가"), "+----+\n| 가 |\n+----+");
/// ```
pub fn boxed(text: &str) -> String {
    let edge = format!("+{}+", "-".repeat(text.width() + 2));
    format!("{edge}\n| {text} |\n{edge}")
}
