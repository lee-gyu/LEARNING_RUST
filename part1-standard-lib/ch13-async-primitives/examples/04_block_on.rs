// 최소 실행기 block_on: 퓨처 하나를 현재 스레드에서 끝까지 실행합니다.
// 웨이커(Wake 구현)는 잠든 스레드를 thread::unpark로 깨우고, 실행기는 Pending을 받으면
// thread::park로 잠듭니다. 깨우지 않으면 멈추고, 쉬지 않고 깨우면 바쁜 폴링이 된다는 것을
// 폴링 횟수로 확인합니다.
//
// 실행: cargo run -p p1-ch13-async-primitives --example 04_block_on
//
// 3번 구역은 일부러 50ms를 기다린 뒤 포기하고, 4번 구역은 30ms 동안 바쁜 폴링을 합니다.
// 4번 구역의 실제 폴링 횟수는 컴퓨터와 실행마다 다르므로 "1000번을 넘는가"만 출력합니다.

use std::pin::{Pin, pin};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 실행기
// ---------------------------------------------------------------------------

/// block_on을 부른 스레드를 깨우는 웨이커.
struct ThreadWaker {
    thread: Thread,
    // "깨워 달라는 연락이 왔다"는 표시. park는 아무도 unpark하지 않았는데 돌아올 수도 있으므로
    // (문서가 허용하는 가짜 깨어남, 또는 다른 코드가 부른 unpark) 이 표시로 진짜 연락인지 확인합니다.
    woken: AtomicBool,
}

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.woken.store(true, Ordering::SeqCst);
        self.thread.unpark();
    }
}

fn thread_waker() -> (Arc<ThreadWaker>, Waker) {
    let inner = Arc::new(ThreadWaker {
        thread: thread::current(),
        woken: AtomicBool::new(false),
    });
    let waker = Waker::from(Arc::clone(&inner));
    (inner, waker)
}

/// 퓨처가 끝날 때까지 현재 스레드를 블로킹하며 실행합니다.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future); // poll하려면 고정해야 합니다. 이 함수의 스택에 고정합니다.
    let (inner, waker) = thread_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                // 연락이 올 때까지 잠듭니다. poll 도중에 이미 연락이 왔다면(스스로 깨운 퓨처)
                // swap이 true를 돌려주므로 잠들지 않고 바로 다시 폴링합니다.
                while !inner.woken.swap(false, Ordering::SeqCst) {
                    thread::park();
                }
            }
        }
    }
}

/// block_on과 같지만 연락 없이 limit가 지나면 포기하고 None을 돌려줍니다(관찰용).
fn block_on_for<F: Future>(future: F, limit: Duration) -> Option<F::Output> {
    let mut future = pin!(future);
    let (inner, waker) = thread_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return Some(output);
        }
        let deadline = Instant::now() + limit;
        while !inner.woken.swap(false, Ordering::SeqCst) {
            let now = Instant::now();
            if now >= deadline {
                return None; // 깨우는 연락이 끝내 오지 않았습니다.
            }
            thread::park_timeout(deadline - now);
        }
    }
}

// ---------------------------------------------------------------------------
// 실행할 퓨처들
// ---------------------------------------------------------------------------

/// n번 Pending을 돌려주며 매번 스스로 깨우고, 끝나면 폴링된 횟수를 돌려줍니다.
struct Countdown {
    remaining: u32,
    polls: u32,
}

fn countdown(n: u32) -> Countdown {
    Countdown {
        remaining: n,
        polls: 0,
    }
}

impl Future for Countdown {
    type Output = u32;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        self.polls += 1;
        if self.remaining == 0 {
            return Poll::Ready(self.polls);
        }
        self.remaining -= 1;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

/// 첫 poll에서 깨울 준비 없이 Pending을 돌려주는 퓨처(03 예제와 같음).
struct Forgetful {
    polled: bool,
}

impl Future for Forgetful {
    type Output = &'static str;
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled {
            return Poll::Ready("두 번째 poll에서 끝났습니다");
        }
        self.polled = true;
        Poll::Pending
    }
}

/// 시각만 확인하는 타이머. 아직이면 바로 스스로 깨우므로 실행기가 쉬지 않고 다시 폴링합니다.
struct BusySleep {
    deadline: Instant,
    polls: u64,
}

impl Future for BusySleep {
    type Output = u64;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u64> {
        self.polls += 1;
        if Instant::now() >= self.deadline {
            Poll::Ready(self.polls)
        } else {
            cx.waker().wake_by_ref(); // "기다리는 일"이 없으니 바로 다시 불러 달라는 뜻이 됩니다.
            Poll::Pending
        }
    }
}

fn main() {
    println!("1. 재료: unpark가 park보다 먼저 와도 신호는 사라지지 않습니다");
    thread::current().unpark(); // 아직 아무도 잠들지 않았지만 "깨움 표시"가 남습니다.
    let start = Instant::now();
    thread::park(); // 표시가 있으므로 바로 돌아옵니다.
    println!(
        "    park가 바로 돌아왔는가(10ms 미만): {}",
        start.elapsed() < Duration::from_millis(10)
    ); // 출력: true

    println!("2. block_on으로 퓨처를 끝까지 실행합니다");
    let polls = block_on(countdown(3));
    println!("    countdown(3)이 폴링된 횟수: {polls}"); // 출력: 4
    let total = block_on(async {
        let a = countdown(1).await; // async 블록 안에서는 .await로 기다립니다.
        let b = countdown(2).await;
        a + b
    });
    println!("    async 블록 안의 두 countdown 폴링 횟수 합: {total}"); // 출력: 5 (2 + 3)

    println!("3. 깨우지 않으면 실행기는 다시 폴링하지 않습니다");
    let start = Instant::now();
    let result = block_on_for(Forgetful { polled: false }, Duration::from_millis(50));
    println!("    결과: {result:?}"); // 출력: None
    println!(
        "    포기하기 전에 50ms 이상 기다렸는가: {}",
        start.elapsed() >= Duration::from_millis(50)
    ); // 출력: true
    // block_on이었다면 여기서 영원히 잠들어 프로그램이 끝나지 않습니다.

    println!("4. 스스로 계속 깨우면 바쁜 폴링이 됩니다");
    let start = Instant::now();
    let polls = block_on(BusySleep {
        deadline: Instant::now() + Duration::from_millis(30),
        polls: 0,
    });
    println!(
        "    30ms를 기다리는 동안 폴링 횟수가 1000번을 넘는가: {}",
        polls > 1000
    ); // 출력: true (실제 횟수는 실행마다 다름)
    println!(
        "    걸린 시간이 30ms 이상인가: {}",
        start.elapsed() >= Duration::from_millis(30)
    ); // 출력: true
}
