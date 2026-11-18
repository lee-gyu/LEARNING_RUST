// 퓨처를 직접 구현하기: Future 트레이트의 poll을 손으로 씁니다.
// 바로 끝나는 퓨처, 정해진 횟수만큼 Pending을 돌려주며 스스로 깨우는 퓨처,
// 깨우는 것을 잊은 퓨처, 그리고 async fn과 같은 일을 하는 손으로 쓴 상태 기계를 비교합니다.
//
// 실행: cargo run -p p1-ch13-async-primitives --example 03_custom_future
//
// 스레드를 쓰지 않으므로 출력은 실행할 때마다 같습니다.

use std::fmt::Debug;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Wake, Waker};

// ---------------------------------------------------------------------------
// 1. 바로 끝나는 퓨처: std::future::ready와 같은 일을 합니다.
// ---------------------------------------------------------------------------

struct Immediate<T>(Option<T>);

// 필드가 모두 Unpin이면 구조체도 자동으로 Unpin입니다. T: Unpin이면 Immediate<T>: Unpin이고,
// 그러면 Pin<&mut Self>를 &mut Self처럼 써서 self.0을 고칠 수 있습니다(README 2.7절).
impl<T: Unpin> Future for Immediate<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        // 값을 꺼내 돌려줍니다. 다시 폴링되면 None이므로 계약 위반을 패닉으로 알립니다.
        let value = self
            .0
            .take()
            .expect("Immediate가 끝난 뒤 다시 폴링되었습니다");
        Poll::Ready(value)
    }
}

// ---------------------------------------------------------------------------
// 2. 정해진 횟수만큼 Pending을 돌려주는 퓨처: 매번 스스로를 깨웁니다.
// ---------------------------------------------------------------------------

struct Countdown {
    remaining: u32,
}

impl Future for Countdown {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.remaining == 0 {
            return Poll::Ready("발사!");
        }
        self.remaining -= 1;
        // Pending을 돌려주기 전에 "다시 폴링해 달라"는 연락을 남깁니다. 기다릴 일이 따로 없으므로
        // 바로 깨웁니다. 실행기는 다른 태스크에 차례를 준 뒤 이 퓨처를 다시 폴링합니다.
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

// ---------------------------------------------------------------------------
// 3. 깨우는 것을 잊은 퓨처: 첫 poll에서 Pending을 돌려주면서 아무 연락도 남기지 않습니다.
// ---------------------------------------------------------------------------

struct Forgetful {
    polled: bool,
}

impl Future for Forgetful {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if self.polled {
            Poll::Ready(())
        } else {
            self.polled = true;
            Poll::Pending // 계약 위반: 깨울 준비 없이 Pending
        }
    }
}

// ---------------------------------------------------------------------------
// 4. async fn과 손으로 쓴 상태 기계
// ---------------------------------------------------------------------------

async fn add_slowly(a: u32, b: u32) -> u32 {
    println!("      [async fn] 시작");
    Countdown { remaining: 1 }.await; // await 지점 1
    println!("      [async fn] 중간");
    Countdown { remaining: 1 }.await; // await 지점 2
    println!("      [async fn] 끝");
    a + b
}

/// add_slowly와 같은 일을 하는 상태 기계. 배리언트는 "어느 await 지점에 멈춰 있는가",
/// 필드는 "그 지점을 넘어서도 살아 있어야 하는 값"입니다. 컴파일러가 async fn을 바꾸는 모양과
/// 개념적으로 같습니다.
enum AddSlowly {
    Start { a: u32, b: u32 },
    AtFirstAwait { a: u32, b: u32, inner: Countdown },
    AtSecondAwait { a: u32, b: u32, inner: Countdown },
    Done,
}

