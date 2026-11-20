//! 독서 기록을 다루는 작은 라이브러리입니다. **린트 설정을 켜기 전에 쓴 판**입니다.
//!
//! 이 파일은 두 패키지가 함께 씁니다.
//! - `before-default`: `[lints]`가 없어 기본 설정의 Clippy로 봅니다. 경고가 없습니다.
//! - `before-strict`: 워크스페이스의 `[workspace.lints]`를 물려받습니다. 일부러 경고가 남습니다.
//!
//! 고친 판은 `after/src/lib.rs`입니다. 두 파일을 나란히 놓고 비교해 보세요.

use std::collections::*;

mod units {
    /// 한 시간에 읽는 쪽수(가정)입니다.
    pub const PAGES_PER_HOUR: u32 = 40;

    /// 쪽수를 읽는 데 걸리는 시간(시)입니다.
    pub fn hours(pages: u64) -> f64 {
        pages as f64 / PAGES_PER_HOUR as f64
    }
}

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

    /// 읽은 쪽수를 더하고, 지금까지 읽은 쪽수를 돌려줍니다.
    pub fn record(&mut self, title: &str, pages: u32) -> Result<u32, ShelfError> {
        let Some(book) = self.books.get_mut(title) else {
            return Err(ShelfError::NotFound(title.to_string()));
        };
        book.read = book.read.saturating_add(pages).min(book.pages);
        Ok(book.read)
    }

    /// 진행률(%)입니다.
    pub fn progress(&self, title: &str) -> u8 {
        let book = self.books.get(title).unwrap();
        (book.read as f64 / book.pages as f64 * 100.0) as u8
    }

    /// 책 수입니다. 1바이트 헤더로 내보낼 때 씁니다.
    pub fn export_count(&self) -> u8 {
        self.books.len() as u8
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
    pub fn summary(&self, heading: String) -> String {
        let mut out = heading.clone();
        for book in self.books.values() {
            out.push_str(&format!(
                "\n- {} ({}/{})",
                book.title, book.read, book.pages
            ));
        }
        out
    }
}

// 예전 형식("제목,쪽수")을 읽던 함수입니다. 지금은 아무도 부르지 않습니다.
#[allow(dead_code)]
fn legacy_parse(line: &str) -> Option<(String, u32)> {
    let (title, pages) = line.split_once(',')?;
    Some((title.to_string(), pages.trim().parse().ok()?))
}
