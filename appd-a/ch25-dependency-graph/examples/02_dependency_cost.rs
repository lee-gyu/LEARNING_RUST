//! 지침 5: 직접 의존성 하나의 비용은 "그것을 빼면 사라지는 패키지"로 셉니다.
//!
//! 잠금 파일에서 직접 의존성마다 (1) 그것을 거쳐 들어오는 모든 패키지(전이 의존성 포함)와
//! (2) 그 가운데 다른 직접 의존성은 쓰지 않는 패키지, 즉 그 의존성을 빼면 함께 사라지는 패키지를 셉니다.
//! 새 의존성을 들일지 따질 때도 같은 질문을 합니다. 이미 그래프에 있는 패키지는 다시 비용이 들지 않습니다.
//!
//! 잠금 파일에는 다른 플랫폼에서만 쓰는 패키지도 적히므로, 일반적으로는 `cargo tree`로 다시 확인합니다.
//! 여기서 읽는 두 잠금 파일은 `cargo tree`에 나오는 패키지와 목록이 같은 것을 확인했습니다.
//!
//! 실행: cargo run -p er-ch25-dependency-graph --example 02_dependency_cost

use std::collections::{BTreeMap, BTreeSet};

const LOCKS: [(&str, &str); 2] = [
    (
        "projects/real_graph",
        include_str!("../projects/real_graph/Cargo.lock"),
    ),
    (
        "projects/planner/before",
        include_str!("../projects/planner/before/Cargo.lock"),
    ),
];

/// 잠금 파일의 `[[package]]` 항목 하나입니다. 01 예제와 같은 방식으로 줄 단위로 읽습니다.
struct Package {
    id: String, // "이름 버전"
    name: String,
    has_source: bool,
    deps: Vec<String>,
}

fn parse(lock: &str) -> Vec<Package> {
    let mut packages: Vec<Package> = Vec::new();
    let mut in_deps = false;
    for line in lock.lines().map(str::trim) {
        if line == "[[package]]" {
            packages.push(Package {
                id: String::new(),
                name: String::new(),
                has_source: false,
                deps: Vec::new(),
            });
            in_deps = false;
            continue;
        }
        let Some(p) = packages.last_mut() else {
            continue;
        };
        if in_deps {
            if line == "]" {
                in_deps = false;
            } else {
                p.deps.push(dep_entry(line));
            }
        } else if let Some(v) = line.strip_prefix("name = ") {
            p.name = v.trim_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("version = ") {
            // 잠금 파일에서는 name 줄이 version 줄보다 먼저 나옵니다.
            p.id = format!("{} {}", p.name, v.trim_matches('"'));
        } else if line.starts_with("source = ") {
            p.has_source = true;
        } else if line == "dependencies = [" {
            in_deps = true;
        }
    }
    packages
}

/// `"tiny-date 1.6.0",` 같은 줄에서 따옴표와 쉼표를 걷어 냅니다.
fn dep_entry(line: &str) -> String {
    line.trim_end_matches(',').trim_matches('"').to_string()
}

/// 의존성 표기("이름" 또는 "이름 버전")를 패키지 ID("이름 버전")로 바꿉니다.
fn resolve(dep: &str, packages: &[Package]) -> String {
    if dep.contains(' ') {
        // "이름 버전 (출처)" 꼴일 수도 있으므로 앞의 두 단어만 씁니다.
        dep.split_whitespace().take(2).collect::<Vec<_>>().join(" ")
    } else {
        // 버전이 없으면 그 이름의 패키지는 하나뿐입니다.
        let p = packages.iter().find(|p| p.name == dep);
        p.map(|p| p.id.clone()).unwrap_or_default()
    }
}

/// `start`에서 출발해 닿는 모든 패키지(자신 포함)를 모읍니다.
fn reachable(start: &str, graph: &BTreeMap<String, Vec<String>>) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![start.to_string()];
    while let Some(id) = stack.pop() {
        if seen.insert(id.clone()) {
            stack.extend(graph.get(&id).into_iter().flatten().cloned());
        }
    }
    seen
}

