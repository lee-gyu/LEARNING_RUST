// Condvar: 조건이 만족될 때까지 CPU를 쓰지 않고 잠들어 기다리기.
// notify는 쌓이지 않으므로 조건은 항상 Mutex 안의 값으로 확인합니다(wait_while).
// 마지막 구역은 Mutex<VecDeque> + Condvar 두 개로 용량이 정해진 대기열을 만들어
// 생산자-소비자를 구현합니다.
//
// 실행: cargo run -p p1-ch11-threads-sync --example 05_condvar
//
// 스레드 안에서 출력하지 않고 결과만 돌려받으므로 출력은 실행할 때마다 같습니다.
// 2번 구역은 제한 시간(30ms)을 일부러 넘기는 대기를 두 번 합니다.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Duration;

/// 용량이 정해진 대기열. 가득 차면 push가, 비어 있으면 pop이 기다립니다.
struct BoundedQueue<T> {
    items: Mutex<VecDeque<T>>,
    not_empty: Condvar, // "하나 이상 들어 있다"를 기다리는 소비자용
    not_full: Condvar,  // "빈자리가 있다"를 기다리는 생산자용
    capacity: usize,
}

impl<T> BoundedQueue<T> {
    fn new(capacity: usize) -> Self {
        BoundedQueue {
            items: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        }
    }

    /// 넣은 뒤의 길이를 돌려줍니다.
    fn push(&self, item: T) -> usize {
        let guard = self.items.lock().unwrap();
        // wait_while은 조건이 참인 동안 잠듭니다. 잠들 때 잠금을 풀고, 깨어나면 다시 잠근 뒤
        // 조건을 또 확인합니다. 그래서 가짜 깨어남이 있어도 가득 찬 대기열에 넣지 않습니다.
        let mut items = self
            .not_full
            .wait_while(guard, |items| items.len() == self.capacity)
            .unwrap();
        items.push_back(item);
        let len = items.len();
        drop(items); // 깨울 상대가 바로 잠금을 얻을 수 있게 먼저 놓습니다.
        self.not_empty.notify_one(); // 기다리는 소비자 하나를 깨웁니다.
        len
    }

    fn pop(&self) -> T {
        let guard = self.items.lock().unwrap();
        let mut items = self
            .not_empty
            .wait_while(guard, |items| items.is_empty())
            .unwrap();
        let item = items.pop_front().unwrap(); // 조건 덕분에 비어 있지 않음이 보장됩니다.
        drop(items);
        self.not_full.notify_one();
        item
    }
}

