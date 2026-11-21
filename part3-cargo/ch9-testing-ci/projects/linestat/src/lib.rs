//! 텍스트의 줄·단어·글자 수를 셉니다.
//!
//! ```
//! let s = linestat::count("한 줄\n두 번째 줄\n");
//! assert_eq!((s.lines, s.words, s.chars), (2, 5, 11));
//! ```

/// 텍스트 통계
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
}

/// 줄 수, 공백으로 나눈 단어 수, 글자(`char`) 수를 셉니다.
///
/// ```
/// use linestat::{count, Stats};
/// assert_eq!(count(""), Stats::default());
/// assert_eq!(count("a b\nc").words, 3);
/// ```
pub fn count(text: &str) -> Stats {
    Stats {
        lines: text.lines().count(),
        words: text.split_whitespace().count(),
        chars: text.chars().count(),
    }
}

/// 가장 많이 나온 단어 `n`개를 (단어, 횟수)로 돌려줍니다. 횟수가 같으면 단어 순서입니다.
///
/// ```
/// let top = linestat::top_words("b a b c b a", 2);
/// assert_eq!(top, vec![("b".to_string(), 3), ("a".to_string(), 2)]);
/// ```
#[cfg(feature = "top")]
pub fn top_words(text: &str, n: usize) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for w in text.split_whitespace() {
        *freq.entry(w).or_default() += 1;
    }
    let mut v: Vec<(String, usize)> = freq.into_iter().map(|(w, c)| (w.to_string(), c)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_newline_is_not_a_line() {
        assert_eq!(count("a\n").lines, 1);
        assert_eq!(count("a\n\n").lines, 2);
    }

    #[test]
    fn chars_are_not_bytes() {
        // "한글"은 2글자, 6바이트
        assert_eq!(count("한글").chars, 2);
    }

    #[cfg(feature = "top")]
    #[test]
    fn top_words_ties_are_sorted() {
        assert_eq!(
            top_words("y x", 2),
            vec![("x".to_string(), 1), ("y".to_string(), 1)]
        );
    }
}
