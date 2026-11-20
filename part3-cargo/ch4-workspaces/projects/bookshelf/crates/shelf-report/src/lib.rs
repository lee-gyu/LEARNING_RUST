//! 책장을 글로 보여 주는 크레이트입니다. `shelf-core`의 타입을 받아 표를 만듭니다.

use shelf_core::Book;
use unicode_width::UnicodeWidthStr;

/// 책 목록을 제목 열의 폭을 맞춘 표로 만듭니다.
pub fn table(books: &[Book]) -> String {
    // 한글은 터미널에서 두 칸을 차지하므로 `char` 수가 아니라 화면 폭으로 맞춥니다.
    let widest = books
        .iter()
        .map(|book| book.title.width())
        .max()
        .unwrap_or(0);
    let mut lines = Vec::new();
    for book in books {
        let padding = " ".repeat(widest - book.title.width());
        let mark = if book.finished {
            "읽음"
        } else {
            "읽는 중"
        };
        lines.push(format!(
            "{}{padding}  {:>4}쪽  {mark}",
            book.title, book.pages
        ));
    }
    lines.join("\n")
}

/// 한 줄 요약을 만듭니다.
pub fn summary(books: &[Book]) -> String {
    let finished = books.iter().filter(|book| book.finished).count();
    format!(
        "{}권 가운데 {finished}권을 읽었습니다({}쪽).",
        books.len(),
        shelf_core::pages_read(books)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts_finished() {
        let books = shelf_core::sample();
        assert_eq!(summary(&books), "3권 가운데 2권을 읽었습니다(980쪽).");
    }
}
