//! `required-features`로 기능이 켜졌을 때만 빌드되는 예제입니다.
//!
//! 이 챕터의 `Cargo.toml`에 다음이 있습니다.
//!
//!     [[example]]
//!     name = "02_required_features"
//!     required-features = ["stats"]
//!
//! 실행: cargo run -q -p p3-ch5-features --example 02_required_features --features stats
//!
//! 기능 없이 `cargo run`으로 실행하면 Cargo가 빌드하기 전에 멈추고(종료 코드 101)
//! 필요한 기능을 알려 줍니다. `cargo build --examples`는 이 예제를 말없이 건너뜁니다.

const SCORES: [u32; 5] = [72, 95, 88, 61, 84];

/// `stats` 기능이 꺼져 있으면 이 함수는 컴파일되지 않습니다.
#[cfg(feature = "stats")]
fn median(scores: &[u32]) -> u32 {
    let mut sorted = scores.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

fn main() {
    // 부르는 쪽은 `cfg`로 가리지 않았습니다. `required-features` 덕분에 이 파일 전체가
    // `stats`가 켜진 빌드에서만 컴파일되므로, 가린 함수를 그냥 불러도 됩니다.
    println!("가운데 점수: {}", median(&SCORES));
    let passed = SCORES.iter().filter(|&&s| s >= 70).count();
    println!("70점 이상: {passed}명 / {}명", SCORES.len());
    // 출력: 가운데 점수: 84
    // 출력: 70점 이상: 4명 / 5명
}
