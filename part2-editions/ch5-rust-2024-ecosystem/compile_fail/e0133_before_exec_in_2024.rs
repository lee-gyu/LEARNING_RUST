// 기대하는 에러: E0133 (함께 deprecated 경고)
// 핵심 메시지: call to unsafe function `before_exec` is unsafe and requires unsafe block
// 에디션: 2024에서 실패 / 2015·2018·2021에서 통과(deprecated 경고와 함께)
// 플랫폼: 유닉스 계열 전용(std::os::unix). Windows에서는 다른 이유(E0433)로 컴파일되지 않습니다.
//
// CommandExt::before_exec는 자식 프로세스가 새 프로그램을 exec하기 직전에 클로저를 실행합니다.
// 이 클로저가 지켜야 할 조건(비동기 시그널 안전)을 컴파일러가 검사할 수 없어서, 1.37.0에서 같은 일을
// 하는 unsafe 함수 pre_exec가 생기고 before_exec는 deprecated되었습니다. 2021 에디션까지는
// deprecated 경고만 났고, 2024 에디션부터는 unsafe 함수로도 다룹니다.
// 해결: pre_exec로 바꾸고 unsafe 블록과 SAFETY 주석을 붙입니다(examples/03_env_unsafe.rs의 4번 구역).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0133_before_exec_in_2024.rs --out-dir /tmp/rust-p2-ch5   → 에러
//   rustc --edition 2021 compile_fail/e0133_before_exec_in_2024.rs --out-dir /tmp/rust-p2-ch5   → 통과(경고)

use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() {
    let status = Command::new("true")
        .before_exec(|| Ok(())) // → 2024 에디션에서 E0133
        .status()
        .expect("true 실행");
    println!("{}", status);
}
