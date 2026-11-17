// 퓨처는 await 지점에서만 다른 퓨처에게 차례를 넘긴다는 것(협력적 멀티태스킹)을 보여 줍니다.
// - await 없이 오래 도는 코드는 그동안 같은 태스크의 다른 퓨처를 전혀 진행시키지 못합니다(굶주림, starvation).
// - trpl::yield_now().await를 사이사이에 넣으면 "잠깐 차례를 넘기겠다"고 런타임에 알려 번갈아 진행됩니다.
// - 그래도 한 스레드에서 번갈아 할 뿐이므로 전체 시간은 줄지 않습니다. 계산 자체를 나누려면 스레드가 필요합니다.
//
// 실행: cargo run -p p0-ch16-async --example 05_yield_now
//
// 주의: 걸린 시간은 실행마다 몇 ms씩 달라지므로 100ms 단위로 반올림해 출력합니다.
// 줄의 순서는 타이머가 아니라 await 지점의 위치로 정해지므로 실행마다 같습니다.

use std::thread;
use std::time::{Duration, Instant};

// 오래 걸리는 계산을 흉내 냅니다. thread::sleep은 trpl::sleep과 달리 퓨처가 아니라
// 스레드 자체를 블로킹하는 평범한 함수입니다. 그래서 이 함수가 도는 동안에는 런타임이 끼어들 틈이 없습니다.
fn slow(name: &str, ms: u64) {
    thread::sleep(Duration::from_millis(ms));
    println!("    [{name}] {ms}ms 계산");
}

fn rounded_ms(start: Instant) -> u128 {
    (start.elapsed().as_millis() + 50) / 100 * 100
}

fn main() {
    trpl::block_on(async {
        println!("1. await 지점이 없으면 한 퓨처가 끝날 때까지 다른 퓨처는 기다리기만 합니다");
        let start = Instant::now();
        let a = async {
            println!("    [a] 시작");
            slow("a", 30);
            slow("a", 10);
            slow("a", 20);
            println!("    [a] 끝");
        };
        let b = async {
            println!("    [b] 시작");
            slow("b", 70);
            slow("b", 10);
            slow("b", 60);
            println!("    [b] 끝");
        };
        trpl::join(a, b).await;
        println!("    걸린 시간: 약 {}ms", rounded_ms(start));
        // 출력:
        //     [a] 시작
        //     [a] 30ms 계산
        //     [a] 10ms 계산
        //     [a] 20ms 계산
        //     [a] 끝
        //     [b] 시작
        //     [b] 70ms 계산
        //     [b] 10ms 계산
        //     [b] 60ms 계산
        //     [b] 끝
        //     걸린 시간: 약 200ms

        println!("2. yield_now로 사이사이 차례를 넘기면 번갈아 진행됩니다");
        let start = Instant::now();
        let a = async {
            println!("    [a] 시작");
            slow("a", 30);
            trpl::yield_now().await;
            slow("a", 10);
            trpl::yield_now().await;
            slow("a", 20);
            trpl::yield_now().await;
            println!("    [a] 끝");
        };
        let b = async {
            println!("    [b] 시작");
            slow("b", 70);
            trpl::yield_now().await;
            slow("b", 10);
            trpl::yield_now().await;
            slow("b", 60);
            trpl::yield_now().await;
            println!("    [b] 끝");
        };
        trpl::join(a, b).await;
        // 번갈아 진행될 뿐, 같은 스레드에서 차례로 계산하므로 전체 시간은 1번과 같습니다.
        println!("    걸린 시간: 약 {}ms", rounded_ms(start));
        // 출력:
        //     [a] 시작
        //     [a] 30ms 계산
        //     [b] 시작
        //     [b] 70ms 계산
        //     [a] 10ms 계산
        //     [b] 10ms 계산
        //     [a] 20ms 계산
        //     [b] 60ms 계산
        //     [a] 끝
        //     [b] 끝
        //     걸린 시간: 약 200ms

        println!("3. 블로킹 코드는 타이머도 늦춥니다");
        let start = Instant::now();
        let ticker = async {
            // 50ms 뒤에 깨워 달라고 했지만, 같은 태스크의 blocker가 차례를 넘기지 않으면 늦게 깨어납니다.
            trpl::sleep(Duration::from_millis(50)).await;
            println!(
                "    [ticker] 50ms 타이머가 약 {}ms에 깨어났습니다",
                rounded_ms(start)
            );
        };
        let blocker = async {
            // await 지점 없이 300ms 동안 스레드를 붙잡습니다.
            slow("blocker", 300);
        };
        trpl::join(ticker, blocker).await;
        // 출력:
        //     [blocker] 300ms 계산
        //     [ticker] 50ms 타이머가 약 300ms에 깨어났습니다
    });
}
