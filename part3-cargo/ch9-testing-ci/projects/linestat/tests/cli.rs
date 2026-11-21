//! 바이너리를 실제로 실행해 보는 통합 테스트입니다.

use std::io::Write;
use std::process::{Command, Stdio};

fn run(input: &str, args: &[&str]) -> String {
    // CARGO_BIN_EXE_<이름>: 통합 테스트를 빌드할 때 Cargo가 알려 주는 실행 파일 경로
    let mut child = Command::new(env!("CARGO_BIN_EXE_linestat"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
}

#[test]
fn prints_counts() {
    assert_eq!(run("a b\nc\n", &[]), "줄 2 / 단어 3 / 글자 6\n");
}

#[cfg(feature = "top")]
#[test]
fn prints_top_words() {
    let out = run("x y x\n", &["--top", "1"]);
    assert!(out.ends_with("  2 x\n"), "{out:?}");
}
