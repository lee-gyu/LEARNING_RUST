// 부록 B ch16 · Q16.1 — Cargo와 Go 모듈은 같은 요구 사항에서 다른 버전을 고른다
//
// 두 크레이트(A, B)가 같은 라이브러리 lib를 요구하고, 레지스트리에 lib의 여러 버전이 있다고 합시다.
// - Cargo: 의존성 버전 "1.2"는 캐럿 요구 사항 ^1.2.0과 같고(Cargo Book "Specifying Dependencies"),
//   SemVer로 호환되는 범위에서 가장 높은 버전을 고른 뒤 Cargo.lock에 적어 고정합니다.
//   잠금 파일이 있으면 cargo update 전까지 그 버전을 계속 씁니다.
// - Go 모듈: require v1.2.0은 "최소 v1.2.0"이라는 뜻이고, 최소 버전 선택(MVS)은 요구된 최소 버전들 가운데
//   가장 높은 것을 고릅니다(Go Modules Reference "Minimal version selection"). 잠금 파일 없이 go.mod만으로
//   정해지고 새 버전이 나와도 바뀌지 않으며, go.sum은 버전을 고정하지 않고 내려받은 파일의 해시를 적습니다.
// 이 예제는 두 규칙을 표준 라이브러리만으로 흉내 내어 비교합니다. 캐럿 규칙 자체는 Part 3 ch2 2.2절과 예제
// 01_version_requirements에서 semver 크레이트로 확인했고, 여기서는 그 규칙을 손으로 옮겨 Go의 규칙과 나란히 놓습니다.
// 실제 Cargo가 같은 상황에서 1.6.0을 고르는 모습은 부록 A ch25 2.3절에서 실행으로 확인했습니다. 실제 해석기는
// 기능(features), 프리릴리스, 되돌아가기 등을 더 다룹니다.
//
// 실행: cargo run -p faq-ch16-tooling-ecosystem --example 01_version_selection

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u64, u64, u64);

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// Cargo의 캐럿 요구 사항 ^req와 호환되는가. 가장 왼쪽의 0이 아닌 자리까지 같아야 하고, req 이상이어야 합니다.
fn caret_matches(req: Version, v: Version) -> bool {
    let same_left = match req {
        Version(0, 0, _) => v.0 == 0 && v.1 == 0 && v.2 == req.2,
        Version(0, minor, _) => v.0 == 0 && v.1 == minor,
        Version(major, _, _) => v.0 == major,
    };
    same_left && v >= req
}

/// Cargo처럼: 모든 요구 사항과 호환되는 버전 가운데 가장 높은 것
fn cargo_pick(available: &[Version], reqs: &[Version]) -> Option<Version> {
    available
        .iter()
        .copied()
        .filter(|&v| reqs.iter().all(|&r| caret_matches(r, v)))
        .max()
}

/// Go MVS처럼: 요구된 최소 버전들 가운데 가장 높은 것(같은 메이저 버전 안에서만 맞는 흉내, 연습 1·2)
fn mvs_pick(reqs: &[Version]) -> Option<Version> {
    reqs.iter().copied().max()
}

fn show(available: &[Version], reqs: &[Version]) {
    let reqs_text: Vec<String> = reqs.iter().map(|r| r.to_string()).collect();
    println!("  요구 사항: {}", reqs_text.join(", "));
    match cargo_pick(available, reqs) {
        Some(v) => println!("  Cargo(잠금 파일 없음): {v}"),
        None => println!("  Cargo(잠금 파일 없음): 맞는 버전 없음"),
    }
    if let Some(v) = mvs_pick(reqs) {
        println!("  Go MVS: {v}");
    }
}

fn main() {
    let mut registry = vec![
        Version(1, 2, 0),
        Version(1, 4, 0),
        Version(1, 6, 0),
        Version(2, 0, 0),
    ];
    let reqs = [Version(1, 2, 0), Version(1, 4, 0)]; // A는 1.2 이상, B는 1.4 이상

    println!("1. 레지스트리: 1.2.0, 1.4.0, 1.6.0, 2.0.0"); // 출력: 1. 레지스트리: 1.2.0, 1.4.0, 1.6.0, 2.0.0
    show(&registry, &reqs);
    // 출력:   요구 사항: 1.2.0, 1.4.0
    // 출력:   Cargo(잠금 파일 없음): 1.6.0
    // 출력:   Go MVS: 1.4.0

    println!("2. 1.7.0이 새로 올라온 뒤"); // 출력: 2. 1.7.0이 새로 올라온 뒤
    registry.push(Version(1, 7, 0));
    show(&registry, &reqs);
    // 출력:   요구 사항: 1.2.0, 1.4.0
    // 출력:   Cargo(잠금 파일 없음): 1.7.0
    // 출력:   Go MVS: 1.4.0
    println!("  Cargo.lock이 있으면: cargo update 전까지 1.6.0을 그대로 씀"); // 출력:   Cargo.lock이 있으면: cargo update 전까지 1.6.0을 그대로 씀

    println!("3. 0.x 버전의 캐럿 규칙"); // 출력: 3. 0.x 버전의 캐럿 규칙
    for v in [Version(0, 4, 1), Version(0, 4, 9), Version(0, 5, 0)] {
        println!(
            "  ^0.4.1 요구 사항에 {v}: {}",
            caret_matches(Version(0, 4, 1), v)
        );
    }
    // 출력:   ^0.4.1 요구 사항에 0.4.1: true
    // 출력:   ^0.4.1 요구 사항에 0.4.9: true
    // 출력:   ^0.4.1 요구 사항에 0.5.0: false
}
