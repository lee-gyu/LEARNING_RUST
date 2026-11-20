//! 매니페스트(`Cargo.toml`)의 표를 종류별로 나눠 봅니다.
//!
//! `projects/unit-tools/Cargo.toml`을 읽어 표 머리(`[package]`, `[[bin]]` …)마다
//! 그 표가 무엇을 정하는지와 안에 적힌 키를 출력합니다. 매니페스트가 길어져도
//! "패키지 정보 / 타깃 / 의존성 / 린트"라는 큰 묶음으로 읽으면 길을 잃지 않습니다.
//!
//! 실행: cargo run -p p3-ch3-manifest-targets --example 01_manifest_map

/// 컴파일할 때 매니페스트의 내용을 문자열로 넣습니다(`include_str!`).
const MANIFEST: &str = include_str!("../projects/unit-tools/Cargo.toml");

/// 표 이름을 보고 그 표가 정하는 것을 돌려줍니다.
fn describe(table: &str) -> &'static str {
    // `package.metadata.unit-tools`처럼 점으로 이어진 이름은 앞부분으로 가릅니다.
    match table {
        "package" => "패키지 정보",
        "lib" => "타깃: 라이브러리(패키지에 하나)",
        "bin" => "타깃: 바이너리",
        "example" => "타깃: 예제",
        "test" => "타깃: 통합 테스트",
        "bench" => "타깃: 벤치마크",
        "dependencies" | "dev-dependencies" | "build-dependencies" => "의존성",
        "features" => "기능",
        "workspace" => "워크스페이스",
        _ if table.starts_with("package.metadata") => "다른 도구용 메타데이터(Cargo는 읽지 않음)",
        _ if table.starts_with("lints.") => "린트 수준",
        _ if table.starts_with("profile.") => "프로필",
        _ if table.starts_with("target.") => "플랫폼별 의존성",
        _ if table.starts_with("patch.") => "의존성 바꿔 끼우기",
        _ => "그 밖",
    }
}

fn print_table(header: &str, keys: &[&str]) {
    // `[[이름]]`은 같은 이름으로 여러 번 쓸 수 있는 표(TOML의 표 배열)입니다.
    let name = header.trim_matches(['[', ']']);
    println!("{header}  — {}", describe(name));
    if keys.is_empty() {
        println!("    (비어 있음)");
    } else {
        println!("    {}", keys.join(", "));
    }
}

fn main() {
    let mut header: Option<&str> = None;
    let mut keys: Vec<&str> = Vec::new();

    for line in MANIFEST.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            // 새 표가 시작되면 지금까지 모은 표를 출력합니다.
            if let Some(previous) = header {
                print_table(previous, &keys);
            }
            header = Some(line);
            keys.clear();
        } else if let Some((key, _value)) = line.split_once(" = ")
            && !line.starts_with('#')
        {
            keys.push(key);
        }
    }
    if let Some(last) = header {
        print_table(last, &keys);
    }
    // 출력: [package]  — 패키지 정보
    // 출력:     name, version, edition, rust-version, description, default-run, publish
    // 출력: [package.metadata.unit-tools]  — 다른 도구용 메타데이터(Cargo는 읽지 않음)
    // 출력:     default-precision
    // 출력: [lib]  — 타깃: 라이브러리(패키지에 하나)
    // 출력:     name
    // 출력: [[bin]]  — 타깃: 바이너리
    // 출력:     name, path, test
    // 출력: [[example]]  — 타깃: 예제
    // 출력:     name, path
    // 출력: [[test]]  — 타깃: 통합 테스트
    // 출력:     name, harness
    // 출력: [dependencies]  — 의존성
    // 출력:     (비어 있음)
    // 출력: [lints.rust]  — 린트 수준
    // 출력:     unsafe_code, missing_docs
    // 출력: [lints.clippy]  — 린트 수준
    // 출력:     pedantic, must_use_candidate
    // 출력: [lints.cargo]  — 린트 수준
    // 출력:     non_kebab_case_bins
    // 출력: [workspace]  — 워크스페이스
    // 출력:     (비어 있음)
}
