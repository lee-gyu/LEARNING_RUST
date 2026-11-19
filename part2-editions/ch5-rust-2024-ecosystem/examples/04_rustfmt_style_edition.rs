// 04_rustfmt_style_edition: rustfmt의 에디션(edition)과 스타일 에디션(style_edition)
//
// rustfmt도 에디션을 봅니다. 쓰임새가 두 가지입니다.
//   - edition: 소스를 어느 에디션의 문법으로 읽을지(파싱). 2015에서는 async가 이름이고,
//     let 체인은 2024에서만 읽힙니다.
//   - style_edition: 읽은 코드를 어느 판의 스타일 가이드로 배치할지(서식). 2024 스타일은
//     use 목록을 버전 정렬하고, raw 식별자를 r# 없이 정렬하고, 여러 서식 버그를 고쳤습니다.
// style_edition을 따로 정하지 않으면 edition과 같은 값을 씁니다.
//
// 이 예제는 rustfmt를 자식 프로세스로 실행합니다. 코드 조각을 표준 입력으로 넘기고 서식이 적용된
// 결과를 표준 출력으로 받습니다. 1번 구역은 같은 입력을 두 스타일 에디션으로 서식해 비교하고,
// 2번 구역은 옵션과 rustfmt.toml에 따라 rustfmt가 읽는 에디션과 스타일이 어떻게 정해지는지 봅니다.
//
// 준비: rustup으로 설치했다면 rustfmt가 이미 있습니다(없으면 rustup component add rustfmt).
//       환경 변수 RUSTFMT가 있으면 그 경로의 rustfmt를 씁니다.
// 실행: cargo run -p p2-ch5-rust-2024-ecosystem --example 04_rustfmt_style_edition

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// 1번 구역의 입력들: (제목, 서식을 적용하기 전의 코드)
const SAMPLES: [(&str, &str); 3] = [
    (
        "버전 정렬: 숫자는 값의 크기로, 대문자 이름과 소문자 이름을 섞어서",
        "use std::num::{NonZeroU64, NonZeroU8, NonZeroU16};
use std::io::{Write, self, stdout};
",
    ),
    (
        "raw 식별자 정렬: r#async를 async로 보고 정렬",
        "use jobs::queue::Queue;
use jobs::r#async::spawn;
use jobs::blocking::run;
",
    ),
    (
        "서식 수정: 뒤따르는 주석, loop 하나뿐인 클로저, 짧은 if-else, 중첩 튜플 접근",
        "fn main() {
    let total = 10; // 합계
                    // 다음 줄은 합계와 상관없는 주석
    std::thread::spawn(|| loop {
        println!(\"작업\");
    });
    let size = || {
        if total > 5 {
            \"큼\"
        } else {
            \"작음\"
        }
    };
    let pair = ((1, 2), 3);
    let _ = (size(), pair.0 .0);
}
",
    ),
];

/// 2번 구역의 입력: 2015 에디션에서만 읽히는 이름(async)과, 스타일에 따라 순서가 바뀌는 use
const OLD_CODE: &str = "use std::num::{NonZeroU16, NonZeroU8};
fn main() {
    let async = NonZeroU8::new(1);
    let _ = (async, NonZeroU16::new(2));
}
";

