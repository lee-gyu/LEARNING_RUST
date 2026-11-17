// 태스크와 스레드를 비교하고, 둘을 함께 쓰는 방법을 보여 줍니다.
// - 태스크는 운영체제가 아니라 런타임이 관리하는 가벼운 실행 단위입니다. 만 개를 만들어도 부담이 적습니다.
// - block_on에 넘긴 퓨처는 그것을 부른 스레드(main)에서, spawn_task로 만든 태스크는 런타임의 작업자 스레드에서 실행됩니다.
// - 계산이 무거운 일은 스레드에 맡기고, 결과는 비동기 채널로 보내 async 코드에서 기다릴 수 있습니다.
//
// 실행: cargo run -p p0-ch16-async --example 08_tasks_and_threads
//
// 주의: 1번의 걸린 시간은 100ms 단위로 반올림해 출력하며, 느린 컴퓨터에서는 더 길게 나올 수 있습니다.
// 2번의 작업자 스레드 이름은 trpl이 내부에서 쓰는 tokio가 정한 것으로, tokio 버전에 따라 다를 수 있습니다.

use std::thread;
use std::time::{Duration, Instant};

fn rounded_ms(start: Instant) -> u128 {
    (start.elapsed().as_millis() + 50) / 100 * 100
}

// 지금 실행 중인 스레드의 이름을 돌려줍니다. 이름 없는 스레드면 "(이름 없음)"입니다.
fn current_thread_name() -> String {
    thread::current()
        .name()
        .unwrap_or("(이름 없음)")
        .to_string()
}

fn main() {
    trpl::block_on(async {
        println!("1. 태스크 10,000개가 각자 100ms씩 기다려도 전체는 짧게 끝납니다");
        let start = Instant::now();
        let mut handles = Vec::new();
        for i in 1..=10_000u64 {
            handles.push(trpl::spawn_task(async move {
                trpl::sleep(Duration::from_millis(100)).await;
                i
            }));
        }
        let mut sum = 0;
        for handle in handles {
            sum += handle.await.unwrap();
        }
        println!(
            "    결과의 합: {sum}, 걸린 시간: 약 {}ms",
            rounded_ms(start)
        );
        // 출력:
        //     결과의 합: 50005000, 걸린 시간: 약 100ms

        println!("2. 퓨처와 태스크가 실행되는 스레드");
        println!("    block_on의 퓨처: {}", current_thread_name());
        // join으로 묶은 퓨처들은 한 퓨처로 합쳐져 같은 태스크 안에 있습니다. 그래서 한 스레드에서
        // 번갈아 실행될 뿐(동시성), 서로 다른 스레드에서 동시에 실행되지는 않습니다(병렬성 없음).
        let first = async { current_thread_name() };
        let second = async { current_thread_name() };
        let (a, b) = trpl::join(first, second).await;
        println!("    join한 퓨처 두 개: {a}, {b}");
        // spawn_task로 만든 태스크는 런타임의 작업자 스레드로 가서, 다른 태스크와 병렬로 실행될 수 있습니다.
        let handle = trpl::spawn_task(async { current_thread_name() });
        let spawned = handle.await.unwrap();
        println!("    spawn_task의 태스크: {spawned}");
        // 출력:
        //     block_on의 퓨처: main
        //     join한 퓨처 두 개: main, main
        //     spawn_task의 태스크: tokio-rt-worker
    });

    println!("3. 무거운 계산은 스레드에서 하고, 결과는 비동기 채널로 받습니다");
    let (tx, mut rx) = trpl::channel();
    for id in 1..=3u64 {
        let tx = tx.clone();
        // 비동기 채널의 send는 async가 아닌 평범한 메서드라서 일반 스레드에서도 부를 수 있습니다.
        thread::spawn(move || {
            let sum: u64 = (1..=id * 1_000_000).sum();
            tx.send((id, sum)).unwrap();
        });
    }
    // main이 들고 있던 원래 tx를 버립니다. 그래야 세 스레드가 끝날 때 채널이 닫혀 recv가 None을 돌려줍니다.
    drop(tx);
    trpl::block_on(async {
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }
        // 스레드가 끝나는 순서는 실행마다 다를 수 있으므로 id 순으로 정렬해 출력합니다.
        results.sort();
        for (id, sum) in results {
            println!("    스레드 {id}: 1부터 {}까지의 합 = {sum}", id * 1_000_000);
        }
    });
    // 출력:
    //     스레드 1: 1부터 1000000까지의 합 = 500000500000
    //     스레드 2: 1부터 2000000까지의 합 = 2000001000000
    //     스레드 3: 1부터 3000000까지의 합 = 4500001500000
}
