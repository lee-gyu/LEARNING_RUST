//! 빌드 스크립트: `csrc/tally.c`를 정적 라이브러리 `libtally.a`로 컴파일하고 링크를 맡깁니다.
//!
//! `#[link(name = "tally")]` 속성과 달리 빌드 스크립트는 코드로 환경을 살필 수 있습니다.
//! 여기서는 C 소스를 직접 컴파일하지만, 시스템에 설치된 라이브러리를 찾거나(pkg-config 등)
//! 운영체제마다 다른 이름을 고르는 일도 같은 자리에서 합니다.

use std::env;
use std::path::PathBuf;

fn main() {
    // Cargo가 넣어 주는 환경 변수로 이 패키지의 디렉터리를 찾습니다.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Cargo가 설정하는 변수입니다");
    let csrc = PathBuf::from(manifest_dir).join("csrc");

    // cc는 환경 변수(CC, CFLAGS 등)에 대한 rerun-if-env-changed만 출력합니다. 그런 지시문이
    // 하나라도 있으면 Cargo는 "패키지의 아무 파일이나 바뀌면 다시 실행"을 그만두므로,
    // C 소스와 헤더를 직접 알려 줘야 고친 C 코드가 다시 컴파일됩니다.
    println!("cargo::rerun-if-changed=csrc/tally.c");
    println!("cargo::rerun-if-changed=csrc/tally.h");

    // cc 크레이트가 이 플랫폼의 C 컴파일러로 tally.c를 컴파일해 OUT_DIR에 libtally.a를 만들고,
    // cargo:rustc-link-lib=static=tally와 cargo:rustc-link-search=native=<OUT_DIR>를 출력합니다.
    cc::Build::new()
        .file(csrc.join("tally.c"))
        .include(&csrc)
        .warnings_into_errors(true)
        .compile("tally");

    // Cargo.toml에 links = "tally"를 적었으므로, 이 패키지에 의존하는 패키지의 빌드 스크립트는
    // 이 값을 DEP_TALLY_INCLUDE 환경 변수로 받아 헤더를 찾을 수 있습니다.
    println!("cargo::metadata=include={}", csrc.display());
}
