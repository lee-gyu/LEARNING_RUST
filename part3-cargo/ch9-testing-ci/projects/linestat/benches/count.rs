//! `harness = false` 벤치마크: libtest 없이 main이 직접 시간을 잽니다.
//!
//! 실행: cargo bench          (bench 프로필 = release를 물려받음)
//! 시간 값은 실행할 때마다, 컴퓨터마다 달라집니다.

use std::hint::black_box;
use std::time::{Duration, Instant};

fn main() {
    // cargo bench는 벤치마크 실행 파일에 `--bench` 인자를 붙여 실행합니다.
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("받은 인자: {args:?}");

    let text = "가나다 abc def\n".repeat(20_000);
    let mut best = Duration::MAX;
    for _ in 0..20 {
        let t = Instant::now();
        let s = linestat::count(black_box(&text));
        black_box(s);
        best = best.min(t.elapsed());
    }
    let s = linestat::count(&text);
    println!(
        "count: 줄 {} / 단어 {} / 글자 {}",
        s.lines, s.words, s.chars
    );
    println!("20번 가운데 가장 짧은 시간: {best:?}");
}
