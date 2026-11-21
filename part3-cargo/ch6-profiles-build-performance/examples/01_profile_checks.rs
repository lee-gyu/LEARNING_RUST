//! 같은 소스가 고른 프로필에 따라 다르게 컴파일되는 것을 봅니다.
//! 실행 파일이 놓이는 디렉터리, `debug_assertions`, 정수 오버플로 검사가 프로필 설정을 따릅니다.
//!
//! 실행:
//!   cargo run -q -p p3-ch6-profiles-build-performance --example 01_profile_checks
//!   cargo run -q -p p3-ch6-profiles-build-performance --example 01_profile_checks --release

use std::hint::black_box;
use std::panic;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

static CHECKS: AtomicU32 = AtomicU32::new(0);

/// 비싼 검사라고 가정한 함수입니다. 불린 횟수를 셉니다.
fn is_sorted(v: &[i32]) -> bool {
    CHECKS.fetch_add(1, Ordering::Relaxed);
    v.windows(2).all(|w| w[0] <= w[1])
}

/// 경로의 마지막 세 구성 요소만 남깁니다. 절대 경로의 앞부분은 컴퓨터마다 다릅니다.
fn last_three(p: &Path) -> String {
    let parts: Vec<_> = p.iter().map(|c| c.to_string_lossy()).collect();
    parts[parts.len().saturating_sub(3)..].join("/")
}

fn main() {
    // 1. 프로필마다 출력 디렉터리가 다릅니다. dev는 역사적인 이유로 `debug`입니다.
    let exe = std::env::current_exe().expect("실행 파일 경로");
    println!("실행 파일: …/{}", last_three(&exe));

    // 2. `debug-assertions` 설정이 `cfg(debug_assertions)`를 켜고 끕니다.
    //    꺼지면 `debug_assert!`는 조건식을 평가하지 않습니다.
    println!("cfg!(debug_assertions): {}", cfg!(debug_assertions));
    let v = vec![1, 2, 3];
    debug_assert!(is_sorted(&v));
    println!(
        "debug_assert! 안의 검사 함수 호출: {}번",
        CHECKS.load(Ordering::Relaxed)
    );

    // 3. `overflow-checks` 설정이 `+`의 오버플로 동작을 정합니다.
    //    `black_box`는 컴파일러가 값을 미리 계산해 컴파일 에러로 만드는 것을 막습니다.
    panic::set_hook(Box::new(|_| {})); // 패닉 메시지는 아래에서 직접 출력합니다.
    let result = panic::catch_unwind(|| black_box(250u8) + black_box(10u8));
    match result {
        Ok(v) => println!("250u8 + 10: {v} (검사 없이 래핑)"),
        Err(_) => println!("250u8 + 10: 패닉 (오버플로 검사)"),
    }

    // 출력(dev 프로필, `cargo run`):
    // 출력: 실행 파일: …/debug/examples/01_profile_checks
    // 출력: cfg!(debug_assertions): true
    // 출력: debug_assert! 안의 검사 함수 호출: 1번
    // 출력: 250u8 + 10: 패닉 (오버플로 검사)
    // 출력(release 프로필, `cargo run --release`):
    // 출력: 실행 파일: …/release/examples/01_profile_checks
    // 출력: cfg!(debug_assertions): false
    // 출력: debug_assert! 안의 검사 함수 호출: 0번
    // 출력: 250u8 + 10: 4 (검사 없이 래핑)
}
