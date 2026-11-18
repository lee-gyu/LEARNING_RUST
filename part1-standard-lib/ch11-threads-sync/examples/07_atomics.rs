// 원자 타입: Mutex 없이 정수와 bool을 여러 스레드에서 안전하게 고치기.
// fetch_add, load/store, compare_exchange, try_update, AtomicBool 멈춤 신호,
// 그리고 static mut 대신 쓰는 static 원자 변수.
//
// 실행: cargo run -p p1-ch11-threads-sync --example 07_atomics
//
// 1번 구역의 "load 후 store" 줄은 일부러 경쟁 조건을 만든 것이라 값이 실행마다 다릅니다.
// 나머지 줄은 순서와 무관한 개수·합계만 출력하므로 실행할 때마다 같습니다.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

// static에 둘 수 있습니다. AtomicU64::new는 const fn이고, 원자 타입은 Sync입니다.
// static mut와 달리 읽고 쓰는 데 unsafe가 필요 없습니다(Part 0 ch19).
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    // fetch_add는 "더하기 전의 값"을 돌려줍니다. 그 값이 이번 호출이 받은 ID입니다.
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

const THREADS: usize = 4;
const ROUNDS: usize = 100_000;

fn main() {
    println!("1. fetch_add는 읽기-더하기-쓰기를 한 번에 합니다");
    {
        let counter = AtomicUsize::new(0);
        thread::scope(|s| {
            for _ in 0..THREADS {
                s.spawn(|| {
                    for _ in 0..ROUNDS {
                        // 단순 카운터는 다른 메모리와 순서를 맞출 필요가 없어 Relaxed면 충분합니다.
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                });
            }
        });
        println!(
            "    fetch_add:     {} (기대값 {})",
            counter.load(Ordering::Relaxed),
            THREADS * ROUNDS
        );

        let racy = AtomicUsize::new(0);
        thread::scope(|s| {
            for _ in 0..THREADS {
                s.spawn(|| {
                    for _ in 0..ROUNDS {
                        // load와 store는 각각은 원자적이지만, 둘 사이에 다른 스레드가 끼어들 수
                        // 있습니다. 두 스레드가 같은 값을 읽고 같은 값을 쓰면 더하기 하나가 사라집니다.
                        let current = racy.load(Ordering::Relaxed);
                        racy.store(current + 1, Ordering::Relaxed);
                    }
                });
            }
        });
        println!(
            "    load 후 store: {} (기대값 {})",
            racy.load(Ordering::Relaxed),
            THREADS * ROUNDS
        );
        // 출력: fetch_add:     400000 (기대값 400000)
        // 출력: load 후 store: (실행마다 다름, 400000보다 작음) (기대값 400000)
    }

    println!("2. compare_exchange: 지금 값이 예상과 같을 때만 바꿉니다");
    {
        // 0이면 "아직 아무도 차지하지 않음". 여러 스레드가 동시에 자기 번호를 넣으려 합니다.
        let winner = AtomicUsize::new(0);
        let wins = thread::scope(|s| {
            let handles: Vec<_> = (1..=8)
                .map(|id| {
                    let winner = &winner;
                    s.spawn(move || {
                        winner
                            .compare_exchange(0, id, Ordering::SeqCst, Ordering::SeqCst)
                            .is_ok()
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .filter(|&ok| ok)
                .count()
        });
        // 누가 이겼는지는 실행마다 다르므로 번호 대신 "몇 명이 이겼는가"를 출력합니다.
        let id = winner.load(Ordering::SeqCst);
        println!(
            "    성공한 스레드 수: {wins}, 차지한 번호가 1~8 중 하나인가: {}",
            (1..=8).contains(&id)
        );
        // 출력: 성공한 스레드 수: 1, 차지한 번호가 1~8 중 하나인가: true
    }

    println!("3. try_update: 조건을 만족할 때만 고치기(재고 5개를 8명이 동시에 주문)");
    {
        let stock = AtomicUsize::new(5);
        let results = thread::scope(|s| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    s.spawn(|| {
                        // 클로저가 None을 돌려주면 바꾸지 않고 Err(지금 값)을 돌려줍니다.
                        // checked_sub는 0에서 빼면 None이므로 재고가 음수가 되지 않습니다.
                        stock.try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        let ok = results.iter().filter(|r| r.is_ok()).count();
        println!(
            "    주문 성공 {ok}, 실패 {}, 남은 재고 {}",
            results.len() - ok,
            stock.load(Ordering::SeqCst)
        );
        // 출력: 주문 성공 5, 실패 3, 남은 재고 0
    }

    println!("4. AtomicBool로 작업자에게 멈추라고 알리기");
    {
        let stop = AtomicBool::new(false);
        let rounds = thread::scope(|s| {
            let worker = s.spawn(|| {
                let mut rounds = 0u32;
                while !stop.load(Ordering::Relaxed) {
                    rounds += 1;
                    thread::sleep(Duration::from_millis(1)); // 일을 조금 한다고 생각합니다.
                }
                rounds
            });
            thread::sleep(Duration::from_millis(20));
            stop.store(true, Ordering::Relaxed);
            worker.join().unwrap()
        });
        // 반복 횟수 자체는 실행마다 다르므로 멈췄다는 사실만 확인합니다.
        println!(
            "    작업자가 신호를 보고 멈췄습니다 (한 번 이상 일했는가: {})",
            rounds > 0
        );
        // 출력: 작업자가 신호를 보고 멈췄습니다 (한 번 이상 일했는가: true)
    }

    println!("5. static 원자 변수로 프로그램 전체에서 겹치지 않는 ID 나눠 주기");
    {
        let mut ids: Vec<u64> = thread::scope(|s| {
            let handles: Vec<_> = (0..4)
                .map(|_| s.spawn(|| [next_id(), next_id(), next_id()]))
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap())
                .collect()
        });
        ids.sort(); // 어느 스레드가 어느 ID를 받았는지는 실행마다 다릅니다.
        println!("    받은 ID(정렬): {ids:?}");
        println!("    다음에 나갈 ID: {}", NEXT_ID.load(Ordering::Relaxed));
        // 출력: 받은 ID(정렬): [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
        // 출력: 다음에 나갈 ID: 13
    }
}
