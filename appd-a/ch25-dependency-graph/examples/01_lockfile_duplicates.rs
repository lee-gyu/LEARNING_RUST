//! 지침 2: 중복 버전은 잠금 파일에 "같은 이름, 다른 버전"으로 남습니다.
//!
//! 이 챕터의 `projects/`에 있는 `Cargo.lock` 셋을 읽어, 여러 버전으로 들어온 패키지와
//! 그 버전을 요구한 패키지를 찾습니다. `cargo tree -d`(`--duplicates`)가 보여 주는 것을
//! 잠금 파일만으로 다시 계산해 보면서, 잠금 파일이 그래프를 어떻게 적는지 확인합니다.
//!
//! 실행: cargo run -p er-ch25-dependency-graph --example 01_lockfile_duplicates

use std::collections::BTreeMap;

// 잠금 파일은 컴파일할 때 실행 파일 안에 그대로 담습니다(include_str!의 경로는 이 파일 기준).
const LOCKS: [(&str, &str); 3] = [
    (
        "projects/planner/before",
        include_str!("../projects/planner/before/Cargo.lock"),
    ),
    (
        "projects/planner/after",
        include_str!("../projects/planner/after/Cargo.lock"),
    ),
    (
        "projects/real_graph",
        include_str!("../projects/real_graph/Cargo.lock"),
    ),
];

/// 잠금 파일의 `[[package]]` 항목 하나입니다.
struct Package {
    name: String,
    version: String,
    /// `source` 줄이 있으면 레지스트리 같은 바깥 출처에서 온 패키지입니다.
    /// 없으면 이 프로젝트 자신(워크스페이스 멤버)이나 path 의존성입니다.
    has_source: bool,
    /// 잠금 파일에 적힌 그대로의 의존성 표기입니다. 예: "remind-kit", "tiny-date 1.6.0"
    deps: Vec<String>,
}

impl Package {
    fn id(&self) -> String {
        format!("{} {}", self.name, self.version)
    }
}

/// 잠금 파일을 줄 단위로 읽습니다. Cargo가 만든 파일만 읽으면 되므로 TOML 전체를 다루지는 않습니다.
fn parse(lock: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut current: Option<Package> = None;
    let mut in_deps = false;
    for line in lock.lines().map(str::trim) {
        if line == "[[package]]" {
            packages.extend(current.take());
            current = Some(Package {
                name: String::new(),
                version: String::new(),
                has_source: false,
                deps: Vec::new(),
            });
            in_deps = false;
            continue;
        }
        // 첫 [[package]] 앞의 `version = 4`는 잠금 파일 형식의 버전이라 건너뜁니다.
        let Some(p) = current.as_mut() else { continue };
        if in_deps {
            if line == "]" {
                in_deps = false;
            } else {
                p.deps.push(dep_entry(line));
            }
        } else if let Some(v) = line.strip_prefix("name = ") {
            p.name = v.trim_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("version = ") {
            p.version = v.trim_matches('"').to_string();
        } else if line.starts_with("source = ") {
            p.has_source = true;
        } else if line == "dependencies = [" {
            in_deps = true;
        }
    }
    packages.extend(current);
    packages
}

/// `"tiny-date 1.6.0",` 같은 줄에서 따옴표와 쉼표를 걷어 냅니다.
fn dep_entry(line: &str) -> String {
    line.trim_end_matches(',').trim_matches('"').to_string()
}

/// 의존성 표기 "이름" 또는 "이름 버전"이 가리키는 패키지인지 봅니다.
/// 같은 이름이 하나뿐이면 Cargo는 버전을 생략하고, 둘 이상이면 버전을 붙여 구별합니다.
fn refers_to(dep: &str, target: &Package) -> bool {
    let mut parts = dep.split_whitespace();
    let name = parts.next().unwrap_or_default();
    match parts.next() {
        Some(version) => name == target.name && version == target.version,
        None => name == target.name,
    }
}

fn report(label: &str, lock: &str) {
    let packages = parse(lock);
    let external = packages.iter().filter(|p| p.has_source).count();
    println!(
        "[{label}] 패키지 {}개 (바깥 출처 {external}개)",
        packages.len()
    );

    // 이름별로 버전을 모읍니다. BTreeMap이라 이름순으로 출력됩니다.
    let mut by_name: BTreeMap<&str, Vec<&Package>> = BTreeMap::new();
    for p in &packages {
        by_name.entry(p.name.as_str()).or_default().push(p);
    }

    let mut found = false;
    for versions in by_name.values().filter(|v| v.len() > 1) {
        for target in versions {
            let users: Vec<String> = packages
                .iter()
                .filter(|p| p.deps.iter().any(|d| refers_to(d, target)))
                .map(Package::id)
                .collect();
            println!("  중복: {} ← {}", target.id(), users.join(", "));
            found = true;
        }
    }
    if !found {
        println!("  중복: 없음");
    }

    // source가 없는 패키지가 이 프로젝트 자신입니다. 그 의존성 표기를 그대로 보여 줍니다.
    for root in packages.iter().filter(|p| !p.has_source) {
        println!("  {}의 dependencies = {:?}", root.id(), root.deps);
    }
}

fn main() {
    for (label, lock) in LOCKS {
        report(label, lock);
    }
}

// 출력:
// [projects/planner/before] 패키지 5개 (바깥 출처 4개)
//   중복: tiny-date 1.6.0 ← cal-grid 0.9.1, remind-kit 0.4.2
//   중복: tiny-date 2.1.0 ← planner 0.1.0
//   planner 0.1.0의 dependencies = ["cal-grid", "remind-kit", "tiny-date 2.1.0"]
// [projects/planner/after] 패키지 4개 (바깥 출처 3개)
//   중복: 없음
//   planner 0.2.0의 dependencies = ["cal-grid", "remind-kit", "tiny-date"]
// [projects/real_graph] 패키지 18개 (바깥 출처 17개)
//   중복: syn 2.0.119 ← darling_core 0.20.11, darling_macro 0.20.11, derive_builder_core 0.20.2, derive_builder_macro 0.20.2
//   중복: syn 3.0.10 ← thiserror-impl 2.0.21
//   ticket_desk 0.1.0의 dependencies = ["anyhow", "derive_builder", "thiserror"]