fn main() {
    println!("1. 출발 신호: 모든 작업자를 한꺼번에 깨울 때는 notify_all");
    {
        let ready = Mutex::new(false);
        let signal = Condvar::new();

        let woke = thread::scope(|s| {
            let workers: Vec<_> = (1..=3)
                .map(|id| {
                    let (ready, signal) = (&ready, &signal);
                    s.spawn(move || {
                        let guard = ready.lock().unwrap();
                        // 조건(!ready)이 참인 동안 기다립니다. main이 먼저 신호를 보냈더라도
                        // 조건이 이미 거짓이라 기다리지 않고 바로 지나갑니다.
                        let guard = signal.wait_while(guard, |ready| !*ready).unwrap();
                        format!("작업자 {id}: ready = {}", *guard)
                    })
                })
                .collect();

            *ready.lock().unwrap() = true; // 조건을 먼저 바꾸고
            signal.notify_all(); // 기다리는 모든 스레드를 깨웁니다.
            workers
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        println!("    {woke:?}");
        // 출력: ["작업자 1: ready = true", "작업자 2: ready = true", "작업자 3: ready = true"]
    }

    println!("2. notify는 쌓이지 않습니다: 조건을 확인하지 않는 대기는 신호를 놓칩니다");
    {
        let done = Mutex::new(false);
        let signal = Condvar::new();

        // 아무도 기다리지 않을 때 보낸 신호는 그냥 사라집니다.
        *done.lock().unwrap() = true;
        signal.notify_one();

        // (가) 조건 없이 wait_timeout: 이미 지나간 신호를 기다리다 시간이 다 됩니다.
        let guard = done.lock().unwrap();
        let (guard, result) = signal
            .wait_timeout(guard, Duration::from_millis(30))
            .unwrap();
        println!(
            "    (가) 조건 없이 기다림  → 시간 초과: {}",
            result.timed_out()
        );
        drop(guard);

        // (나) wait_timeout_while: 조건(!done)이 이미 거짓이라 기다리지 않습니다.
        let guard = done.lock().unwrap();
        let (_guard, result) = signal
            .wait_timeout_while(guard, Duration::from_millis(30), |done| !*done)
            .unwrap();
        println!(
            "    (나) 조건을 보며 기다림 → 시간 초과: {}",
            result.timed_out()
        );

        // (다) 조건이 끝내 만족되지 않으면 wait_timeout_while도 시간 초과로 돌아옵니다.
        // Condvar 하나는 Mutex 하나와만 짝지어 씁니다. 위의 signal을 다른 Mutex와 쓰면
        // 이 환경(macOS)에서는 "attempted to use a condition variable with two mutexes"로 패닉합니다.
        let never = Mutex::new(false);
        let never_signal = Condvar::new();
        let (_guard, result) = never_signal
            .wait_timeout_while(never.lock().unwrap(), Duration::from_millis(30), |v| !*v)
            .unwrap();
        println!(
            "    (다) 아무도 바꾸지 않음 → 시간 초과: {}",
            result.timed_out()
        );
        // 출력: (가) 조건 없이 기다림  → 시간 초과: true
        // 출력: (나) 조건을 보며 기다림 → 시간 초과: false
        // 출력: (다) 아무도 바꾸지 않음 → 시간 초과: true
    }

    println!("3. 생산자-소비자: 용량 3짜리 대기열에 생산자 2개가 10개씩 넣습니다");
    {
        let queue = BoundedQueue::new(3);

        let (max_len, received) = thread::scope(|s| {
            let producers: Vec<_> = (0..2)
                .map(|p| {
                    let queue = &queue;
                    s.spawn(move || {
                        // 넣을 때마다 길이를 기록해 최대 길이를 돌려줍니다.
                        (1..=10).map(|n| queue.push(p * 100 + n)).max().unwrap()
                    })
                })
                .collect();

            let consumer = s.spawn(|| {
                let mut got = Vec::new();
                for _ in 0..20 {
                    got.push(queue.pop());
                    thread::sleep(Duration::from_millis(1)); // 느린 소비자 → 대기열이 금방 참
                }
                got
            });

            let max_len = producers
                .into_iter()
                .map(|h| h.join().unwrap())
                .max()
                .unwrap();
            (max_len, consumer.join().unwrap())
        });

        // 두 생산자의 값이 섞이는 순서는 실행마다 다르므로, 순서와 무관한 사실만 출력합니다.
        let from_0: Vec<_> = received.iter().filter(|&&v| v < 100).collect();
        let from_1: Vec<_> = received.iter().filter(|&&v| v > 100).collect();
        println!(
            "    받은 개수: {}, 합계: {}",
            received.len(),
            received.iter().sum::<u32>()
        );
        println!(
            "    생산자 0의 값이 넣은 순서대로 왔는가: {}",
            from_0.is_sorted()
        );
        println!(
            "    생산자 1의 값이 넣은 순서대로 왔는가: {}",
            from_1.is_sorted()
        );
        println!(
            "    대기열 길이가 용량(3)을 넘은 적이 없는가: {}",
            max_len <= 3
        );
        // 출력: 받은 개수: 20, 합계: 1110
        // 출력: 생산자 0의 값이 넣은 순서대로 왔는가: true
        // 출력: 생산자 1의 값이 넣은 순서대로 왔는가: true
        // 출력: 대기열 길이가 용량(3)을 넘은 적이 없는가: true
    }
}
