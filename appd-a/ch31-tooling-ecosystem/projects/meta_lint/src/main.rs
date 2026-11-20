//! `cargo metadata`의 출력을 `cargo_metadata` 크레이트의 타입으로 읽어, 워크스페이스
//! 멤버가 게시 규칙 두 가지를 지키는지 검사하는 작은 도구입니다.
//!
//! 이 디렉터리(`projects/meta_lint`)에서 실행합니다.
//!
//! ```text
//! cargo run                  # sample_ws/Cargo.toml 검사, 위반이 있어 종료 코드 1
//! cargo run -- Cargo.toml    # 이 도구 자신 검사, 종료 코드 0
//! cargo test                 # 위 두 검사를 테스트로 확인
//! ```
//!
//! 규칙은 게시할 수 있는 패키지(publish를 막지 않은 패키지)에만 적용합니다.
//! 근거는 Cargo Book입니다.
//!
//! 1. `license`나 `license-file`이 있어야 합니다(Manifest Format 장: crates.io는
//!    둘 중 하나를 요구함).
//! 2. path 의존성에는 `version`도 있어야 합니다(Specifying Dependencies 장:
//!    crates.io 밖의 코드에 의존하는 패키지는 게시할 수 없고, dev-dependencies는 예외).
//!
//! 이 프로그램을 `cargo-<이름>` 바이너리로 만들어 `cargo <이름>`으로 부르는 방법은
//! Part 3 `ch11-commands-tools`에서 다룹니다. 여기서는 "메타데이터를 읽는 도구가
//! 이렇게 만들어진다"까지만 보입니다.

use std::process::ExitCode;

use cargo_metadata::semver::VersionReq;
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package};

/// 패키지 하나의 판정입니다.
#[derive(Debug, PartialEq)]
enum Verdict {
    /// 게시하지 않는 패키지라 규칙을 적용하지 않음
    Skipped,
    Passed,
    Failed(Vec<String>),
}

/// `publish` 필드는 `None`이면 제한 없음, `Some(빈 목록)`이면 게시 금지
/// (`publish = false`)입니다. JSON의 `null`과 `[]`를 타입이 구분해 줍니다.
fn is_publishable(p: &Package) -> bool {
    !matches!(&p.publish, Some(registries) if registries.is_empty())
}

fn check_package(p: &Package) -> Verdict {
    if !is_publishable(p) {
        return Verdict::Skipped;
    }
    let mut problems = Vec::new();
    if p.license.is_none() && p.license_file.is_none() {
        problems.push("license와 license-file이 모두 없습니다".to_string());
    }
    for dep in &p.dependencies {
        // version 없이 적은 의존성의 요구 사항은 "*"로 나옵니다. 문자열을 비교하지 않고
        // semver 타입의 상수와 비교하므로 표기가 달라져도 뜻으로 비교됩니다.
        let path_without_version = dep.path.is_some() && dep.req == VersionReq::STAR;
        if dep.kind != DependencyKind::Development && path_without_version {
            problems.push(format!("path 의존성 `{}`에 version이 없습니다", dep.name));
        }
    }
    if problems.is_empty() {
        Verdict::Passed
    } else {
        Verdict::Failed(problems)
    }
}

/// 워크스페이스 멤버를 이름순으로 판정합니다. `packages`에는 `--no-deps`가 없으면
/// 의존성 패키지도 들어 있으므로, `workspace_members`에 든 것만 고릅니다
/// (`workspace_packages`가 그 일을 합니다).
fn check(meta: &Metadata) -> Vec<(String, Verdict)> {
    let mut verdicts: Vec<(String, Verdict)> = meta
        .workspace_packages()
        .into_iter()
        .map(|p| (p.name.to_string(), check_package(p)))
        .collect();
    verdicts.sort_by(|a, b| a.0.cmp(&b.0));
    verdicts
}

fn main() -> ExitCode {
    let manifest = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sample_ws/Cargo.toml".to_string());

    // MetadataCommand는 `$CARGO`(없으면 PATH의 cargo)로 `cargo metadata
    // --format-version 1`을 실행하고 JSON을 Metadata 타입으로 읽습니다.
    let meta = match MetadataCommand::new()
        .manifest_path(&manifest)
        .no_deps()
        .exec()
    {
        Ok(meta) => meta,
        Err(e) => {
            eprintln!("meta-lint: cargo metadata를 읽지 못했습니다: {e}");
            return ExitCode::from(2);
        }
    };

    let verdicts = check(&meta);
    println!("meta-lint: {manifest} (멤버 {}개)", verdicts.len());
    let mut failures = 0;
    for (name, verdict) in &verdicts {
        match verdict {
            Verdict::Skipped => println!("  {name:<14} 건너뜀 (publish = false)"),
            Verdict::Passed => println!("  {name:<14} 통과"),
            Verdict::Failed(problems) => {
                println!("  {name:<14} 위반 {}건", problems.len());
                for problem in problems {
                    println!("    - {problem}");
                }
                failures += problems.len();
            }
        }
    }

    // CI에 넣을 검사 도구는 결과를 종료 코드로 알려야 합니다(README 2.2절).
    if failures == 0 {
        println!("위반 없음");
        ExitCode::SUCCESS
    } else {
        println!("위반 {failures}건");
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata_of(relative: &str) -> Metadata {
        let manifest = format!("{}/{relative}", env!("CARGO_MANIFEST_DIR"));
        MetadataCommand::new()
            .manifest_path(manifest)
            .no_deps()
            .exec()
            .expect("cargo metadata가 실패하면 안 됩니다")
    }

    #[test]
    fn sample_ws_has_two_problems_in_ledger_report() {
        let verdicts = check(&metadata_of("sample_ws/Cargo.toml"));
        let names: Vec<&str> = verdicts.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["ledger-cli", "ledger-core", "ledger-report"]);
        assert_eq!(verdicts[0].1, Verdict::Skipped);
        assert_eq!(verdicts[1].1, Verdict::Passed);
        let Verdict::Failed(problems) = &verdicts[2].1 else {
            panic!("ledger-report는 위반이 있어야 합니다");
        };
        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn this_tool_is_skipped_because_publish_is_false() {
        let verdicts = check(&metadata_of("Cargo.toml"));
        assert_eq!(verdicts, [("meta-lint".to_string(), Verdict::Skipped)]);
    }
}
