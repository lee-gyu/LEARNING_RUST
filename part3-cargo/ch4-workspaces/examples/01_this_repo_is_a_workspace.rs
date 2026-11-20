//! 이 학습 저장소가 워크스페이스라는 것을 두 매니페스트로 확인합니다.
//!
//! 저장소 루트의 `Cargo.toml`은 `[workspace]`만 있는 가상 매니페스트이고,
//! 챕터마다의 `Cargo.toml`은 `edition.workspace = true`처럼 값을 루트에서 물려받습니다.
//! Part 0부터 써 온 `cargo run -p <패키지> --example <이름>`이 저장소 어디서나 통한 것도,
//! `Cargo.lock`과 `target/`이 루트에 하나씩만 있는 것도 이 구조 때문입니다.
//!
//! 실행: cargo run -p p3-ch4-workspaces --example 01_this_repo_is_a_workspace

/// 저장소 루트의 매니페스트(워크스페이스 루트)
const ROOT: &str = include_str!("../../../Cargo.toml");
/// 이 챕터 패키지의 매니페스트(워크스페이스 멤버)
const MEMBER: &str = include_str!("../Cargo.toml");

/// `[table]` 아래의 `키 = 값` 줄을 모읍니다. 다음 표가 시작되면 멈춥니다.
fn table<'a>(manifest: &'a str, name: &str) -> Vec<(&'a str, &'a str)> {
    let header = format!("[{name}]");
    manifest
        .lines()
        .skip_while(|line| line.trim() != header)
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .filter_map(|line| line.split_once(" = "))
        .collect()
}

fn main() {
    println!("== 루트 Cargo.toml: [workspace] ==");
    for (key, value) in table(ROOT, "workspace") {
        println!("{key} = {value}");
    }
    // 출력: resolver = "3"
    // 출력: members = ["part*/ch*", "appd-*/ch*"]

    println!();
    println!("== 루트 Cargo.toml: [workspace.package] ==");
    let shared = table(ROOT, "workspace.package");
    for (key, value) in &shared {
        println!("{key} = {value}");
    }
    // 출력: edition = "2024"
    // 출력: publish = false

    println!();
    println!("== 이 챕터의 Cargo.toml: [package] ==");
    for (key, value) in table(MEMBER, "package") {
        // `키.workspace = true`는 "이 값은 루트의 [workspace.package]에서 가져오라"는 뜻입니다.
        match key.strip_suffix(".workspace") {
            Some(inherited) => {
                let from_root = shared
                    .iter()
                    .find(|(shared_key, _)| *shared_key == inherited)
                    .map_or("(루트에 없음)", |(_, shared_value)| shared_value);
                println!("{key} = {value}  -> 루트의 {inherited} = {from_root}");
            }
            None => println!("{key} = {value}  (직접 적음)"),
        }
    }
    // 출력: name = "p3-ch4-workspaces"  (직접 적음)
    // 출력: version = "0.0.0"  (직접 적음)
    // 출력: edition.workspace = true  -> 루트의 edition = "2024"
    // 출력: publish.workspace = true  -> 루트의 publish = false
}
