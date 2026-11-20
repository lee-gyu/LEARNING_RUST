//! 글의 줄·낱말·글자 수를 세는 작은 라이브러리입니다.
//!
//! 이 파일(`src/lib.rs`)이 있으면 Cargo는 패키지에 라이브러리 타깃이 있다고 봅니다.
//! 패키지 이름은 `text-stat`이지만 크레이트 이름은 `-`를 `_`로 바꾼 `text_stat`입니다.
//! Rust 식별자에는 `-`를 쓸 수 없기 때문입니다.

use std::fs;
use std::io;

// `Cargo.toml`의 `[dependencies]`에 적은 크레이트입니다.
// 의존성 이름 `unicode-segmentation`도 코드에서는 `unicode_segmentation`으로 씁니다.
use unicode_segmentation::UnicodeSegmentation;

/// 인자 없이 실행했을 때 쓰는 예시 글입니다.
///
/// 마지막 줄은 조합형(NFD)으로 적은 "한글"입니다. 화면에는 두 글자로 보이지만
/// `char`는 자모 6개입니다(Part 0 ch8의 자소 클러스터).
pub const SAMPLE: &str = "\
Cargo는 Rust의 패키지 관리자입니다.
cargo build 는 빌드하고
cargo run 은 빌드한 뒤 바로 실행합니다.
\u{1112}\u{1161}\u{11ab}\u{1100}\u{1173}\u{11af}
";

/// 글 한 덩어리를 센 결과입니다.
#[derive(Debug, PartialEq, Eq)]
pub struct Counts {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
    pub graphemes: usize,
}

/// 줄, 낱말, `char`, 자소 클러스터의 수를 셉니다.
///
/// ```
/// let c = text_stat::count("hello world\n");
/// assert_eq!((c.lines, c.words, c.chars), (1, 2, 12));
/// ```
pub fn count(text: &str) -> Counts {
    Counts {
        lines: text.lines().count(),
        words: text.split_whitespace().count(),
        chars: text.chars().count(),
        // 표준 라이브러리에는 자소 클러스터를 나누는 기능이 없어 외부 크레이트를 씁니다.
        graphemes: text.graphemes(true).count(),
    }
}

/// 화면에 보이는 글자 수(자소 클러스터 수)를 셉니다.
pub fn width(line: &str) -> usize {
    line.graphemes(true).count()
}

/// 자소 클러스터 수가 가장 많은 줄을 돌려줍니다. 길이가 같으면 먼저 나온 줄입니다.
pub fn longest_line(text: &str) -> Option<&str> {
    let mut longest: Option<&str> = None;
    for line in text.lines() {
        // `>`로 비교하므로 길이가 같은 줄이 뒤에 나와도 바꾸지 않습니다.
        if longest.is_none_or(|best| width(line) > width(best)) {
            longest = Some(line);
        }
    }
    longest
}

/// 경로가 있으면 그 파일을, 없으면 [`SAMPLE`]을 읽습니다.
///
/// 세 바이너리(`text-stat`, `longest-line`, `top-words`)가 함께 쓰는 함수입니다.
/// 같은 패키지의 바이너리들은 라이브러리를 통해 코드를 나눠 씁니다.
pub fn load(path: Option<&str>) -> io::Result<String> {
    match path {
        Some(path) => fs::read_to_string(path),
        None => Ok(SAMPLE.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_has_no_lines() {
        assert_eq!(longest_line(""), None);
        assert_eq!(count("").lines, 0);
    }

    #[test]
    fn first_of_equal_lines_wins() {
        assert_eq!(longest_line("ab\ncd\ne"), Some("ab"));
    }
}
