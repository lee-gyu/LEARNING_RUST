//! CI 단계 "유효한 기능 조합을 모두 빌드": 검사할 기능 조합을 데이터로 세고 고릅니다.
//!
//! 기능 목록과 "이 기능이 켜는 다른 기능"을 데이터로 적고,
//! 1. 모든 부분집합(2^N개)과, 함께 켜지는 기능을 반영하면 실제로 다른 빌드가 몇 개인지,
//! 2. "없음 + 하나씩 + 전부"로 줄인 집합이 몇 개이고 어떤 cargo 인자가 되는지,
//! 3. 그 집합이 CI 작업 수(운영체제·툴체인과 곱함)로 얼마가 되는지를 셉니다.
//!
//! cargo를 실행하지 않고 계산만 하므로 바로 끝납니다.
//!
//! 실행: cargo run -p er-ch32-continuous-integration --example 03_feature_matrix

use std::collections::BTreeSet;

/// 기능 하나와, 그 기능이 함께 켜는 기능(`Cargo.toml`의 `a = ["b"]`)입니다.
struct Feature {
    name: &'static str,
    enables: &'static [&'static str],
}

struct Package {
    name: &'static str,
    features: &'static [Feature],
    default: &'static [&'static str],
}

type Set = BTreeSet<&'static str>;

