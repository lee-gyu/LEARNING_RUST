// 퓨처를 조합하는 퓨처와 std::future의 도구들.
// 두 퓨처를 함께 기다리는 join을 직접 만들고, poll_fn·task::ready!·IntoFuture를 씁니다.
//
// 실행: cargo run -p p1-ch13-async-primitives --example 07_join_and_tools
//
// 시간은 "몇 ms 이상/미만인가"라는 비교 결과로만 출력합니다. 그 밖의 출력은 스레드 순서에
// 영향을 받지 않으므로 실행할 때마다 같습니다.

use std::cell::RefCell;
use std::future::{self, poll_fn};
use std::pin::{Pin, pin};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker, ready};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 04의 block_on과 05의 sleep (이 파일만으로 실행되도록 옮겨 왔습니다)
// ---------------------------------------------------------------------------

struct ThreadWaker {
    thread: Thread,
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

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let inner = Arc::new(ThreadWaker {
        thread: thread::current(),
        woken: AtomicBool::new(false),
    });
    let waker = Waker::from(Arc::clone(&inner));
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                while !inner.woken.swap(false, Ordering::SeqCst) {
                    thread::park();
                }
            }
        }
    }
}

struct TimerState {
    done: bool,
    waker: Option<Waker>,
}

struct Sleep {
    duration: Duration,
    state: Option<Arc<Mutex<TimerState>>>,
}

fn sleep(duration: Duration) -> Sleep {
    Sleep {
        duration,
        state: None,
    }
}

impl Future for Sleep {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        let duration = this.duration;
        let state = this.state.get_or_insert_with(|| {
            let state = Arc::new(Mutex::new(TimerState {
                done: false,
                waker: None,
            }));
            let for_thread = Arc::clone(&state);
            thread::spawn(move || {
                thread::sleep(duration);
                let waker = {
                    let mut st = for_thread.lock().unwrap();
                    st.done = true;
                    st.waker.take()
                };
                if let Some(waker) = waker {
                    waker.wake();
                }
            });
            state
        });
        let mut st = state.lock().unwrap();
        if st.done {
            return Poll::Ready(());
        }
        st.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

// ---------------------------------------------------------------------------
// 1. 직접 만든 join: 두 퓨처를 번갈아 폴링하고, 둘 다 끝나면 결과를 튜플로 돌려줍니다.
// ---------------------------------------------------------------------------

struct Join<A: Future, B: Future> {
    // 안쪽 퓨처는 힙에 고정해 둡니다. Pin<Box<_>>는 언제나 Unpin이므로 Join이 Unpin이 되기 쉽고,
    // 그러면 unsafe 없이 get_mut으로 필드를 고칠 수 있습니다.
    a: Pin<Box<A>>,
    b: Pin<Box<B>>,
    a_out: Option<A::Output>, // 먼저 끝난 쪽의 결과를 보관하는 칸
    b_out: Option<B::Output>,
}

fn join<A: Future, B: Future>(a: A, b: B) -> Join<A, B> {
    Join {
        a: Box::pin(a),
        b: Box::pin(b),
        a_out: None,
        b_out: None,
    }
}

// 출력 타입이 Unpin이어야 Join 전체가 Unpin입니다. 대부분의 값(i32, String, 튜플 등)은 Unpin입니다.
impl<A, B> Future for Join<A, B>
where
    A: Future,
    B: Future,
    A::Output: Unpin,
    B::Output: Unpin,
{
    type Output = (A::Output, B::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        // 아직 끝나지 않은 쪽만 폴링합니다. 끝난 퓨처를 다시 폴링하면 안 되기 때문입니다.
        // 두 퓨처에 같은 cx를 넘기므로 어느 쪽이 깨우든 이 Join이 다시 폴링됩니다.
        if this.a_out.is_none()
            && let Poll::Ready(value) = this.a.as_mut().poll(cx)
        {
            this.a_out = Some(value);
        }
        if this.b_out.is_none()
            && let Poll::Ready(value) = this.b.as_mut().poll(cx)
        {
            this.b_out = Some(value);
        }
        if this.a_out.is_some() && this.b_out.is_some() {
            Poll::Ready((this.a_out.take().unwrap(), this.b_out.take().unwrap()))
        } else {
            Poll::Pending // 깨우는 일은 안쪽 퓨처가 맡았으므로 여기서는 따로 할 일이 없습니다.
        }
    }
}

// ---------------------------------------------------------------------------
// 2. poll_fn: 구조체 없이 클로저 하나로 퓨처 만들기
// ---------------------------------------------------------------------------

/// 처음 한 번 차례를 양보하는 퓨처(06 예제의 YieldOnce와 같음).
fn yield_now() -> impl Future<Output = ()> {
    let mut yielded = false; // 클로저가 소유하는 상태
    poll_fn(move |cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
}

/// 한 단계마다 기록을 남기고 차례를 양보하는 작업. 모두 한 스레드에서 실행되므로 기록장은 RefCell입니다.
async fn steps(name: &'static str, count: u32, log: &RefCell<Vec<String>>) -> u32 {
    for i in 1..=count {
        log.borrow_mut().push(format!("{name}{i}")); // 대여는 이 문장 안에서 끝납니다.
        yield_now().await;
    }
    count * 100
}

// ---------------------------------------------------------------------------
// 3. task::ready!: Pending이면 바로 돌려주고, Ready면 값을 꺼냅니다.
// ---------------------------------------------------------------------------

struct Map<F, G> {
    inner: Pin<Box<F>>,
    f: Option<G>, // 한 번만 부를 FnOnce라서 Option에 넣었다가 꺼냅니다.
}

fn map<F, G>(future: F, f: G) -> Map<F, G> {
    Map {
        inner: Box::pin(future),
        f: Some(f),
    }
}

impl<F, G, T> Future for Map<F, G>
where
    F: Future,
    G: FnOnce(F::Output) -> T + Unpin,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let this = self.get_mut();
        // match this.inner.as_mut().poll(cx) { Ready(v) => v, Pending => return Poll::Pending }
        let value = ready!(this.inner.as_mut().poll(cx));
        let f = this.f.take().expect("Map이 끝난 뒤 다시 폴링되었습니다");
        Poll::Ready(f(value))
    }
}

