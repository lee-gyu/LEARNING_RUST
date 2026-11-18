// 여러 태스크를 돌리는 작은 실행기: spawn한 퓨처를 태스크 대기열(채널)에 넣고, 실행기는 대기열에서
// 하나씩 꺼내 폴링합니다. 태스크의 웨이커는 "자기 자신을 대기열에 다시 넣는 것"입니다.
// 대기열이 비면 실행기는 recv에서 잠들고, 어느 태스크든 깨워지면 다시 일합니다.
//
// 실행: cargo run -p p1-ch13-async-primitives --example 08_mini_executor
//
// 1번 구역은 스스로 깨우는 태스크만 있어 순서가 언제나 같습니다. 2번 구역은 타이머 스레드가
// 깨우므로 끝나는 시각은 실행마다 조금씩 다르고, 시간은 비교 결과로만 출력합니다.

use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 실행기
// ---------------------------------------------------------------------------

/// 태스크가 들고 있는 퓨처. 서로 다른 async 블록을 한 타입으로 담기 위해 트레이트 객체로 만들고,
/// 웨이커를 따라 다른 스레드로 건너갈 수 있으므로 Send를 요구합니다.
type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

struct Task {
    name: &'static str,
    // 끝난 태스크는 None. 같은 태스크가 대기열에 두 번 들어갈 수 있으므로(두 번 깨워짐)
    // 꺼냈는데 이미 끝났으면 건너뜁니다. Arc<Task>가 여러 스레드에서 공유되므로 Mutex로 감쌉니다.
    future: Mutex<Option<BoxFuture>>,
    queue: Sender<Arc<Task>>,
    polls: AtomicUsize,
}

// 웨이커 = "이 태스크를 대기열에 다시 넣어 주세요".
impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        // 실행기가 이미 끝났다면(수신자가 없음) 보낼 곳이 없으므로 무시합니다.
        let _ = self.queue.send(Arc::clone(self));
    }
}

struct Executor {
    sender: Sender<Arc<Task>>,
    queue: Receiver<Arc<Task>>,
    unfinished: usize,
}

impl Executor {
    fn new() -> Self {
        let (sender, queue) = mpsc::channel();
        Executor {
            sender,
            queue,
            unfinished: 0,
        }
    }

    /// 퓨처를 태스크로 만들어 대기열에 넣습니다. 아직 실행하지는 않습니다.
    fn spawn(&mut self, name: &'static str, future: impl Future<Output = ()> + Send + 'static) {
        let task = Arc::new(Task {
            name,
            future: Mutex::new(Some(Box::pin(future))),
            queue: self.sender.clone(),
            polls: AtomicUsize::new(0),
        });
        self.sender.send(task).unwrap();
        self.unfinished += 1;
    }

    /// 모든 태스크가 끝날 때까지 실행하고, (이름, 폴링 횟수)를 끝난 순서대로 돌려줍니다.
    fn run(mut self) -> Vec<(&'static str, usize)> {
        let mut finished = Vec::new();
        while self.unfinished > 0 {
            // 대기열이 비어 있으면 누군가 깨울 때까지 여기서 잠듭니다(바쁜 폴링이 아닙니다).
            let task = self.queue.recv().unwrap();
            let mut slot = task.future.lock().unwrap();
            let Some(future) = slot.as_mut() else {
                continue; // 이미 끝난 태스크가 한 번 더 깨워진 경우
            };
            let waker = Waker::from(Arc::clone(&task));
            let mut cx = Context::from_waker(&waker);
            let polls = task.polls.fetch_add(1, Ordering::SeqCst) + 1;
            if future.as_mut().poll(&mut cx).is_ready() {
                *slot = None; // 퓨처를 버립니다. 안에 있던 값들도 이때 drop됩니다.
                self.unfinished -= 1;
                finished.push((task.name, polls));
            }
            // Pending이면 할 일이 없습니다. 깨워 주는 쪽이 태스크를 대기열에 다시 넣습니다.
        }
        finished
    }
}

// ---------------------------------------------------------------------------
// 태스크 안에서 쓸 퓨처 (06·05 예제와 같음)
// ---------------------------------------------------------------------------

struct YieldOnce {
    yielded: bool,
}

fn yield_now() -> YieldOnce {
    YieldOnce { yielded: false }
}

impl Future for YieldOnce {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref(); // 대기열의 맨 뒤로 갑니다.
            Poll::Pending
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
                    waker.wake(); // 타이머 스레드에서 태스크를 대기열에 넣습니다.
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

fn main() {
    println!("1. 스스로 양보하는 태스크 셋: 대기열 순서대로 번갈아 실행됩니다");
    let log = Arc::new(Mutex::new(Vec::new())); // 태스크는 'static이어야 하므로 Arc로 공유합니다.
    let mut executor = Executor::new();
    for (name, steps) in [("A", 3), ("B", 2), ("C", 1)] {
        let log = Arc::clone(&log);
        executor.spawn(name, async move {
            for i in 1..=steps {
                log.lock().unwrap().push(format!("{name}{i}"));
                yield_now().await;
            }
        });
    }
    let finished = executor.run();
    println!("    실행 순서: {}", log.lock().unwrap().join(" ")); // 출력: A1 B1 C1 A2 B2 A3
    println!("    끝난 순서와 폴링 횟수: {finished:?}"); // 출력: [("C", 2), ("B", 3), ("A", 4)]

    println!("2. 타이머를 기다리는 태스크 셋: 기다리는 동안 실행기는 잠듭니다");
    let start = Instant::now();
    let mut executor = Executor::new();
    for (name, ms) in [("짧은", 20), ("중간", 60), ("긴", 100)] {
        executor.spawn(name, async move {
            sleep(Duration::from_millis(ms)).await;
        });
    }
    let mut finished = executor.run();
    let elapsed = start.elapsed();
    finished.sort(); // 끝나는 시각은 타이머 스레드에 달려 있으므로 이름순으로 정렬해 출력합니다.
    println!("    태스크별 폴링 횟수: {finished:?}"); // 출력: [("긴", 2), ("중간", 2), ("짧은", 2)]
    println!(
        "    전체 시간이 가장 긴 타이머(100ms) 이상인가: {}, 세 타이머의 합(180ms)보다 짧은가: {}",
        elapsed >= Duration::from_millis(100),
        elapsed < Duration::from_millis(180)
    ); // 출력: true, true
}
