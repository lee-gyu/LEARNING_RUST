//! Cargo home을 둘러봅니다. 내려받은 크레이트와 `cargo install`로 설치한
//! 도구가 모이는 디렉터리입니다.
//!
//! 실행: cargo run -p p3-ch1-getting-started --example 02_cargo_home
//!
//! **출력은 컴퓨터마다 다릅니다.** 무엇을 내려받고 설치했는지에 따라 "있음/없음"과
//! 개수가 달라집니다. 홈 디렉터리 경로는 출력하지 않고 `$CARGO_HOME`으로 적습니다.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Cargo home의 위치를 Cargo와 같은 규칙으로 찾습니다.
///
/// 1. 환경 변수 `CARGO_HOME`이 있으면 그 값
/// 2. 없으면 홈 디렉터리 아래의 `.cargo`
fn cargo_home() -> Option<(PathBuf, &'static str)> {
    if let Some(dir) = env::var_os("CARGO_HOME") {
        return Some((PathBuf::from(dir), "환경 변수 CARGO_HOME"));
    }
    // 유닉스 계열은 HOME, Windows는 USERPROFILE에 홈 디렉터리가 들어 있습니다.
    let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"))?;
    Some((PathBuf::from(home).join(".cargo"), "홈 디렉터리의 .cargo"))
}

/// 디렉터리 바로 아래의 항목 이름을 정렬해 돌려줍니다. 디렉터리가 없으면 빈 목록입니다.
fn entries(dir: &Path) -> Vec<String> {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read_dir
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn mark(exists: bool) -> &'static str {
    if exists { "있음" } else { "없음" }
}

fn main() {
    let Some((home, found_by)) = cargo_home() else {
        println!("Cargo home을 찾지 못했습니다.");
        return;
    };
    println!("Cargo home을 찾은 방법: {found_by}");
    // 출력: Cargo home을 찾은 방법: 환경 변수 CARGO_HOME
    // (`cargo run`으로 실행했을 때. rustup이 설치한 `cargo` 명령이 이 변수를 넣어 줍니다.)

    println!();
    println!("== 파일 ==");
    let files = [
        ("config.toml", "모든 패키지에 적용되는 Cargo 설정"),
        ("credentials.toml", "cargo login으로 저장한 레지스트리 토큰"),
        (".crates.toml", "cargo install로 설치한 크레이트 기록"),
        (".crates2.json", "cargo install로 설치한 크레이트 기록"),
    ];
    for (name, role) in files {
        println!("{:<4} {name:<18} {role}", mark(home.join(name).is_file()));
    }
    // 출력(컴퓨터마다 다름): 없음   config.toml        모든 패키지에 적용되는 Cargo 설정
    // 이하 "있음/없음"은 그 컴퓨터에서 한 일에 따라 달라집니다.

    println!();
    println!("== 디렉터리 ==");
    let dirs = [
        ("bin", "cargo install과 rustup이 넣은 실행 파일"),
        (
            "registry/index",
            "레지스트리 색인(어떤 크레이트의 어떤 버전이 있는가)",
        ),
        ("registry/cache", "내려받은 .crate 압축 파일"),
        ("registry/src", ".crate를 풀어 둔 소스(rustc가 읽는 곳)"),
        ("git/db", "git 의존성의 저장소 사본"),
        ("git/checkouts", "git 의존성에서 꺼낸 특정 커밋의 소스"),
    ];
    for (name, role) in dirs {
        println!("{:<4} {name:<18} {role}", mark(home.join(name).is_dir()));
    }

    println!();
    println!("== 레지스트리 캐시 ==");
    // `registry/cache` 아래에는 레지스트리마다 디렉터리가 하나씩 있습니다.
    let cache = home.join("registry").join("cache");
    let registries = entries(&cache);
    if registries.is_empty() {
        println!("아직 내려받은 크레이트가 없습니다.");
    }
    for registry in &registries {
        let crates = entries(&cache.join(registry));
        let count = crates
            .iter()
            .filter(|name| name.ends_with(".crate"))
            .count();
        println!("{registry}: .crate 파일 {count}개");
        // 출력(컴퓨터마다 다름): index.crates.io-1949cf8c6b5b557f: .crate 파일 282개

        // 이 챕터의 projects/text-stat이 의존하는 크레이트가 캐시에 있는지 찾아봅니다.
        let wanted = "unicode-segmentation-";
        let found: Vec<&String> = crates
            .iter()
            .filter(|name| name.starts_with(wanted))
            .collect();
        if found.is_empty() {
            println!(
                "  {wanted}*.crate 없음 (projects/text-stat에서 cargo fetch를 실행하면 생깁니다)"
            );
        }
        for name in found {
            println!("  $CARGO_HOME/registry/cache/{registry}/{name}");
            // 압축을 푼 소스는 확장자를 뺀 같은 이름의 디렉터리로 registry/src에 있습니다.
            let unpacked = name.trim_end_matches(".crate");
            let src = home
                .join("registry")
                .join("src")
                .join(registry)
                .join(unpacked);
            println!(
                "  $CARGO_HOME/registry/src/{registry}/{unpacked}/ ({})",
                mark(src.is_dir())
            );
        }
    }
}