fn report(label: &str, lock: &str) {
    let packages = parse(lock);
    let graph: BTreeMap<String, Vec<String>> = packages
        .iter()
        .map(|p| {
            let deps = p.deps.iter().map(|d| resolve(d, &packages)).collect();
            (p.id.clone(), deps)
        })
        .collect();

    // source가 없는 패키지가 이 프로젝트 자신이고, 그 의존성이 직접 의존성입니다.
    let Some(root) = packages.iter().find(|p| !p.has_source) else {
        return;
    };
    let direct = &graph[&root.id];
    let reach: Vec<BTreeSet<String>> = direct.iter().map(|d| reachable(d, &graph)).collect();
    let all: BTreeSet<&String> = reach.iter().flatten().collect();
    println!(
        "[{label}] {}: 직접 의존성 {}개, 함께 들어온 패키지 {}개",
        root.id,
        direct.len(),
        all.len()
    );

    for (i, dep) in direct.iter().enumerate() {
        // 다른 직접 의존성이 하나도 닿지 않는 패키지만 남깁니다.
        let used_by_others = |id: &String| {
            reach
                .iter()
                .enumerate()
                .any(|(j, r)| j != i && r.contains(id))
        };
        let only_here: Vec<&String> = reach[i].iter().filter(|id| !used_by_others(id)).collect();
        println!(
            "  {dep}: 거쳐 들어오는 패키지 {}개, 빼면 사라지는 패키지 {}개",
            reach[i].len(),
            only_here.len()
        );
        let names: Vec<&str> = only_here.iter().map(|s| s.as_str()).collect();
        println!("    사라지는 것: {}", names.join(", "));
    }

    // 둘 이상의 직접 의존성이 함께 쓰는 패키지입니다. 하나를 빼도 남습니다.
    let shared: Vec<&str> = all
        .iter()
        .filter(|id| reach.iter().filter(|r| r.contains(**id)).count() > 1)
        .map(|s| s.as_str())
        .collect();
    println!("  나눠 쓰는 것: {}", shared.join(", "));
}

fn main() {
    for (label, lock) in LOCKS {
        report(label, lock);
    }
}

// 출력:
// [projects/real_graph] ticket_desk 0.1.0: 직접 의존성 3개, 함께 들어온 패키지 17개
//   anyhow 1.0.104: 거쳐 들어오는 패키지 1개, 빼면 사라지는 패키지 1개
//     사라지는 것: anyhow 1.0.104
//   derive_builder 0.20.2: 거쳐 들어오는 패키지 13개, 빼면 사라지는 패키지 10개
//     사라지는 것: darling 0.20.11, darling_core 0.20.11, darling_macro 0.20.11, derive_builder 0.20.2, derive_builder_core 0.20.2, derive_builder_macro 0.20.2, fnv 1.0.7, ident_case 1.0.1, strsim 0.11.1, syn 2.0.119
//   thiserror 2.0.21: 거쳐 들어오는 패키지 6개, 빼면 사라지는 패키지 3개
//     사라지는 것: syn 3.0.10, thiserror 2.0.21, thiserror-impl 2.0.21
//   나눠 쓰는 것: proc-macro2 1.0.107, quote 1.0.47, unicode-ident 1.0.27
// [projects/planner/before] planner 0.1.0: 직접 의존성 3개, 함께 들어온 패키지 4개
//   cal-grid 0.9.1: 거쳐 들어오는 패키지 2개, 빼면 사라지는 패키지 1개
//     사라지는 것: cal-grid 0.9.1
//   remind-kit 0.4.2: 거쳐 들어오는 패키지 2개, 빼면 사라지는 패키지 1개
//     사라지는 것: remind-kit 0.4.2
//   tiny-date 2.1.0: 거쳐 들어오는 패키지 1개, 빼면 사라지는 패키지 1개
//     사라지는 것: tiny-date 2.1.0
//   나눠 쓰는 것: tiny-date 1.6.0
// planner의 tiny-date 1.6.0은 remind-kit과 cal-grid가 함께 쓰므로, 둘 중 하나만 올려서는 사라지지 않습니다.
