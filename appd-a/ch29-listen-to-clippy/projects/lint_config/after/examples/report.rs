//! `after` 라이브러리를 써서 독서 기록을 출력하는 예제 프로그램입니다.
//!
//! 실행(`projects/lint_config`에서): `cargo run -p after --example report`
//!
//! `[lints]`는 패키지 단위라서 이 예제에도 라이브러리와 같은 설정이 걸립니다. 화면에 출력하는
//! 것이 목적인 프로그램이므로 `print_stdout`만 크레이트 루트에서 예외로 둡니다.
#![expect(
    clippy::print_stdout,
    reason = "예제 프로그램은 표준 출력에 쓰는 것이 목적임"
)]

use after::{Shelf, ShelfError};

fn main() -> Result<(), ShelfError> {
    let mut shelf = Shelf::new();
    shelf.add("러스트 입문", 400)?;
    shelf.add("소유권 이야기", 240)?;
    shelf.record("러스트 입문", 100)?;
    shelf.record("소유권 이야기", 300)?;

    println!("{}", shelf.summary("이번 달 독서"));
    for title in ["러스트 입문", "소유권 이야기", "없는 책"] {
        match shelf.progress(title) {
            Some(p) => println!("{title}: {p}%"),
            None => println!("{title}: 기록 없음"),
        }
    }
    println!("남은 시간: {:.1}시간", shelf.hours_left());
    println!("내보낼 책 수: {}", shelf.export_count()?);
    Ok(())
}
