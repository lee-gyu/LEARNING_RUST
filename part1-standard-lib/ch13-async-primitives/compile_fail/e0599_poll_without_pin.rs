// 기대하는 에러: E0599
// 핵심 메시지: no method named `poll` found for `async` block `{async block@...}` in the current scope
//
// async 블록이 만든 퓨처에 poll을 바로 부르려 합니다. poll의 self는 Pin<&mut Self>라서
// 고정하지 않은 퓨처에서는 메서드 자체를 찾지 못합니다.
//
// 확인: rustc --edition 2024 compile_fail/e0599_poll_without_pin.rs --out-dir /tmp/rust-p1-ch13

use std::task::{Context, Waker};

fn main() {
    let mut cx = Context::from_waker(Waker::noop());
    let mut future = async { 1 + 2 };
    let result = future.poll(&mut cx); // 고정하지 않은 퓨처에 poll → E0599
    println!("{result:?}");
}
