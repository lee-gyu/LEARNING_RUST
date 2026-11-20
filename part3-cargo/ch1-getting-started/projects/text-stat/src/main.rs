// `src/main.rs`는 패키지 이름과 같은 이름의 바이너리(`text-stat`)가 됩니다.
//
// 실행: cargo run --bin text-stat            (예시 글)
//       cargo run --bin text-stat -- 파일경로

// 바이너리 크레이트는 같은 패키지의 라이브러리 크레이트를 외부 크레이트처럼 이름으로 씁니다.
// `crate::`는 이 바이너리 크레이트 자신을 가리키므로 `crate::count`라고 쓰면 찾지 못합니다.
use text_stat::{Counts, count, load};

fn main() -> std::io::Result<()> {
    let path = std::env::args().nth(1);
    let text = load(path.as_deref())?;
    let Counts {
        lines,
        words,
        chars,
        graphemes,
    } = count(&text);
    println!("줄 {lines} / 낱말 {words} / char {chars} / 자소 클러스터 {graphemes}");
    Ok(())
}
