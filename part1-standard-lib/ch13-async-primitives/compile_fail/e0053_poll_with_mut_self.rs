// 기대하는 에러: E0053
// 핵심 메시지: method `poll` has an incompatible type for trait
//              expected `Pin<&mut Countdown>`, found `&mut Countdown`
//
// Future::poll을 구현하면서 self를 평범한 &mut self로 적었습니다. 트레이트가 정한
// self: Pin<&mut Self>와 시그니처가 달라 구현으로 인정되지 않습니다.
//
// 확인: rustc --edition 2024 compile_fail/e0053_poll_with_mut_self.rs --out-dir /tmp/rust-p1-ch13

use std::task::{Context, Poll};

struct Countdown {
    remaining: u32,
}

impl Future for Countdown {
    type Output = ();

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        // self의 타입이 트레이트와 다름 → E0053
        if self.remaining == 0 {
            return Poll::Ready(());
        }
        self.remaining -= 1;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

fn main() {
    let _countdown = Countdown { remaining: 3 };
}
