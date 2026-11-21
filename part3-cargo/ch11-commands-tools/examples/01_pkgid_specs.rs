//! 패키지 ID 지정(Package ID Specification)으로 의존성 그래프의 패키지 하나를 가리켜 봅니다.
//! 이 저장소의 워크스페이스에는 `rand`가 두 버전 들어 있어, 이름만으로는 모호합니다.
//!
//! 실행: cargo run -q -p p3-ch11-commands-tools --example 01_pkgid_specs
//!
//! `cargo pkgid <지정>`을 자식 프로세스로 실행합니다. 빌드도 네트워크도 쓰지 않습니다(Cargo.lock을 읽음).

use std::path::Path;
use std::process::Command;

fn main() {
    // `cargo run`이 알려 주는 cargo의 경로. 같은 툴체인의 cargo를 부르려고 씁니다.
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    // 이 챕터 디렉터리의 두 단계 위가 저장소의 루트 워크스페이스입니다.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize().expect("저장소 루트");
    let manifest = root.join("Cargo.toml");
    let prefix = format!("file://{}", root.display());

    for spec in [
        "p3-ch11-commands-tools",
        "rand",
        "rand@0.8",
        "rand@0.10.4",
        "https://github.com/rust-lang/crates.io-index#rand@0.10.4",
        "rand@0.9",
    ] {
        let out = Command::new(&cargo)
            .args(["pkgid", "--manifest-path"])
            .arg(&manifest)
            .arg(spec)
            .output()
            .expect("cargo pkgid 실행");
        println!("$ cargo pkgid {spec}");
        let text = if out.status.success() {
            String::from_utf8_lossy(&out.stdout).into_owned()
        } else {
            String::from_utf8_lossy(&out.stderr).into_owned()
        };
        // 절대 경로는 컴퓨터마다 다르므로 줄입니다.
        for line in text.lines() {
            if line.is_empty() {
                println!();
            } else {
                println!("  {}", line.replace(&prefix, "file://…"));
            }
        }
    }

    // 출력: $ cargo pkgid p3-ch11-commands-tools
    // 출력:   path+file://…/part3-cargo/ch11-commands-tools#p3-ch11-commands-tools@0.0.0
    // 출력: $ cargo pkgid rand
    // 출력:   error: specification `rand` is ambiguous
    // 출력:   help: re-run this command with one of the following specifications
    // 출력:     rand@0.8.8
    // 출력:     rand@0.10.4
    // 출력: $ cargo pkgid rand@0.8
    // 출력:   registry+https://github.com/rust-lang/crates.io-index#rand@0.8.8
    // 출력: $ cargo pkgid rand@0.10.4
    // 출력:   registry+https://github.com/rust-lang/crates.io-index#rand@0.10.4
    // 출력: $ cargo pkgid https://github.com/rust-lang/crates.io-index#rand@0.10.4
    // 출력:   registry+https://github.com/rust-lang/crates.io-index#rand@0.10.4
    // 출력: $ cargo pkgid rand@0.9
    // 출력:   error: package ID specification `rand@0.9` did not match any packages
    // 출력:
    // 출력:   help: a package with a similar name exists: `rand`
}
