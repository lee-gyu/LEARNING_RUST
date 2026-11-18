// 다른 스레드가 완료시키는 퓨처: 공유 상태(Arc<Mutex<..>>)와 저장해 둔 웨이커.
// poll은 "아직이면 웨이커를 맡겨 두고 Pending", 다른 스레드는 "일을 끝내면 맡겨 둔 웨이커로 깨움".
// 이 모양으로 타이머와 일회용 채널(oneshot)을 만들고, 04의 block_on으로 실행합니다.
//
// 실행: cargo run -p p1-ch13-async-primitives --example 05_timer_and_oneshot
//
// 시간은 "몇 ms 이상 걸렸는가"라는 비교 결과로만 출력합니다. 타이머 스레드는 웨이커를 부른 직후
// 끝나고, oneshot 예제의 스레드는 스코프 스레드라서 남는 스레드가 없습니다.

use std::pin::{Pin, pin};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 04 예제의 block_on (이 파일만으로 실행되도록 그대로 옮겨 왔습니다)
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

// ---------------------------------------------------------------------------
// 1. 타이머: 첫 poll에서 스레드를 띄우고, 시간이 되면 그 스레드가 깨워 줍니다.
// ---------------------------------------------------------------------------

/// 퓨처와 타이머 스레드가 함께 보는 상태.
struct TimerState {
    done: bool,
    waker: Option<Waker>, // 마지막 poll에서 받은 웨이커
}

struct Sleep {
    duration: Duration,
    state: Option<Arc<Mutex<TimerState>>>, // 첫 poll 전에는 None (퓨처는 게으릅니다)
    polls: u32,
}

fn sleep(duration: Duration) -> Sleep {
    Sleep {
        duration,
        state: None,
        polls: 0,
    }
}

impl Future for Sleep {
    type Output = u32; // 관찰용으로 폴링된 횟수를 돌려줍니다.

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        // Sleep의 필드는 모두 Unpin이므로 get_mut으로 평범한 &mut Sleep을 꺼냅니다(README 2.7절).
        // 그러면 필드를 따로따로 빌릴 수 있습니다.
        let this = self.get_mut();
        this.polls += 1;
        let duration = this.duration;

