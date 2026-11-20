//! 기능(feature)이 코드에 닿는 두 가지 길, `#[cfg(feature = "…")]`와 `cfg!(feature = "…")`를 봅니다.
//!
//! 실행(기능 조합에 따라 출력이 달라집니다):
//!   cargo run -q -p p3-ch5-features --example 01_cfg_feature
//!   cargo run -q -p p3-ch5-features --example 01_cfg_feature --no-default-features
//!   cargo run -q -p p3-ch5-features --example 01_cfg_feature --features stats
//!   cargo run -q -p p3-ch5-features --example 01_cfg_feature --all-features
//!
//! Cargo는 켜진 기능마다 `rustc`에 `--cfg 'feature="이름"'`을 넘깁니다.
//! 그래서 기능은 컴파일할 때 이미 정해져 있고, 실행 중에 바꿀 수 없습니다.

const SCORES: [u32; 5] = [72, 95, 88, 61, 84];

/// `#[cfg]`: 조건이 거짓이면 이 함수는 컴파일 결과에서 통째로 사라집니다.
/// 없는 함수를 부르면 컴파일 에러가 나므로, 부르는 쪽도 같은 조건으로 가려야 합니다.
#[cfg(feature = "stats")]
fn average(scores: &[u32]) -> f64 {
    scores.iter().sum::<u32>() as f64 / scores.len() as f64
}

#[cfg(feature = "stats")]
fn best(scores: &[u32]) -> u32 {
    scores.iter().copied().max().unwrap_or(0)
}

/// 같은 이름의 함수를 두 판으로 두고, 기능에 따라 한쪽만 컴파일되게 할 수도 있습니다.
#[cfg(feature = "pretty")]
fn show(line: &str) {
    let edge = "-".repeat(line.chars().count() + 2);
    println!("+{edge}+");
    println!("| {line} |");
    println!("+{edge}+");
}

#[cfg(not(feature = "pretty"))]
fn show(line: &str) {
    println!("{line}");
}

fn main() {
    // `cfg!`: 컴파일할 때 정해지는 `true`/`false` 값입니다. `if`의 두 갈래는 모두 컴파일되므로,
    // 갈래 안에서 `#[cfg]`로 가린 함수를 부를 수는 없습니다.
    println!("pretty 켜짐: {}", cfg!(feature = "pretty"));
    println!("stats  켜짐: {}", cfg!(feature = "stats"));

    let joined: Vec<String> = SCORES.iter().map(|s| s.to_string()).collect();
    show(&format!("점수 {}", joined.join(" ")));

    // 함수를 부르는 블록도 같은 조건으로 가립니다.
    #[cfg(feature = "stats")]
    {
        println!("평균 {:.1}, 최고 {}", average(&SCORES), best(&SCORES));
    }
    #[cfg(not(feature = "stats"))]
    {
        println!("통계는 stats 기능을 켜야 계산합니다.");
    }
    // 출력(기본 기능, pretty만 켜짐):
    // 출력: pretty 켜짐: true
    // 출력: stats  켜짐: false
    // 출력: +-------------------+
    // 출력: | 점수 72 95 88 61 84 |
    // 출력: +-------------------+
    // 출력: 통계는 stats 기능을 켜야 계산합니다.
}
