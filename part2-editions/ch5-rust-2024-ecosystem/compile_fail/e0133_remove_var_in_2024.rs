// 기대하는 에러: E0133
// 핵심 메시지: call to unsafe function `remove_var` is unsafe and requires unsafe block
// 에디션: 2024에서 실패 / 2015·2018·2021에서 통과(경고도 없음)
//
// 비밀 값이 자식 프로세스에 새지 않도록, 자식을 만들기 전에 내 환경에서 지웠습니다.
// env::remove_var는 env::set_var와 함께 2024 에디션부터 unsafe 함수로 부릅니다(Part 1 ch10 2.4절).
// 2021 에디션에서는 같은 호출이 경고 없이 컴파일됩니다. deprecated_safe_2024 린트의 기본 수준이
// allow이기 때문입니다(rustc --edition 2021 -W deprecated-safe-2024로 켜면 경고가 나옵니다).
// 해결: 자식에게 주지 않으려는 것이라면 내 환경은 그대로 두고 Command::env_remove로 자식 몫만 뺍니다.
//       내 환경을 꼭 바꿔야 한다면 스레드를 만들기 전에 unsafe 블록 안에서, SAFETY 주석과 함께 부릅니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0133_remove_var_in_2024.rs --out-dir /tmp/rust-p2-ch5   → 에러
//   rustc --edition 2021 compile_fail/e0133_remove_var_in_2024.rs --out-dir /tmp/rust-p2-ch5   → 통과

use std::env;
use std::process::Command;

fn main() {
    env::remove_var("P2CH5_TOKEN"); // → 2024 에디션에서 E0133
    let status = Command::new("printenv")
        .arg("P2CH5_TOKEN")
        .status()
        .expect("printenv 실행");
    println!("{}", status);
}
