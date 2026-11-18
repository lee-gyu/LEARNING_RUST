// 기대하는 에러: E0308 — mismatched types: expected `&str`, found `&Vec<u8>`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0308_stdout_bytes_as_str.rs --out-dir /tmp/rust-p1-ch10
//
// 자식의 표준 출력을 문자열처럼 다루려고 했습니다.
// Output의 stdout 필드는 Vec<u8>, 즉 바이트입니다. 프로세스의 출력은 아무 바이트나 될 수 있고
// UTF-8이라는 보장이 없으므로, 표준 라이브러리는 문자열로 바꾸는 일을 대신 해 주지 않습니다.
// 해결: String::from_utf8(output.stdout)? (UTF-8이 아니면 에러)
//       또는 String::from_utf8_lossy(&output.stdout) (깨진 바이트는 U+FFFD로 바꿈)
//       (examples/03_command.rs의 stdout_of 함수)

use std::process::Command;

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

fn main() {
    let output = Command::new("echo").arg("hello process world").output().expect("echo 실행");
    let words = count_words(&output.stdout);
    println!("단어 {words}개");
}