        let state = this.state.get_or_insert_with(|| {
            let state = Arc::new(Mutex::new(TimerState {
                done: false,
                waker: None,
            }));
            let for_thread = Arc::clone(&state);
            thread::spawn(move || {
                thread::sleep(duration);
                // 잠근 채로 완료를 표시하고 웨이커를 꺼낸 뒤, 잠금을 놓고 나서 깨웁니다.
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

        // 완료 확인과 웨이커 저장을 같은 잠금 안에서 합니다. 그래야 "확인한 직후 타이머가 끝나서
        // 저장하기도 전에 깨우려 했는데 웨이커가 없다"는 틈이 생기지 않습니다.
        let mut st = state.lock().unwrap();
        if st.done {
            return Poll::Ready(this.polls);
        }
        match &mut st.waker {
            // 이미 웨이커가 있으면 clone_from: 같은 태스크의 웨이커(will_wake가 true)면 복제를 건너뜁니다.
            Some(old) => old.clone_from(cx.waker()),
            None => st.waker = Some(cx.waker().clone()),
        }
        Poll::Pending
    }
}

// ---------------------------------------------------------------------------
// 2. 일회용 채널(oneshot): 값 하나를 다른 스레드에서 받아 기다립니다.
// ---------------------------------------------------------------------------

struct Slot<T> {
    value: Option<T>,
    sender_gone: bool,
    waker: Option<Waker>,
}

struct Sender<T> {
    slot: Arc<Mutex<Slot<T>>>,
}

struct Receiver<T> {
    slot: Arc<Mutex<Slot<T>>>,
}

fn oneshot<T>() -> (Sender<T>, Receiver<T>) {
    let slot = Arc::new(Mutex::new(Slot {
        value: None,
        sender_gone: false,
        waker: None,
    }));
    (
        Sender {
            slot: Arc::clone(&slot),
        },
        Receiver { slot },
    )
}

impl<T> Sender<T> {
    /// 값을 넣습니다. self를 소비하므로 함수가 끝날 때 아래 Drop이 불려 받는 쪽을 깨웁니다.
    fn send(self, value: T) {
        self.slot.lock().unwrap().value = Some(value);
    }
}

impl<T> Drop for Sender<T> {
    // 보내든 보내지 않든 송신자가 사라지면 받는 쪽을 깨웁니다. 깨우지 않으면 값 없이 버려진
    // 송신자를 기다리는 퓨처가 영원히 Pending으로 남습니다.
    fn drop(&mut self) {
        let waker = {
            let mut slot = self.slot.lock().unwrap();
            slot.sender_gone = true;
            slot.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl<T> Future for Receiver<T> {
    type Output = Option<T>; // 보내기 전에 송신자가 사라지면 None

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        // self.slot은 읽기만 하므로(Arc 복제 없이 잠금만) Pin<&mut Self>의 Deref로 충분합니다.
        let mut slot = self.slot.lock().unwrap();
        if let Some(value) = slot.value.take() {
            return Poll::Ready(Some(value));
        }
        if slot.sender_gone {
            return Poll::Ready(None);
        }
        slot.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

// ---------------------------------------------------------------------------
// 3. 마지막 poll의 웨이커만 깨워야 합니다: 깨워지면 이름을 채널로 보내는 웨이커
// ---------------------------------------------------------------------------

struct NamedWaker {
    name: &'static str,
    tx: mpsc::Sender<&'static str>, // mpsc::Sender는 Sync라서 Wake 구현에 그대로 둘 수 있습니다.
}

impl Wake for NamedWaker {
    fn wake(self: Arc<Self>) {
        let _ = self.tx.send(self.name);
    }
}

fn main() {
    println!("1. 타이머: 스레드가 시간이 되면 깨워 줍니다");
    let start = Instant::now();
    let polls = block_on(sleep(Duration::from_millis(50)));
    println!(
        "    sleep(50ms): 폴링 {polls}번, 50ms 이상 걸렸는가: {}",
        start.elapsed() >= Duration::from_millis(50)
    ); // 출력: 폴링 2번, 50ms 이상 걸렸는가: true

    let start = Instant::now();
    block_on(async {
        sleep(Duration::from_millis(30)).await;
        sleep(Duration::from_millis(30)).await; // 앞의 것이 끝나야 시작합니다.
    });
    println!(
        "    30ms 두 번을 차례로 await: 60ms 이상 걸렸는가: {}",
        start.elapsed() >= Duration::from_millis(60)
    ); // 출력: true (함께 기다리는 방법은 07 예제)

    println!("2. oneshot: 다른 스레드가 보낸 값 하나를 기다립니다");
    let (tx, rx) = oneshot();
    let received = thread::scope(|s| {
        s.spawn(move || {
            let sum: u64 = (1..=1_000_000).sum(); // 무거운 계산은 스레드에서
            tx.send(sum);
        });
        block_on(rx)
    });
    println!("    받은 값: {received:?}"); // 출력: Some(500000500000)

    let (tx, rx) = oneshot::<u64>();
    let received = thread::scope(|s| {
        s.spawn(move || {
            thread::sleep(Duration::from_millis(20));
            drop(tx); // 보내지 않고 송신자를 버립니다. Drop이 받는 쪽을 깨웁니다.
        });
        block_on(rx)
    });
    println!("    보내지 않고 버린 경우: {received:?}"); // 출력: None

    println!("3. 웨이커가 바뀌면 마지막 poll의 웨이커만 깨웁니다");
    let (tx, rx) = mpsc::channel();
    let waker_a = Waker::from(Arc::new(NamedWaker {
        name: "A",
        tx: tx.clone(),
    }));
    let waker_b = Waker::from(Arc::new(NamedWaker { name: "B", tx }));
    let mut timer = pin!(sleep(Duration::from_millis(20)));
    let first = timer.as_mut().poll(&mut Context::from_waker(&waker_a));
    let second = timer.as_mut().poll(&mut Context::from_waker(&waker_b));
    println!("    A로 poll: {first:?}, B로 poll: {second:?}"); // 출력: Pending, Pending
    let woken = rx.recv().unwrap(); // 타이머 스레드가 깨울 때까지 기다립니다.
    println!(
        "    깨워진 웨이커: {woken}, 다른 연락이 더 있는가: {}",
        rx.try_recv().is_ok()
    );
    // 출력: 깨워진 웨이커: B, 다른 연락이 더 있는가: false
    let last = timer.as_mut().poll(&mut Context::from_waker(&waker_b));
    println!("    B로 다시 poll: {last:?}"); // 출력: Ready(3)
}
