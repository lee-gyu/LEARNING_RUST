//! 빌드 스크립트: C 라이브러리를 컴파일해 링크하고, `bindgen` 기능이 켜져 있으면
//! 같은 판의 헤더에서 Rust 선언을 생성해 `OUT_DIR/bindings.rs`에 씁니다.
//!
//! C 소스의 판은 환경 변수 `TALLY_C_VERSION`(1, 2, 3, 기본 1)으로 고릅니다.
//! "C 라이브러리가 새 판으로 올라간 상황"을 흉내 내기 위한 장치입니다.

use std::env;
use std::path::PathBuf;

fn main() {
    let version = env::var("TALLY_C_VERSION").unwrap_or_else(|_| "1".to_string());
    if !matches!(version.as_str(), "1" | "2" | "3") {
        panic!("TALLY_C_VERSION은 1, 2, 3 중 하나여야 합니다(받은 값: {version:?})");
    }
    println!("cargo::rerun-if-env-changed=TALLY_C_VERSION");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Cargo가 설정하는 변수입니다");
    let csrc = PathBuf::from(manifest_dir)
        .join("../../csrc")
        .canonicalize()
        .expect("C 소스 디렉터리가 있어야 합니다")
        .join(format!("v{version}"));
    println!("cargo::rerun-if-changed=../../csrc/v{version}");

    // 1. C 소스를 정적 라이브러리로 컴파일하고 링크 지시문을 출력합니다(ch34와 같음).
    cc::Build::new()
        .file(csrc.join("tally.c"))
        .include(&csrc)
        .warnings_into_errors(true)
        .compile("tally");

    // 2. bindgen 기능이 켜져 있으면 같은 판의 헤더에서 선언을 생성합니다.
    #[cfg(feature = "bindgen")]
    generate(&csrc.join("tally.h"));
    let declarations = if cfg!(feature = "bindgen") {
        "generated"
    } else {
        "committed"
    };

    // 3. 이 패키지에 직접 의존하는 패키지의 빌드 스크립트에 넘길 값입니다.
    //    links = "tally"이므로 DEP_TALLY_INCLUDE, DEP_TALLY_C_VERSION, DEP_TALLY_DECLARATIONS가 됩니다.
    println!("cargo::metadata=include={}", csrc.display());
    println!("cargo::metadata=c_version={version}");
    println!("cargo::metadata=declarations={declarations}");
}

#[cfg(feature = "bindgen")]
fn generate(header: &std::path::Path) {
    let bindings = bindgen::Builder::default()
        .header(header.to_str().expect("헤더 경로가 UTF-8이어야 합니다"))
        // 생성 범위를 이 라이브러리의 접두사로 좁힙니다. 좁히지 않으면 <stdint.h>가 끌어오는
        // 플랫폼 내부 매크로(__DARWIN_… 등)까지 생성되어 결과가 기계마다 달라집니다.
        .allowlist_function("tally_.*")
        .allowlist_type("tally_.*")
        .allowlist_var("TALLY_.*")
        // 헤더(와 헤더가 포함하는 파일)가 바뀌면 이 스크립트를 다시 실행하게 합니다.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // 나머지는 기본값입니다: Rust 대상 판은 이 bindgen이 아는 가장 새 stable(unsafe extern
        // 블록을 생성), enum은 상수, Debug·Copy 파생, 레이아웃 검사 켬.
        .generate()
        .expect("bindgen이 헤더에서 선언을 생성하지 못했습니다");

    let out = PathBuf::from(env::var("OUT_DIR").expect("Cargo가 설정하는 변수입니다"));
    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("OUT_DIR에 bindings.rs를 쓰지 못했습니다");
}