fn main() {
    let rustfmt = env::var_os("RUSTFMT").unwrap_or_else(|| "rustfmt".into());
    // 결과가 이 컴퓨터의 다른 rustfmt 설정에 흔들리지 않도록, 임시 디렉터리에 설정 파일을 직접
    // 만들고 그 디렉터리에서 rustfmt를 실행합니다. rustfmt는 현재 디렉터리부터 위로 올라가며
    // rustfmt.toml을 찾고, 처음 찾은 파일을 씁니다.
    let base = env::temp_dir().join(format!("rust-p2-ch5-fmt-{}", std::process::id()));
    let plain = make_dir(&base, "plain", ""); // 빈 설정 = 모든 값이 기본값
    let style_2024 = make_dir(&base, "style2024", "style_edition = \"2024\"\n");
    let edition_2024 = make_dir(&base, "edition2024", "edition = \"2024\"\n");

    let Some(version) = run(&rustfmt, &plain, &["--version"], "") else {
        println!("rustfmt를 실행하지 못했습니다. rustup component add rustfmt로 설치합니다.");
        let _ = fs::remove_dir_all(&base);
        return;
    };
    println!("사용한 rustfmt: {}", version.unwrap_or_else(|e| e).trim());

    println!("1. 같은 입력, 다른 스타일 에디션 (--edition 2024 --style-edition 2021 / 2024)");
    for (title, code) in SAMPLES {
        println!("[{title}]");
        for style in ["2021", "2024"] {
            let args = ["--edition", "2024", "--style-edition", style];
            println!("  ({style} 스타일)");
            print_indented(run(&rustfmt, &plain, &args, code));
        }
    }

    println!("2. 무엇이 에디션과 스타일 에디션을 정하는가 (입력: let async = ...; 와 use 한 줄)");
    let cases: [(&str, &Path, &[&str]); 5] = [
        ("rustfmt", &plain, &[]),
        (
            "rustfmt --style-edition 2024",
            &plain,
            &["--style-edition", "2024"],
        ),
        ("rustfmt --edition 2024", &plain, &["--edition", "2024"]),
        ("rustfmt.toml: style_edition = \"2024\"", &style_2024, &[]),
        ("rustfmt.toml: edition = \"2024\"", &edition_2024, &[]),
    ];
    for (label, dir, args) in cases {
        let summary = match run(&rustfmt, dir, args, OLD_CODE) {
            Some(Ok(out)) => format!("읽기 성공 → {}", out.lines().next().unwrap_or("")),
            Some(Err(err)) => format!("읽기 실패 → {}", first_error_line(&err)),
            None => "실행 실패".to_string(),
        };
        println!("  {label:<38} {summary}");
    }

    let _ = fs::remove_dir_all(&base);
}

/// base 아래에 하위 디렉터리를 만들고 rustfmt.toml에 config를 씁니다.
fn make_dir(base: &Path, name: &str, config: &str) -> PathBuf {
    let dir = base.join(name);
    fs::create_dir_all(&dir).expect("임시 디렉터리 만들기");
    fs::write(dir.join("rustfmt.toml"), config).expect("rustfmt.toml 쓰기");
    dir
}

/// rustfmt를 dir에서 실행하고 input을 표준 입력으로 넘깁니다.
/// 실행 자체가 안 되면 None, 성공하면 Some(Ok(표준 출력)), 실패하면 Some(Err(표준 에러))입니다.
fn run(
    rustfmt: &std::ffi::OsStr,
    dir: &Path,
    args: &[&str],
    input: &str,
) -> Option<Result<String, String>> {
    let mut child = Command::new(rustfmt)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    // 입력이 작아서 다 쓴 뒤에 출력을 읽어도 막히지 않습니다. 쓰고 나서 stdin을 닫아야(drop)
    // rustfmt가 입력의 끝을 알고 일을 시작합니다.
    let mut stdin = child.stdin.take()?;
    stdin.write_all(input.as_bytes()).ok()?;
    drop(stdin);
    let output = child.wait_with_output().ok()?;
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
    if output.status.success() {
        Some(Ok(text(&output.stdout)))
    } else {
        Some(Err(text(&output.stderr)))
    }
}

fn print_indented(result: Option<Result<String, String>>) {
    let text = match result {
        Some(Ok(out)) => out,
        Some(Err(err)) => format!("(에러) {}", first_error_line(&err)),
        None => "(실행 실패)".to_string(),
    };
    for line in text.lines() {
        println!("    {line}");
    }
}

/// rustfmt의 에러 출력에서 "error: ..." 첫 줄만 꺼냅니다.
fn first_error_line(stderr: &str) -> &str {
    stderr
        .lines()
        .find(|line| line.starts_with("error"))
        .unwrap_or("(메시지 없음)")
}
