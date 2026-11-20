//! 책과 책장을 나타내는 타입입니다. 다른 두 멤버가 함께 쓰는 바탕 크레이트입니다.

/// 책 한 권입니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book {
    /// 제목
    pub title: String,
    /// 전체 쪽수
    pub pages: u32,
    /// 다 읽었는지
    pub finished: bool,
}

impl Book {
    /// `제목|쪽수|읽음 여부` 꼴의 한 줄을 책으로 바꿉니다.
    ///
    /// ```
    /// let book = shelf_core::Book::parse("러스트 입문|420|읽음").unwrap();
    /// assert_eq!(book.pages, 420);
    /// assert!(book.finished);
    /// ```
    pub fn parse(line: &str) -> Option<Book> {
        let mut parts = line.split('|');
        let title = parts.next()?.trim().to_string();
        let pages = parts.next()?.trim().parse().ok()?;
        let finished = parts.next()?.trim() == "읽음";
        Some(Book {
            title,
            pages,
            finished,
        })
    }
}

/// 예시 책장입니다.
pub fn sample() -> Vec<Book> {
    [
        "러스트 입문|420|읽음",
        "The Cargo Book|310|읽는 중",
        "시스템 프로그래밍|560|읽음",
    ]
    .iter()
    .filter_map(|line| Book::parse(line))
    .collect()
}

/// 다 읽은 책의 쪽수를 더합니다.
pub fn pages_read(books: &[Book]) -> u32 {
    books
        .iter()
        .filter(|book| book.finished)
        .map(|book| book.pages)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_broken_line() {
        assert_eq!(Book::parse("쪽수가 없는 줄"), None);
        assert_eq!(Book::parse("제목|많음|읽음"), None);
    }

    #[test]
    fn counts_only_finished_books() {
        assert_eq!(pages_read(&sample()), 980);
    }
}
