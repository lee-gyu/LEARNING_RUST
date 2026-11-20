//! 원칙 "경고는 실패": 경고를 CI의 실패로 만드는 세 가지 방법을 비교합니다.
//!
//! 임시 디렉터리에 경고가 하나 나는 작은 패키지를 만들고, 같은 패키지에 명령을 차례로
//! 실행해 종료 코드, 다시 컴파일했는지(`Checking` 줄이 나왔는지), 경고가 보이는 모양을
//! 비교합니다. 패키지는 의존성이 없어 명령마다 1초 안팎으로 끝납니다.
//!
//! 실행: cargo run -p er-ch32-continuous-integration --example 01_warnings_as_errors

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;

/// 쓰지 않는 변수 하나 때문에 `unused_variables` 경고가 납니다.
const SOURCE: &str = "fn main() {\n    let unused = 42;\n}\n";

/// 명령 한 번의 결과입니다.
struct Outcome {
    code: i32,
    /// `Checking warn-demo` 줄이 있으면 이 실행에서 다시 컴파일한 것입니다.
    rebuilt: bool,
    /// 사람이 볼 핵심 줄(경고·에러의 머리줄)만 모읍니다.
    headlines: Vec<String>,
}

/// `cargo run`이 넣어 준 `CARGO`(지금 쓰는 cargo)를 다시 부릅니다.
fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

/// `args` 앞에 환경 변수 `envs`를 붙여 실행합니다.
fn run(dir: &Path, envs: &[(&str, &str)], args: &[&str]) -> Outcome {
    let mut cmd = Command::new(cargo());
    cmd.args(args).current_dir(dir);
    // 바깥 환경이 결과를 바꾸지 않도록 경고 수준과 관련된 변수를 지우고,
    // 빌드 디렉터리는 임시 패키지 안에 둡니다(이 저장소의 빌드 디렉터리를 건드리지 않음).
    for key in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_RUSTFLAGS",
        "CARGO_BUILD_WARNINGS",
        "CARGO_BUILD_TARGET_DIR",
    ] {
        cmd.env_remove(key);
    }
    cmd.env("CARGO_TARGET_DIR", dir.join("target"))
        .env("CARGO_TERM_COLOR", "never");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("cargo를 실행하지 못했습니다");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let rebuilt = stderr
        .lines()
        .any(|l| l.trim_start().starts_with("Checking warn-demo"));
    // 요약 줄(`could not compile`, `generated 1 warning`)은 명령마다 문구가 달라 뺍니다.
    let headlines = stderr
        .lines()
        .filter(|l| {
            l.starts_with("warning: unused")
                || l.starts_with("error: unused")
                || l.starts_with("error: warnings are denied")
        })
        .map(str::to_owned)
        .collect();
    Outcome {
        code: out.status.code().unwrap_or(-1),
        rebuilt,
        headlines,
    }
}

fn show(label: &str, o: &Outcome) {
    let rebuilt = if o.rebuilt { "예" } else { "아니요" };
    println!("  {label}");
    println!("    종료 코드 {}, 다시 컴파일 {rebuilt}", o.code);
    for h in &o.headlines {
        println!("    | {h}");
    }
}

