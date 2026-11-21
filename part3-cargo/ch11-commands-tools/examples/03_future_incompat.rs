//! 장래 비호환 보고서(future-incompat report): 의존성에 앞으로 에러가 될 코드가 있으면 Cargo가 알려 줍니다.
//!
//! 실행: cargo run -q -p p3-ch11-commands-tools --example 03_future_incompat
//!
//! 임시 디렉터리에 연습용 레지스트리(디렉터리 소스, Part 3 ch2 2.6절)를 만들고, 그 안의 의존성 `olddep`에
//! 매크로 끝의 세미콜론 문제(`semicolon_in_expressions_from_macros` 린트)를 넣습니다.
//! 그 의존성을 쓰는 패키지를 자식 프로세스의 cargo로 검사합니다. 네트워크는 쓰지 않습니다.

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn main() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let root = std::env::temp_dir().join(format!("p3-ch11-fir-{}", std::process::id()));

    // 의존성: 2021 에디션의 라이브러리. 표현식 자리에서 쓰는 매크로가 `true;`처럼 세미콜론으로 끝납니다.
    let dep = root.join("vendor/olddep-0.1.0");
    write(
        &dep.join("Cargo.toml"),
        "[package]\nname = \"olddep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dep.join("src/lib.rs"),
        "macro_rules! t {\n    () => { true; };\n}\n\npub fn a() -> bool {\n    let x = t!();\n    x\n}\n",
    );
    write(
        &dep.join(".cargo-checksum.json"),
        "{\"files\":{},\"package\":null}",
    );

    // 그 의존성을 쓰는 패키지. crates.io 대신 vendor/를 쓰도록 소스를 교체합니다.
    let app = root.join("app");
    write(
        &app.join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nolddep = \"0.1\"\n\n[workspace]\n",
    );
    write(
        &app.join("src/main.rs"),
        "fn main() {\n    println!(\"{}\", olddep::a());\n}\n",
    );
    write(
        &app.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = \"vend\"\n\n[source.vend]\ndirectory = \"../vendor\"\n",
    );

    // 설정 파일은 실행한 디렉터리에서 찾으므로(ch7 2.2절) app/ 안에서 실행합니다.
    let check = Command::new(&cargo)
        .arg("check")
        .current_dir(&app)
        .output()
        .unwrap();
    println!(
        "cargo check 종료 상태: {}",
        check.status.code().unwrap_or(-1)
    );
    for line in String::from_utf8_lossy(&check.stderr).lines() {
        if line.starts_with("warning: the following") || line.starts_with("note: to see") {
            println!("{line}");
        }
    }

    let report = Command::new(&cargo)
        .args(["report", "future-incompatibilities"])
        .current_dir(&app)
        .output()
        .unwrap();
    println!(
        "cargo report future-incompatibilities 종료 상태: {}",
        report.status.code().unwrap_or(-1)
    );
    for line in String::from_utf8_lossy(&report.stdout).lines() {
        if line.contains("currently triggers")
            || line.starts_with("> warning:")
            || line.contains("previously accepted")
        {
            println!("{line}");
        }
    }

    fs::remove_dir_all(&root).unwrap();

    // 출력: cargo check 종료 상태: 0
    // 출력: warning: the following packages contain code that will be rejected by a future version of Rust: olddep v0.1.0
    // 출력: note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
    // 출력: cargo report future-incompatibilities 종료 상태: 0
    // 출력: The package `olddep v0.1.0` currently triggers the following future incompatibility lints:
    // 출력: > warning: trailing semicolon in macro used in expression position
    // 출력: >   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
}
