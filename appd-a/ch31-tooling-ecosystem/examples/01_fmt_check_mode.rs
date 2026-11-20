//! 지침 1: Cargo에 있는 도구를 쓰되, CI에는 고치지 않고 검사만 하는 모드를 둡니다.
//!
//! 임시 디렉터리에 서식이 어긋난 작은 패키지를 만들고, 같은 `cargo fmt`를
//! 고치는 모드(`cargo fmt`)와 검사 모드(`cargo fmt --check`)로 실행해 출력,
//! 종료 코드, 파일이 바뀌었는지를 비교합니다. `cargo fmt`는 빌드하지 않고
//! rustfmt만 부르므로 몇 초 안에 끝납니다.
//!
//! 실행: cargo run -p er-ch31-tooling-ecosystem --example 01_fmt_check_mode

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;

/// rustfmt가 고칠 곳이 두 군데 있는 소스입니다(공백, 한 줄로 쓴 함수).
const MESSY: &str = "pub fn total(xs:&[u32])->u32{\n    xs.iter().sum()\n}\n\npub fn  label( n:u32 )->String { format!(\"{n}개\") }\n";

/// 한 번 실행한 결과입니다.
struct Run {
    code: i32,
    stdout: String,
}

/// `cargo run`은 자기 경로를 `CARGO` 환경 변수로 넘겨 줍니다. 그 cargo를 다시 부르면
/// 지금 쓰는 툴체인과 같은 cargo가 실행됩니다. 직접 실행했다면 PATH의 cargo를 씁니다.
fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

fn cargo_fmt(dir: &Path, check: bool) -> Run {
    let mut cmd = Command::new(cargo());
    cmd.arg("fmt").current_dir(dir);
    if check {
        cmd.arg("--check");
    }
    let out = cmd.output().expect("cargo fmt를 실행하지 못했습니다");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
    }
}

/// rustfmt의 diff 머리줄에는 임시 디렉터리의 절대 경로가 들어 있습니다.
/// 실행마다 달라지는 그 부분을 `<임시>`로 바꿔서 출력합니다.
/// diff의 빈 문맥 줄은 공백 하나로 시작하므로, 줄 끝 공백은 지우고 출력합니다.
fn show_diff(stdout: &str, dir_name: &str) {
    for line in stdout.lines() {
        let shown = match line.find(dir_name) {
            Some(i) => format!("    Diff in <임시>{}", &line[i + dir_name.len()..]),
            None => format!("    {line}"),
        };
        println!("{}", shown.trim_end());
    }
}

fn main() {
    let dir_name = format!("er-ch31-fmt-{}", std::process::id());
    let dir = std::env::temp_dir().join(&dir_name);
    fs::create_dir_all(dir.join("src")).expect("임시 디렉터리를 만들지 못했습니다");
    // 빈 [workspace]로 이 패키지를 혼자인 워크스페이스로 만듭니다.
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"fmt-demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .expect("Cargo.toml을 쓰지 못했습니다");
    let lib = dir.join("src/lib.rs");
    let reset = || fs::write(&lib, MESSY).expect("src/lib.rs를 쓰지 못했습니다");
    let changed = || fs::read_to_string(&lib).expect("src/lib.rs를 읽지 못했습니다") != MESSY;

    println!("1. 비권장: CI에서 고치는 모드(cargo fmt)를 실행");
    reset();
    let run = cargo_fmt(&dir, false);
    let lines = run.stdout.lines().count();
    println!("  종료 코드 {}, 출력 {lines}줄", run.code);
    // 출력: 종료 코드 0, 출력 0줄
    println!("  파일이 바뀜: {}", changed());
    // 출력: 파일이 바뀜: true
    // CI는 작업이 끝나면 고친 사본을 버리므로, 저장소의 코드는 어긋난 채로 남습니다.
    println!("  → CI의 사본만 고쳐지고 작업은 성공으로 끝남");

    println!("2. 권장: CI에서는 검사 모드(cargo fmt --check)");
    reset();
    let run = cargo_fmt(&dir, true);
    println!("  종료 코드 {}", run.code);
    // 출력: 종료 코드 1
    show_diff(&run.stdout, &dir_name);
    println!("  파일이 바뀜: {}", changed());
    // 출력: 파일이 바뀜: false

    println!("3. 로컬(또는 편집기)에서 고친 뒤 다시 검사");
    let fix = cargo_fmt(&dir, false);
    let check = cargo_fmt(&dir, true);
    println!(
        "  cargo fmt 종료 코드 {}, cargo fmt --check 종료 코드 {}",
        fix.code, check.code
    );
    // 출력: cargo fmt 종료 코드 0, cargo fmt --check 종료 코드 0

    fs::remove_dir_all(&dir).expect("임시 디렉터리를 지우지 못했습니다");
}
