//! 책장을 출력하는 실행 파일 `shelf`입니다. 두 라이브러리 멤버를 이어 붙이기만 합니다.
//!
//! 실행: cargo run   (bookshelf 디렉터리에서)

use unicode_width::UnicodeWidthStr;

fn main() {
    let books = shelf_core::sample();
    // 버전은 이 패키지의 Cargo.toml이 아니라 워크스페이스 루트의 [workspace.package]에서 왔습니다.
    let title = format!("내 책장 (shelf {})", env!("CARGO_PKG_VERSION"));
    println!("{title}");
    println!("{}", "=".repeat(title.width()));
    println!("{}", shelf_report::table(&books));
    println!("{}", shelf_report::summary(&books));
}