impl Future for AddSlowly {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        // 한 번의 poll에서 진행할 수 있는 만큼 상태를 넘깁니다. 안쪽 퓨처가 Pending이면 그대로 돌려줍니다.
        loop {
            match &mut *self {
                AddSlowly::Start { a, b } => {
                    println!("      [상태 기계] 시작");
                    let (a, b) = (*a, *b);
                    *self = AddSlowly::AtFirstAwait {
                        a,
                        b,
                        inner: Countdown { remaining: 1 },
                    };
                }
                AddSlowly::AtFirstAwait { a, b, inner } => {
                    // .await가 하는 일: 안쪽 퓨처를 폴링하고, Pending이면 바깥도 Pending.
                    // 같은 cx를 넘기므로 안쪽 퓨처가 남긴 연락처가 곧 바깥 태스크의 연락처입니다.
                    if Pin::new(inner).poll(cx).is_pending() {
                        return Poll::Pending;
                    }
                    println!("      [상태 기계] 중간");
                    // a와 b는 두 번째 await 뒤에서 쓰이므로 다음 상태로 들고 갑니다.
                    let (a, b) = (*a, *b);
                    *self = AddSlowly::AtSecondAwait {
                        a,
                        b,
                        inner: Countdown { remaining: 1 },
                    };
                }
                AddSlowly::AtSecondAwait { a, b, inner } => {
                    if Pin::new(inner).poll(cx).is_pending() {
                        return Poll::Pending;
                    }
                    println!("      [상태 기계] 끝");
                    let sum = *a + *b;
                    *self = AddSlowly::Done; // 끝난 뒤 다시 폴링되면 알아챌 수 있게 표시합니다.
                    return Poll::Ready(sum);
                }
                AddSlowly::Done => panic!("AddSlowly가 끝난 뒤 다시 폴링되었습니다"),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 손으로 폴링하는 도구
// ---------------------------------------------------------------------------

/// 깨워진 횟수만 세는 웨이커(02 예제의 간단한 판).
struct CountingWaker(AtomicUsize);

impl Wake for CountingWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// 퓨처가 끝날 때까지 손으로 폴링하며 한 줄씩 기록합니다. Pending인데 웨이커가 불리지 않았으면
/// 다시 폴링할 근거가 없으므로 멈춥니다. 실행기였다면 이 퓨처를 영원히 기다리게 됩니다(04 예제).
fn drive<F>(future: F) -> Option<F::Output>
where
    F: Future,
    F::Output: Debug,
{
    let counter = Arc::new(CountingWaker(AtomicUsize::new(0)));
    let waker = Waker::from(Arc::clone(&counter));
    let mut cx = Context::from_waker(&waker);
    let mut future = pin!(future);
    let mut polls = 0;
    loop {
        polls += 1;
        let wakes_before = counter.0.load(Ordering::SeqCst);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => {
                println!("    poll {polls}: Ready({value:?})");
                return Some(value);
            }
            Poll::Pending => {
                let woken = counter.0.load(Ordering::SeqCst) > wakes_before;
                println!("    poll {polls}: Pending, 그사이 웨이커가 불렸는가: {woken}");
                if !woken {
                    println!("    → 아무도 깨우지 않았으므로 더 폴링하지 않습니다");
                    return None;
                }
            }
        }
    }
}

fn main() {
    println!("1. 바로 끝나는 퓨처");
    drive(Immediate(Some(String::from("준비된 값"))));
    // 출력: poll 1: Ready("준비된 값")

    println!("2. Countdown {{ remaining: 3 }}: Pending 3번, 그때마다 스스로 깨움");
    drive(Countdown { remaining: 3 });
    // 출력: poll 1~3: Pending(웨이커 호출 true), poll 4: Ready("발사!")

    println!("3. 깨우는 것을 잊은 퓨처");
    let result = drive(Forgetful { polled: false });
    println!("    결과: {result:?}"); // 출력: 결과: None
    // 두 번째 poll이었다면 Ready였겠지만, 실행기는 연락이 없는 퓨처를 다시 폴링하지 않습니다.

    println!("4. async fn과 손으로 쓴 상태 기계는 같은 순서로 진행합니다");
    println!("  (가) async fn add_slowly(2, 3)");
    drive(add_slowly(2, 3));
    println!("  (나) AddSlowly::Start {{ a: 2, b: 3 }}");
    drive(AddSlowly::Start { a: 2, b: 3 });
    // 출력: 두 경우 모두 poll 1·2는 Pending, poll 3은 Ready(5)이고 시작·중간·끝 줄의 위치가 같습니다.
}
