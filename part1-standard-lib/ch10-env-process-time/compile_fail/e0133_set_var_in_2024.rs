// 기대하는 에러: E0133 — call to unsafe function `set_var` is unsafe and requires unsafe block
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0133_set_var_in_2024.rs --out-dir /tmp/rust-p1-ch10
//
// 자식 프로세스에 환경 변수를 넘기려고 내 프로세스의 환경을 먼저 바꿨습니다.
// std::env::set_var는 2021 에디션까지는 안전한 함수였지만 2024 에디션부터 unsafe 함수입니다.
// 유닉스 계열에서는 다른 스레드가 환경을 읽는 도중에 바꾸면 미정의 동작이 될 수 있는데,
// 어떤 코드가 환경을 읽는지(C 라이브러리 포함) 프로그램이 다 알 수 없기 때문입니다.
// 같은 코드를 --edition 2021로 컴파일하면 에러 없이 통과합니다.
// 해결: 자기 환경을 바꾸지 말고 자식 몫만 정합니다. Command::new(..).env("P1CH10_COLOR", "blue")
//       (examples/03_command.rs의 2번 구역)

use std::env;
use std::process::Command;

fn main() {
    env::set_var("P1CH10_COLOR", "blue");
    let status = Command::new("printenv").arg("P1CH10_COLOR").status().expect("printenv 실행");
    println!("{status}");
}
