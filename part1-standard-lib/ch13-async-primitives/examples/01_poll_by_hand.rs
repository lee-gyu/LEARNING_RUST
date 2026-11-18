// Future 트레이트 해부: 실행기 없이 poll을 손으로 불러 퓨처를 한 단계씩 진행시킵니다.
// Waker::noop()으로 Context를 만들고, 퓨처를 고정(Pin)한 뒤 poll의 결과(Poll)를 직접 봅니다.
//
// 실행: cargo run -p p1-ch13-async-primitives --example 01_poll_by_hand
//
// 4번 구역은 끝난 퓨처를 일부러 다시 폴링해 패닉을 일으키고 catch_unwind로 붙잡습니다.
// 그동안 패닉 훅을 아무것도 출력하지 않는 훅으로 바꿔 두므로 표준 에러에는 아무것도 나오지 않습니다.

use std::future;
use std::panic::{self, AssertUnwindSafe};
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

fn main() {
    // poll에는 Context가 필요하고, Context는 웨이커(Waker)를 담습니다. 웨이커는 "준비되면 다시
    // 폴링해 달라"고 실행기에 알리는 연락처입니다. 이 예제는 우리가 직접 poll을 부르므로
    // 연락을 받아도 아무 일도 하지 않는 Waker::noop()이면 충분합니다(1.85.0).
    let mut cx = Context::from_waker(Waker::noop());

    println!("1. std::future::ready: 첫 poll에서 바로 Ready");
    let mut ready = future::ready(42);
    // poll은 self: Pin<&mut Self>를 받습니다. Ready<i32>는 Unpin이라서 Pin::new로 감쌀 수 있습니다.
    let first = Pin::new(&mut ready).poll(&mut cx);
    println!("    첫 poll: {first:?}"); // 출력: Ready(42)
    println!(
        "    is_ready: {}, map(|v| v * 2): {:?}",
        first.is_ready(),
        first.map(|v| v * 2)
    );
    // 출력: is_ready: true, map(|v| v * 2): Ready(84)

    println!("2. std::future::pending: 몇 번을 폴링해도 Pending");
    let mut never = future::pending::<i32>();
    for i in 1..=3 {
        println!("    {i}번째 poll: {:?}", Pin::new(&mut never).poll(&mut cx)); // 출력: Pending
    }

    println!("3. async 블록: 본문은 만들 때가 아니라 첫 poll에서 실행됩니다");
    let sum = async {
        println!("    [본문] 실행 시작");
        let a = future::ready(1).await; // 안쪽 퓨처가 바로 Ready라서 여기서 멈추지 않습니다.
        let b = future::ready(2).await;
        println!("    [본문] a + b를 돌려줍니다");
        a + b
    };
    println!("    퓨처를 만들었습니다 (본문은 아직 실행되지 않음)");
    // async 블록이 만든 퓨처는 Unpin이 아니므로 pin!으로 이 자리에 고정합니다(README 2.7절).
    let mut sum = pin!(sum);
    println!("    첫 poll: {:?}", sum.as_mut().poll(&mut cx)); // 출력: Ready(3)

    // await 지점에서 Pending을 받으면 퓨처는 "어디까지 했는지"를 기억한 채 돌아옵니다.
    // 다시 폴링해도 본문을 처음부터 실행하지 않고 멈춘 await 지점에서 이어 갑니다.
    let mut stuck = pin!(async {
        println!("    [본문] await 전");
        future::pending::<()>().await;
        println!("    [본문] await 뒤 (실행되지 않습니다)");
    });
    for i in 1..=3 {
        println!("    stuck {i}번째 poll: {:?}", stuck.as_mut().poll(&mut cx));
    }
    // 출력: [본문] await 전 (한 번만), 그 뒤 Pending 세 번

    println!("4. 끝난 퓨처를 다시 폴링하면 안 됩니다");
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {})); // 패닉 메시지를 표준 에러에 찍지 않습니다.

    let mut done = pin!(async { 7 });
    println!("    async 블록 첫 poll: {:?}", done.as_mut().poll(&mut cx)); // 출력: Ready(7)
    let again = panic::catch_unwind(AssertUnwindSafe(|| done.as_mut().poll(&mut cx)));
    println!("    다시 poll: {}", describe(again));
    // 출력: 다시 poll: 패닉 — `async fn` resumed after completion

    let again = panic::catch_unwind(AssertUnwindSafe(|| Pin::new(&mut ready).poll(&mut cx)));
    println!("    ready(42)를 다시 poll: {}", describe(again));
    // 출력: ready(42)를 다시 poll: 패닉 — `Ready` polled after completion

    panic::set_hook(default_hook);
}

/// catch_unwind의 결과를 한 줄로 설명합니다. 패닉 페이로드는 패닉을 일으킨 방식에 따라
/// &str(async 블록)일 수도 있고 String(Ready 안의 expect)일 수도 있어 둘 다 확인합니다.
fn describe<T: std::fmt::Debug>(result: std::thread::Result<Poll<T>>) -> String {
    match result {
        Ok(poll) => format!("{poll:?}"),
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_default();
            format!("패닉 — {message}")
        }
    }
}
