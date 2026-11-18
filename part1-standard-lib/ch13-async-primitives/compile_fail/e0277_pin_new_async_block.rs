// 기대하는 에러: E0277
// 핵심 메시지: `{async block@...}` cannot be unpinned
//
// Pin::new는 가리키는 값이 Unpin일 때만 쓸 수 있습니다. async 블록이 만든 퓨처는
// 자기 참조를 가질 수 있어 Unpin이 아니므로 Pin::new로 고정할 수 없습니다.
//
// 확인: rustc --edition 2024 compile_fail/e0277_pin_new_async_block.rs --out-dir /tmp/rust-p1-ch13

use std::pin::Pin;
use std::task::{Context, Waker};

fn main() {
    let mut cx = Context::from_waker(Waker::noop());
    let mut future = async { 1 + 2 };
    let result = Pin::new(&mut future).poll(&mut cx); // Unpin이 아닌 값에 Pin::new → E0277
    println!("{result:?}");
}
