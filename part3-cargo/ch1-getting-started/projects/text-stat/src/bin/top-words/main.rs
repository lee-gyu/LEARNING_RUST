// 바이너리 하나가 여러 파일로 이뤄지면 `src/bin/` 아래에 디렉터리를 만들고
// 그 안에 `main.rs`와 모듈 파일을 둡니다. 디렉터리 이름(`top-words`)이 바이너리 이름입니다.
//
// 실행: cargo run --bin top-words

// 같은 디렉터리의 `rank.rs`를 이 바이너리 크레이트의 모듈로 선언합니다.
mod rank;

use text_stat::load;

fn main() -> std::io::Result<()> {
    let path = std::env::args().nth(1);
    let text = load(path.as_deref())?;
    for (word, n) in rank::top(&text, 3) {
        println!("{n}번: {word}");
    }
    Ok(())
}
