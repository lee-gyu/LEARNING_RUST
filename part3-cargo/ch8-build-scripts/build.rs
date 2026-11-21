//! 이 챕터 패키지의 빌드 스크립트입니다. 예제를 컴파일하기 전에 Cargo가 먼저 컴파일해 실행합니다.
//!
//! 1. data/units.txt(기능 `extra-units`가 켜지면 data/extra_units.txt도)를 읽어
//!    `OUT_DIR/units.rs`에 Rust 코드를 만듭니다(예제 01).
//! 2. Cargo가 넘겨준 빌드 정보를 `cargo::rustc-env`로 크레이트에 넘깁니다(예제 02).

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

fn main() {
    // 입력이 바뀔 때만 다시 실행하게 합니다. 적지 않으면 패키지의 어느 파일이 바뀌어도 다시 실행됩니다.
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=data/units.txt");

    let mut files = vec!["data/units.txt"];
    // 켜진 기능은 CARGO_FEATURE_<이름>(대문자, `-`는 `_`) 환경 변수로 들어옵니다.
    if env::var_os("CARGO_FEATURE_EXTRA_UNITS").is_some() {
        println!("cargo::rerun-if-changed=data/extra_units.txt");
        files.push("data/extra_units.txt");
    }

    let mut units: Vec<(String, String)> = Vec::new();
    for file in files {
        // 빌드 스크립트의 현재 디렉터리는 패키지 디렉터리입니다.
        let text = fs::read_to_string(file).unwrap_or_else(|e| panic!("{file}을 읽지 못함: {e}"));
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((name, factor)) = line.split_once(' ') else {
                // 빌드를 멈추고 메시지를 보여 줍니다.
                println!(
                    "cargo::error={file}:{}: `이름 배수` 형식이 아닙니다: {line}",
                    i + 1
                );
                return;
            };
            if units.iter().any(|(n, _)| n == name) {
                // 빌드는 계속하고 경고만 보여 줍니다. 먼저 나온 값을 씁니다.
                println!(
                    "cargo::warning={file}:{}: 단위 `{name}`이 이미 있어 무시합니다",
                    i + 1
                );
                continue;
            }
            units.push((name.to_string(), factor.trim().to_string()));
        }
    }

    // 만든 코드는 OUT_DIR에만 씁니다. 패키지 디렉터리의 파일을 고치면 안 됩니다.
    let mut code =
        String::from("// 빌드 스크립트가 data/*.txt로 만든 파일입니다. 고치지 마세요.\n\n");
    code.push_str("pub const UNITS: &[(&str, f64)] = &[\n");
    for (name, factor) in &units {
        writeln!(code, "    ({name:?}, {factor}_f64),").unwrap();
    }
    code.push_str("];\n\npub fn factor(unit: &str) -> Option<f64> {\n    match unit {\n");
    for (name, factor) in &units {
        writeln!(code, "        {name:?} => Some({factor}_f64),").unwrap();
    }
    code.push_str("        _ => None,\n    }\n}\n");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("units.rs");
    fs::write(&out, code).unwrap();

    // Cargo가 빌드 스크립트에 넘겨준 값을 크레이트가 env!로 읽을 수 있게 넘깁니다.
    for key in ["TARGET", "HOST", "PROFILE", "OPT_LEVEL", "DEBUG"] {
        println!("cargo::rustc-env=CH8_{key}={}", env::var(key).unwrap());
    }
}
