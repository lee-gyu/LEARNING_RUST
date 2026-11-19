//! 기능이 켜졌을 때만 빌드하는 예제. Cargo.toml의 `[[example]]`에 `required-features`를 적습니다.
//! 지침: 기능 전용 API를 쓰는 예제·테스트·바이너리는 `required-features`로 표시해, 어떤 기능
//! 조합으로 `cargo build --examples`를 실행해도 빌드가 깨지지 않게 합니다.
//!
//! 실행: cargo run -p er-ch26-feature-creep --example 04_required_features --features fast-hash
//!
//! 기능 없이 실행하면 Cargo가 이 예제를 빌드하지 않고 어떤 기능이 필요한지 알려 줍니다.
//! `cargo build --examples`처럼 여러 대상을 한꺼번에 빌드할 때는 이 예제를 조용히 건너뜁니다.
//!
//! 비권장: 이 파일 맨 위에 `#![cfg(feature = "fast-hash")]`를 붙여 파일 전체를 가리는 것.
//! 기능이 꺼진 빌드에서는 `main`까지 지워져 E0601(`main` function not found in crate ..)로
//! 실패합니다.

// 이 챕터 패키지에는 라이브러리가 없어서, 라이브러리의 기능 전용 API를 이 파일 안의 함수로
// 흉내 냅니다. 02의 `v2::fingerprint_fast`와 같은 함수입니다.
#[cfg(feature = "fast-hash")]
fn fingerprint_fast(s: &str) -> u32 {
    s.bytes()
        .fold(0, |h: u32, b| h.wrapping_mul(31).wrapping_add(u32::from(b)))
}

fn main() {
    // required-features 덕분에 이 파일은 fast-hash가 켜진 빌드에서만 컴파일됩니다.
    // 그래서 기능 전용 함수를 부르는 곳마다 #[cfg]를 붙이지 않아도 됩니다.
    println!("fast-hash 기능이 켜진 빌드에만 있는 예제입니다");
    for word in ["사과", "배", "포도"] {
        println!("  {word} → {:#010x}", fingerprint_fast(word));
    }
    // 출력:
    //   사과 → 0x9a3156bd
    //   배 → 0x0003882b
    //   포도 → 0x9c9d6016
}
