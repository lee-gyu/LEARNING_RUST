//! 빌드 스크립트가 받은 환경 변수를 `cargo::rustc-env`로 넘겨받아 출력합니다.
//!
//! 실행:
//!   cargo run -q -p p3-ch8-build-scripts --example 02_build_info
//!   cargo run -q -p p3-ch8-build-scripts --example 02_build_info --release

fn main() {
    // build.rs가 `cargo::rustc-env=CH8_TARGET=…`처럼 출력한 값입니다. 컴파일할 때 박힙니다.
    println!("TARGET    (빌드 대상): {}", env!("CH8_TARGET"));
    println!("HOST      (빌드한 곳): {}", env!("CH8_HOST"));
    println!("PROFILE:   {}", env!("CH8_PROFILE"));
    println!("OPT_LEVEL: {}", env!("CH8_OPT_LEVEL"));
    println!("DEBUG:     {}", env!("CH8_DEBUG"));
    // 크레이트 쪽에서는 TARGET 같은 빌드 스크립트용 변수를 직접 읽을 수 없습니다.
    println!(
        "크레이트에서 option_env!(\"TARGET\"): {:?}",
        option_env!("TARGET")
    );

    // 출력(dev, macOS aarch64):
    // 출력: TARGET    (빌드 대상): aarch64-apple-darwin
    // 출력: HOST      (빌드한 곳): aarch64-apple-darwin
    // 출력: PROFILE:   debug
    // 출력: OPT_LEVEL: 0
    // 출력: DEBUG:     true
    // 출력: 크레이트에서 option_env!("TARGET"): None
    // 출력(--release): PROFILE·OPT_LEVEL·DEBUG 세 줄만 다릅니다.
    // 출력: PROFILE:   release
    // 출력: OPT_LEVEL: 3
    // 출력: DEBUG:     false
}
