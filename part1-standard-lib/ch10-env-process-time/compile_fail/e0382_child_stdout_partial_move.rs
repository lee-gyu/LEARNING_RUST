// 기대하는 에러: E0382 — borrow of partially moved value: `child`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0382_child_stdout_partial_move.rs --out-dir /tmp/rust-p1-ch10
//
// 자식의 표준 출력을 읽은 뒤 wait로 거둬들이려 했습니다.
// child.stdout은 Option<ChildStdout> 필드입니다. child.stdout.unwrap()은 그 필드를 child 밖으로 옮기므로(부분 이동)
// child는 일부가 빠진 값이 되어, &mut self가 필요한 wait를 부를 수 없습니다.
// 해결: child.stdout.take().unwrap()으로 꺼냅니다. take()는 필드 자리에 None을 남기므로 child는 온전합니다.
//       (examples/04_pipes.rs의 1·4번 구역)

use std::io::Read;
use std::process::{Command, Stdio};

fn main() {
    let mut child = Command::new("echo")
        .arg("hello")
        .stdout(Stdio::piped())
        .spawn()
        .expect("echo 실행");
    let mut stdout = child.stdout.unwrap();
    let mut text = String::new();
    stdout.read_to_string(&mut text).expect("읽기");
    let status = child.wait().expect("기다리기");
    println!("{text:?}, {status}");
}
