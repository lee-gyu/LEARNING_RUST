//! JSON 문자열 리터럴을 만드는 작은 크레이트입니다.
//!
//! 기능:
//! - `unicode-escape`: ASCII가 아닌 글자를 `\uXXXX`로 바꿉니다. 꺼져 있으면 그대로 둡니다.

/// 문자열을 큰따옴표로 감싼 JSON 문자열 리터럴로 바꿉니다.
///
/// ```
/// assert_eq!(jsonesc::quote("a\"b"), r#""a\"b""#);
/// ```
pub fn quote(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            // 기능이 켜져 있을 때만 컴파일되는 갈래입니다.
            #[cfg(feature = "unicode-escape")]
            c if !c.is_ascii() => {
                // 기본 다국어 평면 밖의 글자는 UTF-16 대리 쌍 두 개로 나눠 씁니다.
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
