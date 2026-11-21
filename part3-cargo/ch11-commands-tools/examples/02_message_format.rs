//! `cargo check --message-format=json`의 출력을 읽어 봅니다. 한 줄에 JSON 객체가 하나씩 나옵니다.
//!
//! 실행: cargo run -q -p p3-ch11-commands-tools --example 02_message_format
//!
//! 임시 디렉터리에 작은 패키지(경고가 하나 나는 라이브러리)를 만들고, 그 패키지를 자식 프로세스의 cargo로
//! 검사합니다. 예제를 짧게 하려고 JSON 파서 크레이트 없이 문자열 검색으로 값을 꺼냅니다.
//! 실제 도구는 JSON을 파싱해 읽어야 합니다(projects/cargo-hello는 serde_json을 씁니다).

use std::fs;
use std::process::Command;

/// 한 줄의 JSON에서 `"key":"값"`의 값을 문자열 검색으로 꺼냅니다(이 예제의 단순한 출력에만 맞는 방법).
fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\":\"");
    let start = line.find(&pat)? + pat.len();
    let end = line[start..].find('"')? + start;
    Some(&line[start..end])
}

fn main() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let dir = std::env::temp_dir().join(format!("p3-ch11-msgfmt-{}", std::process::id()));
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"msgdemo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .unwrap();
    // 쓰지 않는 변수 하나: 경고가 하나 나도록
    fs::write(
        dir.join("src/lib.rs"),
        "pub fn f() {\n    let unused = 1;\n}\n",
    )
    .unwrap();

    let out = Command::new(&cargo)
        .args(["check", "--message-format=json", "--manifest-path"])
        .arg(dir.join("Cargo.toml"))
        .output()
        .expect("cargo check 실행");

    // 표준 출력의 각 줄이 JSON 메시지 하나입니다. "reason"이 메시지의 종류입니다.
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let reason = field(line, "reason").unwrap_or("?");
        match reason {
            "compiler-message" => {
                // "rendered"는 사람이 읽는 진단 전체입니다. 줄바꿈이 `\n`으로 들어 있어 첫 줄만 씁니다.
                // ("level"이나 "message"는 children 안에도 있어 이런 문자열 검색으로는 잘못 꺼내기 쉽습니다.)
                let rendered = field(line, "rendered").unwrap_or("?");
                let first = rendered.split("\\n").next().unwrap_or(rendered);
                println!("{reason}: {first}");
            }
            "compiler-artifact" => {
                let name = field(line, "name").unwrap_or("?");
                let fresh = line.contains("\"fresh\":true");
                println!("{reason}: {name} (fresh: {fresh})");
            }
            "build-finished" => {
                println!("{reason}: 성공 {}", line.contains("\"success\":true"));
            }
            other => println!("{other}"),
        }
    }
    // 사람이 읽는 진행 메시지(Checking, Finished)는 그대로 표준 에러로 나갑니다.
    let stderr = String::from_utf8_lossy(&out.stderr);
    println!("표준 에러의 줄 수: {}", stderr.lines().count());

    fs::remove_dir_all(&dir).unwrap();

    // 출력: compiler-message: warning: unused variable: `unused`
    // 출력: compiler-artifact: msgdemo (fresh: false)
    // 출력: build-finished: 성공 true
    // 출력: 표준 에러의 줄 수: 2
}
