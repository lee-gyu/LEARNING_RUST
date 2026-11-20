//! 책에 별점을 매기는 기능을 시험해 보는 크레이트입니다.

/// 별 다섯 개 만점의 별점을 문자열로 그립니다.
pub fn stars(score: u8) -> String {
    let filled = usize::from(score.min(5));
    format!("{}{}", "★".repeat(filled), "☆".repeat(5 - filled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_at_five() {
        assert_eq!(stars(3), "★★★☆☆");
        assert_eq!(stars(9), "★★★★★");
    }
}