const fn feature(name: &'static str, enables: &'static [&'static str]) -> Feature {
    Feature { name, enables }
}

/// 함께 켜지는 기능을 더 이상 늘지 않을 때까지 더합니다.
fn close(p: &Package, start: &Set) -> Set {
    let mut set = start.clone();
    loop {
        let before = set.len();
        for f in p.features {
            if set.contains(f.name) {
                set.extend(f.enables.iter().copied());
            }
        }
        if set.len() == before {
            return set;
        }
    }
}

/// 모든 부분집합을 만듭니다(기능 N개면 2^N개).
fn powerset(p: &Package) -> Vec<Set> {
    let n = p.features.len();
    (0..1u32 << n)
        .map(|mask| {
            (0..n)
                .filter(|i| mask & (1 << i) != 0)
                .map(|i| p.features[i].name)
                .collect()
        })
        .collect()
}

/// 기능 없음, 기본, 하나씩, 전부. 함께 켜지는 기능을 반영한 뒤 같은 것은 하나로 칩니다.
fn reduced(p: &Package) -> Vec<Set> {
    let mut picks: Vec<Set> = vec![Set::new(), p.default.iter().copied().collect()];
    picks.extend(p.features.iter().map(|f| Set::from([f.name])));
    picks.push(p.features.iter().map(|f| f.name).collect());
    let mut seen = BTreeSet::new();
    picks
        .into_iter()
        .map(|s| close(p, &s))
        .filter(|s| seen.insert(s.clone()))
        .collect()
}

/// 기능 집합을 cargo 인자로 바꿉니다. 함께 켜지는 기능은 cargo가 켜 주므로 적지 않아도 되지만,
/// 읽는 사람이 실제 빌드를 알 수 있게 닫힌 집합 그대로 적습니다.
fn cargo_args(p: &Package, s: &Set) -> String {
    if s.is_empty() {
        "--no-default-features".to_string()
    } else if s.len() == p.features.len() {
        "--all-features".to_string()
    } else {
        let list: Vec<&str> = s.iter().copied().collect();
        format!("--no-default-features --features {}", list.join(","))
    }
}

fn report(p: &Package) {
    let all = powerset(p);
    let distinct: BTreeSet<Set> = all.iter().map(|s| close(p, s)).collect();
    let picks = reduced(p);
    println!(
        "{}: 기능 {}개, 부분집합 {}개, 실제로 다른 빌드 {}개, 줄인 집합 {}개",
        p.name,
        p.features.len(),
        all.len(),
        distinct.len(),
        picks.len()
    );
    for s in &picks {
        println!("    cargo clippy --all-targets {}", cargo_args(p, s));
    }
}

const POLLCOUNT: Package = Package {
    name: "projects/pollcount",
    features: &[feature("csv", &[]), feature("json", &[])],
    default: &["csv"],
};

const FIVE: Package = Package {
    name: "독립 기능 다섯",
    features: &[
        feature("a", &[]),
        feature("b", &[]),
        feature("c", &[]),
        feature("d", &[]),
        feature("e", &[]),
    ],
    default: &[],
};

/// ch33의 `std`·`alloc`처럼 한 기능이 다른 기능을 켜는 패키지입니다.
const LAYERED: Package = Package {
    name: "std/alloc 계층",
    features: &[
        feature("std", &["alloc"]),
        feature("alloc", &[]),
        feature("json", &["alloc"]),
    ],
    default: &["std"],
};

fn main() {
    println!("1. 조합 수: 부분집합, 실제로 다른 빌드, 줄인 집합");
    report(&POLLCOUNT);
    // 출력: projects/pollcount: 기능 2개, 부분집합 4개, 실제로 다른 빌드 4개, 줄인 집합 4개
    // 출력:     cargo clippy --all-targets --no-default-features
    // 출력:     cargo clippy --all-targets --no-default-features --features csv
    // 출력:     cargo clippy --all-targets --no-default-features --features json
    // 출력:     cargo clippy --all-targets --all-features
    report(&FIVE);
    // 출력: 독립 기능 다섯: 기능 5개, 부분집합 32개, 실제로 다른 빌드 32개, 줄인 집합 7개
    // (cargo 인자 일곱 줄은 줄임)
    report(&LAYERED);
    // 출력: std/alloc 계층: 기능 3개, 부분집합 8개, 실제로 다른 빌드 5개, 줄인 집합 5개
    // 출력:     cargo clippy --all-targets --no-default-features
    // 출력:     cargo clippy --all-targets --no-default-features --features alloc,std
    // 출력:     cargo clippy --all-targets --no-default-features --features alloc
    // 출력:     cargo clippy --all-targets --no-default-features --features alloc,json
    // 출력:     cargo clippy --all-targets --all-features
    // 함께 켜지는 기능 때문에 8개 가운데 3개는 다른 부분집합과 같은 빌드입니다
    // (예: {std}와 {std, alloc}). 이런 패키지에서는 줄인 집합이 곧 모든 빌드입니다.

    println!("2. std 없는 타깃에서 빌드해야 하는 조합 (std/alloc 계층)");
    for s in reduced(&LAYERED) {
        if !s.contains("std") {
            println!(
                "    cargo build --target thumbv6m-none-eabi {}",
                cargo_args(&LAYERED, &s)
            );
        }
    }
    // 출력:     cargo build --target thumbv6m-none-eabi --no-default-features
    // 출력:     cargo build --target thumbv6m-none-eabi --no-default-features --features alloc
    // 출력:     cargo build --target thumbv6m-none-eabi --no-default-features --features alloc,json
    // std를 켠 조합은 호스트에서 빌드하고, 나머지는 std가 없는 타깃에서 빌드해야
    // 의존성이 몰래 std를 끌어오는 것을 잡습니다(ch33 2.5절).

    println!("3. CI 작업 수: 기능 조합 × 운영체제 × 툴체인");
    let os = ["ubuntu", "macos", "windows"];
    let toolchains = ["고정한 1.100.0", "MSRV 1.98.1"];
    for p in [&POLLCOUNT, &FIVE] {
        let every = powerset(p).len();
        let picks = reduced(p).len();
        println!(
            "  {}: 모든 조합 {} × {} × {} = {}개, 변경마다(줄인 집합, ubuntu, 고정 툴체인) {}개 + MSRV 1개",
            p.name,
            every,
            os.len(),
            toolchains.len(),
            every * os.len() * toolchains.len(),
            picks
        );
    }
    // 출력:   projects/pollcount: 모든 조합 4 × 3 × 2 = 24개, 변경마다(줄인 집합, ubuntu, 고정 툴체인) 4개 + MSRV 1개
    // 출력:   독립 기능 다섯: 모든 조합 32 × 3 × 2 = 192개, 변경마다(줄인 집합, ubuntu, 고정 툴체인) 7개 + MSRV 1개
    // 모두 곱한 매트릭스는 변경마다 돌리기에 너무 큽니다. 변경마다는 줄인 집합을 한 환경에서,
    // 모든 조합(cargo hack --feature-powerset)은 정기 예약 작업에서 돌리는 식으로 나눕니다.
}