// ---------------------------------------------------------------------------
// 4. IntoFuture: .await할 때 퓨처로 바뀌는 빌더
// ---------------------------------------------------------------------------

struct Reminder {
    message: &'static str,
    after: Duration,
}

impl Reminder {
    fn new(message: &'static str) -> Self {
        Reminder {
            message,
            after: Duration::ZERO,
        }
    }
    fn after_millis(mut self, ms: u64) -> Self {
        self.after = Duration::from_millis(ms);
        self
    }
}

// IntoFuture는 Edition 2024 프렐류드에 있어 use 없이 구현합니다.
impl IntoFuture for Reminder {
    type Output = String;
    // 연관 타입에는 impl Future를 쓸 수 없으므로 상자에 담은 트레이트 객체로 적습니다.
    type IntoFuture = Pin<Box<dyn Future<Output = String>>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            sleep(self.after).await;
            format!("알림: {}", self.message)
        })
    }
}

fn main() {
    println!("1. 직접 만든 join: 두 작업이 번갈아 진행됩니다");
    let log = RefCell::new(Vec::new());
    let result = block_on(join(steps("A", 3, &log), steps("B", 2, &log)));
    println!("    진행 순서: {:?}", log.borrow()); // 출력: ["A1", "B1", "A2", "B2", "A3"]
    println!("    결과(인자 순서대로): {result:?}"); // 출력: (300, 200)

    let start = Instant::now();
    block_on(join(
        sleep(Duration::from_millis(50)),
        sleep(Duration::from_millis(50)),
    ));
    let elapsed = start.elapsed();
    println!(
        "    50ms 타이머 두 개를 join: 50ms 이상인가 {}, 100ms 미만인가 {}",
        elapsed >= Duration::from_millis(50),
        elapsed < Duration::from_millis(100)
    ); // 출력: true, true (05의 차례로 await와 비교)

    println!("2. poll_fn: 한 번만 폴링해 보고 결과를 값으로 받기");
    let outcome = block_on(async {
        let mut timer = pin!(sleep(Duration::from_millis(20)));
        // 바깥 async 블록의 cx를 그대로 넘겨 timer를 딱 한 번 폴링하고, 그 Poll을 값으로 돌려줍니다.
        let peek = poll_fn(|cx| Poll::Ready(timer.as_mut().poll(cx))).await;
        timer.await; // Pin<&mut Sleep>도 Future입니다. 끝까지 기다립니다.
        peek
    });
    println!("    첫 폴링 결과: {outcome:?}"); // 출력: Pending

    println!("3. task::ready!로 만든 map");
    let text = block_on(map(steps("C", 1, &log), |n| format!("{n}점")));
    println!("    {text}"); // 출력: 100점
    let doubled = block_on(map(future::ready(21), |n| n * 2));
    println!("    ready(21)에 map: {doubled}"); // 출력: 42

    println!("4. IntoFuture: .await가 into_future()를 불러 줍니다");
    let start = Instant::now();
    let text = block_on(async { Reminder::new("물 마시기").after_millis(30).await });
    println!(
        "    {text} (30ms 이상 기다렸는가: {})",
        start.elapsed() >= Duration::from_millis(30)
    ); // 출력: 알림: 물 마시기 (30ms 이상 기다렸는가: true)
    let future = Reminder::new("바로").into_future(); // 직접 부르면 퓨처를 손에 쥘 수 있습니다.
    println!("    {}", block_on(future)); // 출력: 알림: 바로
}
