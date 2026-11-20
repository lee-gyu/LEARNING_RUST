//! 독서 기록을 다루는 작은 라이브러리입니다. **린트 설정에 맞춰 고친 판**입니다.
//!
//! `before/lib.rs`를 `before-strict`로 검사해 나온 경고 18개를 세 가지로 처리했습니다.
//! - 고침: 잘리는 캐스트, `unwrap`, 글롭 가져오기, 닿지 않는 `pub`, 이유 없는 `#[allow]`
//! - 작은 리팩터링: 진행률을 정수로 계산해 캐스트를 없앰, 값으로 받던 인자를 `&str`로
//! - 이유를 적고 예외로 둠: `u64`를 `f64`로 바꾸는 캐스트 하나(`units::hours`)

use std::collections::BTreeMap;
use std::fmt::Write as _;

mod units {
    /// 한 시간에 읽는 쪽수(가정)입니다.
    pub(crate) const PAGES_PER_HOUR: u32 = 40;

    /// 쪽수를 읽는 데 걸리는 시간(시)입니다.
    pub(crate) fn hours(pages: u64) -> f64 {
        // u64에서 f64로 가는 `From`은 없으므로 `as`가 유일한 방법입니다.
        #[expect(
            clippy::cast_precision_loss,
            reason = "쪽수 합은 2^53(약 9천조)보다 훨씬 작아 f64로 정확히 나타낼 수 있음"
        )]
        let pages = pages as f64;
        pages / f64::from(PAGES_PER_HOUR)
    }
}

// clippy.toml의 allowed-wildcard-imports에 `units`를 적어 두어 경고하지 않습니다.
use units::*;

/// 책 한 권의 기록입니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book {
    /// 제목
    pub title: String,
    /// 전체 쪽수
    pub pages: u32,
    /// 읽은 쪽수
    pub read: u32,
}

/// 책장 작업이 실패한 이유입니다.
#[derive(Debug, PartialEq, Eq)]
pub enum ShelfError {
    /// 같은 제목의 책이 이미 있음
    Duplicate(String),
    /// 그런 제목의 책이 없음
    NotFound(String),
    /// 1바이트 헤더에 담을 수 없을 만큼 책이 많음(실제 권수)
    TooMany(usize),
}

/// 책 목록입니다. 제목 순으로 정렬해 둡니다.
#[derive(Debug, Default)]
pub struct Shelf {
    books: BTreeMap<String, Book>,
}

impl Shelf {
    /// 빈 책장을 만듭니다.
    pub fn new() -> Self {
        Self::default()
    }

    /// 책을 더합니다.
    ///
    /// # Errors
    ///
    /// 같은 제목의 책이 이미 있으면 [`ShelfError::Duplicate`]를 돌려줍니다.
    pub fn add(&mut self, title: &str, pages: u32) -> Result<(), ShelfError> {
        if self.books.contains_key(title) {
            return Err(ShelfError::Duplicate(title.to_string()));
        }
        let book = Book {
            title: title.to_string(),
            pages,
            read: 0,
        };
        self.books.insert(title.to_string(), book);
        Ok(())
    }

    /// 읽은 쪽수를 더하고, 지금까지 읽은 쪽수를 돌려줍니다. 전체 쪽수를 넘지 않습니다.
    ///
    /// # Errors
    ///
    /// 그런 제목의 책이 없으면 [`ShelfError::NotFound`]를 돌려줍니다.
    pub fn record(&mut self, title: &str, pages: u32) -> Result<u32, ShelfError> {
        let Some(book) = self.books.get_mut(title) else {
            return Err(ShelfError::NotFound(title.to_string()));
        };
        book.read = book.read.saturating_add(pages).min(book.pages);
        Ok(book.read)
    }

    /// 진행률(%)입니다. 그런 책이 없거나 쪽수가 0이면 `None`입니다.
    pub fn progress(&self, title: &str) -> Option<u8> {
        let book = self.books.get(title)?;
        // 정수로 계산하면 캐스트가 필요 없습니다. read <= pages이므로 결과는 0..=100입니다.
        let percent = (u64::from(book.read) * 100).checked_div(u64::from(book.pages))?;
        u8::try_from(percent).ok()
    }

    /// 1바이트 헤더로 내보낼 책 수입니다.
    ///
    /// # Errors
    ///
    /// 책이 255권을 넘으면 [`ShelfError::TooMany`]에 실제 권수를 담아 돌려줍니다.
    pub fn export_count(&self) -> Result<u8, ShelfError> {
        u8::try_from(self.books.len()).map_err(|_| ShelfError::TooMany(self.books.len()))
    }

    /// 남은 쪽을 모두 읽는 데 걸리는 시간(시)입니다.
    pub fn hours_left(&self) -> f64 {
        let left: u64 = self
            .books
            .values()
            .map(|b| u64::from(b.pages - b.read))
            .sum();
        hours(left)
    }

    /// 머리말을 붙인 요약입니다.
    pub fn summary(&self, heading: &str) -> String {
        let mut out = heading.to_string();
        for book in self.books.values() {
            // String에 쓰는 write!는 실패하지 않습니다.
            let _ = write!(out, "\n- {} ({}/{})", book.title, book.read, book.pages);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Shelf {
        let mut shelf = Shelf::new();
        shelf.add("러스트 입문", 400).unwrap();
        shelf.add("빈 공책", 0).unwrap();
        shelf.record("러스트 입문", 100).unwrap();
        shelf
    }

    #[test]
    fn progress_is_integer_percent() {
        let shelf = sample();
        assert_eq!(shelf.progress("러스트 입문"), Some(25));
        assert_eq!(shelf.progress("빈 공책"), None);
        assert_eq!(shelf.progress("없는 책"), None);
    }

    #[test]
    fn export_count_refuses_to_truncate() {
        let mut shelf = Shelf::new();
        for i in 0..300 {
            shelf.add(&format!("책{i}"), 10).unwrap();
        }
        assert_eq!(shelf.export_count(), Err(ShelfError::TooMany(300)));
    }

    #[test]
    fn errors_are_reported() {
        let mut shelf = sample();
        assert_eq!(
            shelf.add("러스트 입문", 1),
            Err(ShelfError::Duplicate("러스트 입문".to_string()))
        );
        assert_eq!(
            shelf.record("없는 책", 1),
            Err(ShelfError::NotFound("없는 책".to_string()))
        );
    }
}
