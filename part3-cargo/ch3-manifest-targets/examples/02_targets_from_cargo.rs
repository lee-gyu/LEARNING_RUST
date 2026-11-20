//! Cargo가 `projects/unit-tools`에서 실제로 인식한 타깃을 Cargo에게 물어봅니다.
//!
//! 타깃은 두 곳에서 옵니다. 관례가 찾아낸 것(`src/lib.rs`, `src/bin/*.rs` …)과
//! `Cargo.toml`의 타깃 표가 선언한 것(`[[bin]]`, `[[example]]` …)입니다.
//! `cargo metadata`는 둘을 합친 최종 결과를 JSON으로 출력합니다.
//!
//! 실행: cargo run -p p3-ch3-manifest-targets --example 02_targets_from_cargo
//!
//! `cargo metadata`를 자식 프로세스로 한 번 실행합니다(빌드는 하지 않습니다).
//! 이 명령과 JSON 출력은 Part 3 `ch11-commands-tools`에서 다룹니다. 여기서는
//! 필요한 값만 문자열 검색으로 꺼냅니다. 실제 도구라면 JSON 파서를 써야 합니다.

use std::env;
use std::process::Command;

/// 이 챕터 패키지의 디렉터리입니다. 컴파일할 때 Cargo가 넣어 주는 값입니다.
const CHAPTER_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// JSON 조각에서 `"key":"value"` 꼴의 문자열 값을 꺼냅니다.
fn text<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\":\"");
    let start = json.find(&marker)? + marker.len();
    let end = start + json[start..].find('"')?;
    Some(&json[start..end])
}

/// JSON 조각에서 `"key":true` 또는 `"key":false`를 읽어 표에 쓸 표시로 바꿉니다.
fn flag(json: &str, key: &str) -> &'static str {
    if json.contains(&format!("\"{key}\":true")) {
        "O"
    } else {
        "-"
    }
}

fn main() {
    let project = format!("{CHAPTER_DIR}/projects/unit-tools");
    // `cargo run`은 자기 경로를 환경 변수 CARGO에 넣어 줍니다. 없으면 PATH의 cargo를 씁니다.
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .arg("--manifest-path")
        .arg(format!("{project}/Cargo.toml"))
        .output()
        .expect("cargo를 실행할 수 있어야 합니다");
    let json = String::from_utf8_lossy(&output.stdout);

    // `[package]`에 적은 값 가운데 타깃과 관련된 것입니다.
    println!("default-run: {}", text(&json, "default_run").unwrap_or("-"));
    println!(
        "rust-version: {}",
        text(&json, "rust_version").unwrap_or("-")
    );
    println!();
    // 출력: default-run: unit-tools
    // 출력: rust-version: 1.85

    // "targets":[{…},{…}] 부분을 타깃 하나씩으로 자릅니다.
    let start = json
        .find("\"targets\":[")
        .expect("targets 배열이 있어야 합니다");
    let end = start
        + json[start..]
            .find("],\"features\"")
            .expect("targets 배열의 끝");
    let targets = &json[start..end];

    println!(
        "{:<8} {:<11} {:<18} {:<5} {:<8} doc",
        "kind", "name", "path", "test", "doctest"
    );
    for target in targets.split("{\"kind\":[\"").skip(1) {
        let kind = target.split('"').next().unwrap_or("?");
        let name = text(target, "name").unwrap_or("?");
        // 절대 경로에서 프로젝트 디렉터리까지를 떼어 내 상대 경로로 보여 줍니다.
        let path = text(target, "src_path")
            .and_then(|p| p.strip_prefix(&project))
            .map_or("?", |p| p.trim_start_matches('/'));
        println!(
            "{kind:<8} {name:<11} {path:<18} {:<5} {:<8} {}",
            flag(target, "test"),
            flag(target, "doctest"),
            flag(target, "doc"),
        );
    }
    // 출력: kind     name        path               test  doctest  doc
    // 출력: lib      units       src/lib.rs         O     O        O
    // 출력: bin      c2f         tools/c2f.rs       -     -        O
    // 출력: bin      km2mi       src/bin/km2mi.rs   O     -        O
    // 출력: bin      unit-tools  src/main.rs        O     -        O
    // 출력: example  quick       examples/quick.rs  -     -        -
    // 출력: example  table       demos/table.rs     -     -        -
    // 출력: test     convert     tests/convert.rs   O     -        -
    // 출력: test     report      tests/report.rs    O     -        -
}