fn main() {
    let dir = std::env::temp_dir().join(format!("er-ch32-warn-{}", std::process::id()));
    fs::create_dir_all(dir.join("src")).expect("임시 디렉터리를 만들지 못했습니다");
    // 빈 [workspace]로 혼자인 워크스페이스로 만듭니다.
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"warn-demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .expect("Cargo.toml을 쓰지 못했습니다");
    fs::write(dir.join("src/main.rs"), SOURCE).expect("src/main.rs를 쓰지 못했습니다");

    println!("1. 비권장: 경고를 그대로 둠");
    show("cargo check", &run(&dir, &[], &["check"]));
    // 출력: 종료 코드 0, 다시 컴파일 예
    // 출력: | warning: unused variable: `unused`
    show("cargo check (한 번 더)", &run(&dir, &[], &["check"]));
    // 출력: 종료 코드 0, 다시 컴파일 아니요
    // 출력: | warning: unused variable: `unused`
    // Cargo는 경고를 저장해 두었다가 다시 보여 줍니다. 그래도 종료 코드가 0이라
    // CI는 이 경고를 성공으로 봅니다.

    println!("2. RUSTFLAGS=-Dwarnings: 실패하지만 처음 바꿀 때 다시 컴파일");
    show(
        "RUSTFLAGS=-Dwarnings cargo check",
        &run(&dir, &[("RUSTFLAGS", "-Dwarnings")], &["check"]),
    );
    // 출력: 종료 코드 101, 다시 컴파일 예
    // 출력: | error: unused variable: `unused`
    // RUSTFLAGS는 rustc에 그대로 넘어가는 플래그라 다른 컴파일로 칩니다. 실제 프로젝트라면
    // 의존성까지 모든 크레이트에 걸려 전부 다시 컴파일합니다.
    show(
        "cargo check (RUSTFLAGS 없이 다시)",
        &run(&dir, &[], &["check"]),
    );
    // 출력: 종료 코드 0, 다시 컴파일 아니요
    // 출력: | warning: unused variable: `unused`
    // cargo 1.100.0은 RUSTFLAGS 값마다 산출물을 따로 두므로(빌드 디렉터리에 해시가 다른
    // 디렉터리가 둘 생김) 되돌릴 때는 다시 컴파일하지 않았습니다. 디스크는 두 벌을 씁니다.

    println!("3. cargo clippy -- -D warnings: Clippy에만 걸리고 인자가 바뀌면 다시 검사");
    show("cargo clippy", &run(&dir, &[], &["clippy"]));
    // 출력: 종료 코드 0, 다시 컴파일 예
    // 출력: | warning: unused variable: `unused`
    show(
        "cargo clippy -- -D warnings",
        &run(&dir, &[], &["clippy", "--", "-D", "warnings"]),
    );
    // 출력: 종료 코드 101, 다시 컴파일 예
    // 출력: | error: unused variable: `unused`
    show(
        "cargo clippy (인자 없이 다시)",
        &run(&dir, &[], &["clippy"]),
    );
    // 출력: 종료 코드 0, 다시 컴파일 예
    // 출력: | warning: unused variable: `unused`
    // `--` 뒤의 인자는 Clippy가 받는 값이라, 바뀔 때마다 그 크레이트를 다시 검사합니다.
    // 로컬에서 `cargo clippy`, CI 스크립트에서 `cargo clippy -- -D warnings`를 번갈아
    // 돌리면 매번 다시 검사하게 됩니다.

    println!("4. 권장(Cargo 1.97+): build.warnings = \"deny\"");
    let deny = [("CARGO_BUILD_WARNINGS", "deny")];
    show(
        "CARGO_BUILD_WARNINGS=deny cargo check",
        &run(&dir, &deny, &["check"]),
    );
    // 출력: 종료 코드 101, 다시 컴파일 아니요
    // 출력: | warning: unused variable: `unused`
    // 출력: | error: warnings are denied by `build.warnings` configuration
    show(
        "CARGO_BUILD_WARNINGS=deny cargo clippy",
        &run(&dir, &deny, &["clippy"]),
    );
    // 출력: 종료 코드 101, 다시 컴파일 아니요
    // 출력: | warning: unused variable: `unused`
    // 출력: | error: warnings are denied by `build.warnings` configuration
    // 경고 수준은 컴파일 결과가 아니라 Cargo가 결과를 판정하는 방식이므로,
    // 저장해 둔 결과를 그대로 쓰고 판정만 바꿉니다.

    fs::remove_dir_all(&dir).expect("임시 디렉터리를 지우지 못했습니다");
}
