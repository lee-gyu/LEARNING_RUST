//! Cargo가 `projects/bookshelf` 워크스페이스에서 알아본 멤버와, 물려받은 값이
//! 실제로 무엇이 됐는지를 Cargo에게 물어봅니다.
//!
//! 멤버의 `Cargo.toml`에는 `version.workspace = true`라고만 적혀 있습니다.
//! `cargo metadata`는 그것을 풀어 낸 결과(`0.3.0`)를 알려 줍니다.
//!
//! 실행: cargo run -p p3-ch4-workspaces --example 02_workspace_members
//!
//! `cargo metadata --no-deps`를 자식 프로세스로 한 번 실행합니다(빌드도 네트워크도 쓰지 않습니다).
//! ch3의 예제 02와 같은 방식으로 JSON에서 필요한 값만 문자열 검색으로 꺼냅니다.

use std::env;
use std::fs;
use std::process::Command;

const CHAPTER_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// JSON 조각에서 `"key":"value"` 꼴의 문자열 값 가운데 첫 번째를 꺼냅니다.
fn first<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\":\"");
    let start = json.find(&marker)? + marker.len();
    let end = start + json[start..].find('"')?;
    Some(&json[start..end])
}

/// JSON 조각에서 `"key":"value"` 꼴의 문자열 값 가운데 마지막 것을 꺼냅니다.
fn last<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{key}\":\"");
    let start = json.rfind(&marker)? + marker.len();
    let end = start + json[start..].find('"')?;
    Some(&json[start..end])
}

fn main() {
    let root = format!("{CHAPTER_DIR}/projects/bookshelf");
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .arg("--manifest-path")
        .arg(format!("{root}/Cargo.toml"))
        .output()
        .expect("cargo를 실행할 수 있어야 합니다");
    if !output.status.success() {
        // 실패한 채로 계속하면 빈 출력을 "멤버가 아니다"로 잘못 읽게 됩니다.
        eprintln!("cargo metadata가 실패했습니다:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    let json = String::from_utf8_lossy(&output.stdout);

    println!(
        "{:<13} {:<8} {:<8} {:<13} member manifest",
        "name", "version", "edition", "rust-version"
    );
    // 패키지마다 `"manifest_path"`가 한 번씩 나옵니다. 그 앞쪽에 이름과 버전이,
    // 뒤쪽에 패키지의 에디션과 rust-version이 있습니다.
    let pieces: Vec<&str> = json.split("\"manifest_path\":\"").collect();
    for pair in pieces.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        // 타깃과 의존성에는 `"version"` 키가 없으므로 마지막 `"version"`이 패키지의 버전입니다.
        let version = last(before, "version").unwrap_or("?");
        let name = before
            .rfind("\"version\":\"")
            .and_then(|at| last(&before[..at], "name"))
            .unwrap_or("?");
        let edition = first(after, "edition").unwrap_or("?");
        let rust_version = first(after, "rust_version").unwrap_or("-");

        // 멤버의 Cargo.toml에는 버전이 어떻게 적혀 있는지 함께 보여 줍니다.
        let manifest_path = after.split('"').next().unwrap_or("");
        let written = fs::read_to_string(manifest_path)
            .ok()
            .and_then(|text| {
                text.lines()
                    .find(|line| line.starts_with("version"))
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "?".to_string());
        println!("{name:<13} {version:<8} {edition:<8} {rust_version:<13} {written}");
    }
    // 출력: name          version  edition  rust-version  member manifest
    // 출력: shelf-cli     0.3.0    2024     1.85          version.workspace = true
    // 출력: shelf-core    0.3.0    2024     1.85          version.workspace = true
    // 출력: shelf-report  0.3.0    2024     1.85          version.workspace = true

    println!();
    // crates/ 아래에는 디렉터리가 넷 있지만 멤버는 셋입니다. 루트의 `exclude`가 하나를 뺐습니다.
    let listed = json.contains("crates/shelf-wip");
    println!(
        "crates/shelf-wip은 멤버인가: {}",
        if listed { "예" } else { "아니요(exclude)" }
    );
    // 출력: crates/shelf-wip은 멤버인가: 아니요(exclude)
}
