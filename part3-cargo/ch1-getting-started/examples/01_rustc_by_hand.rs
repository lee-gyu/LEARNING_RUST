//! Cargo 없이 `rustc`만으로 "라이브러리 + 실행 파일"을 빌드해 봅니다.
//!
//! Cargo가 대신 해 주는 일(컴파일 순서 정하기, 크레이트끼리 이어 주기)을
//! 손으로 해 보면 Cargo가 왜 필요한지 보입니다.
//!
//! 실행: cargo run -p p3-ch1-getting-started --example 01_rustc_by_hand
//!
//! 이 예제는 `rustc`를 자식 프로세스로 세 번 실행하고(Part 1 ch10의 `Command`),
//! 임시 디렉터리에 만든 파일은 끝날 때 지웁니다. 1초 안팎이 걸립니다.

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

// 다른 크레이트가 가져다 쓸 라이브러리의 소스입니다.
const GREET_RS: &str = r#"pub fn hello(name: &str) -> String {
    format!("안녕하세요, {name}!")
}
"#;

// 위 라이브러리를 쓰는 실행 파일의 소스입니다.
const APP_RS: &str = r#"use greet::hello;

fn main() {
    println!("{}", hello("Cargo 없이 빌드한 프로그램"));
}
"#;

/// 임시 디렉터리에서 `rustc`를 실행하고, 성공 여부와 표준 에러의 첫 줄을 돌려줍니다.
fn rustc(dir: &Path, args: &[&str]) -> io::Result<(bool, String)> {
    println!("$ rustc {}", args.join(" "));
    // `cargo run`으로 실행하면 이 `rustc`는 Cargo가 쓰는 것과 같은 툴체인입니다.
    let output = Command::new("rustc").args(args).current_dir(dir).output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_line = stderr.lines().next().unwrap_or("").to_string();
    Ok((output.status.success(), first_line))
}

fn main() -> io::Result<()> {
    let dir = env::temp_dir().join(format!("p3-ch1-by-hand-{}", std::process::id()));
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("greet.rs"), GREET_RS)?;
    fs::write(dir.join("app.rs"), APP_RS)?;

    println!("== 1. 실행 파일만 컴파일해 보기 ==");
    // `rustc`는 `greet`라는 크레이트가 어디 있는지 모릅니다. 알려 주지 않으면 찾지 않습니다.
    let (ok, message) = rustc(&dir, &["--edition", "2024", "app.rs"])?;
    println!("성공: {ok}");
    println!("{message}");
    // 출력: 성공: false
    // 출력: error[E0432]: unresolved import `greet`

    println!();
    println!("== 2. 라이브러리를 먼저 컴파일 ==");
    // 의존하는 쪽보다 의존받는 쪽을 먼저 컴파일해야 합니다. 이 순서를 사람이 정했습니다.
    let (ok, _) = rustc(
        &dir,
        &["--edition", "2024", "--crate-type", "lib", "greet.rs"],
    )?;
    println!("성공: {ok}");
    println!("libgreet.rlib 생성: {}", dir.join("libgreet.rlib").exists());
    // 출력: 성공: true
    // 출력: libgreet.rlib 생성: true

    println!();
    println!("== 3. 라이브러리의 위치를 알려 주고 실행 파일 컴파일 ==");
    // `--extern 이름=경로`가 "`greet`라는 크레이트는 이 파일"이라고 알려 주는 옵션입니다.
    let (ok, _) = rustc(
        &dir,
        &[
            "--edition",
            "2024",
            "app.rs",
            "--extern",
            "greet=libgreet.rlib",
        ],
    )?;
    println!("성공: {ok}");
    // 출력: 성공: true

    println!();
    println!("== 4. 실행 ==");
    let app = dir.join(format!("app{}", env::consts::EXE_SUFFIX));
    let output = Command::new(&app).output()?;
    println!("$ ./app");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    // 출력: 안녕하세요, Cargo 없이 빌드한 프로그램!

    fs::remove_dir_all(&dir)?;

    println!();
    println!("크레이트 2개에 명령 2개와 순서 1가지를 손으로 챙겼습니다.");
    println!("Cargo는 Cargo.toml과 디렉터리 관례를 읽어 이 명령들을 대신 만듭니다.");
    Ok(())
}
