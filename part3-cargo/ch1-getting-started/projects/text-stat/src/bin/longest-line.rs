// `src/bin/` 바로 아래의 `.rs` 파일은 파일 이름이 곧 바이너리 이름입니다.
// 이 파일은 `longest-line`이라는 실행 파일이 됩니다.
//
// 실행: cargo run --bin longest-line

use text_stat::{load, longest_line, width};

fn main() -> std::io::Result<()> {
    let path = std::env::args().nth(1);
    let text = load(path.as_deref())?;
    match longest_line(&text) {
        Some(line) => println!("가장 긴 줄({}자): {line}", width(line)),
        None => println!("줄이 없습니다."),
    }
    Ok(())
}
