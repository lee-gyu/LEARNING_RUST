//! 빌드 스크립트: `../csrc/v{N}/tally.c`를 정적 라이브러리 `libtally.a`로 컴파일하고 링크를 맡깁니다.
//!
//! ch34의 빌드 스크립트와 같고, C 소스의 판을 환경 변수 `TALLY_C_VERSION`(1, 2, 3, 기본 1)으로
//! 고르는 부분만 더했습니다. "시스템에 설치된 C 라이브러리가 새 판으로 올라간 상황"을 흉내 냅니다.
//! 손으로 쓴 선언(`src/ffi.rs`)은 v1 헤더를 보고 쓴 것이고, 이 스크립트는 그 사실을 모릅니다.

use std::env;
use std::path::PathBuf;

fn main() {
    let version = env::var("TALLY_C_VERSION").unwrap_or_else(|_| "1".to_string());
    if !matches!(version.as_str(), "1" | "2" | "3") {
        panic!("TALLY_C_VERSION은 1, 2, 3 중 하나여야 합니다(받은 값: {version:?})");
    }
    // 환경 변수가 바뀌면 다시 실행해 다른 판을 컴파일합니다.
    println!("cargo::rerun-if-env-changed=TALLY_C_VERSION");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Cargo가 설정하는 변수입니다");
    let csrc = PathBuf::from(manifest_dir)
        .join("../csrc")
        .join(format!("v{version}"));
    // 디렉터리를 주면 그 안의 파일이 하나라도 바뀌었을 때 다시 실행합니다.
    println!("cargo::rerun-if-changed=../csrc/v{version}");

    cc::Build::new()
        .file(csrc.join("tally.c"))
        .include(&csrc)
        .warnings_into_errors(true)
        .compile("tally");

    println!("cargo::metadata=include={}", csrc.display());
    // 데모 출력에 쓸 판 번호를 라이브러리에 넘깁니다(env!로 읽음).
    println!("cargo::rustc-env=TALLY_C_VERSION={version}");
}
