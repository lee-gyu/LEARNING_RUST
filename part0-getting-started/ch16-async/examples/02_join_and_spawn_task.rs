// 여러 퓨처를 동시에 진행시키는 두 가지 방법을 보여 줍니다.
// - trpl::join / trpl::join!: 퓨처 여러 개를 하나로 묶어 모두 끝날 때까지 기다립니다.
// - trpl::spawn_task: 퓨처를 런타임에 맡겨 별도의 태스크(task)로 실행하고, JoinHandle로 결과를 받습니다.
//
// 실행: cargo run -p p0-ch16-async --example 02_join_and_spawn_task
//
// 주의: 걸린 시간은 실행마다 몇 ms씩 달라지므로 100ms 단위로 반올림해 출력합니다.
// 대기 시간을 50ms 이상 벌려 두었기 때문에 줄의 순서는 보통 아래 주석과 같습니다.
// 다만 순서는 타이머에 기대고 있으므로 컴퓨터가 몹시 바쁘면 달라질 수 있습니다.

use std::time::{Duration, Instant};

// 이름을 붙인 가짜 작업입니다. 실제 프로그램이라면 네트워크나 파일을 기다리는 자리입니다.
async fn work(name: &str, ms: u64) -> u64 {
    println!("    [{name}] 시작");
    trpl::sleep(Duration::from_millis(ms)).await;
    println!("    [{name}] 끝 ({ms}ms)");
    ms
}

// 시작 시각부터 지금까지 걸린 시간을 100ms 단위로 반올림합니다.
fn rounded_ms(start: Instant) -> u128 {
    (start.elapsed().as_millis() + 50) / 100 * 100
}

fn main() {
    trpl::block_on(async {
        println!("1. 차례로 await하면 시간이 더해집니다");
        let start = Instant::now();
        let a = work("A", 300).await;
        let b = work("B", 200).await;
        println!("    결과 ({a}, {b}), 걸린 시간: 약 {}ms", rounded_ms(start));
        // 출력:
        //     [A] 시작
        //     [A] 끝 (300ms)
        //     [B] 시작
        //     [B] 끝 (200ms)
        //     결과 (300, 200), 걸린 시간: 약 500ms

        println!("2. join으로 묶으면 함께 기다리므로 가장 긴 쪽만큼 걸립니다");
        let start = Instant::now();
        // join은 두 퓨처를 번갈아 진행시키는 새 퓨처를 만듭니다. A가 sleep에서 멈춘 동안 B를 진행합니다.
        // 결과는 인자 순서대로 튜플에 담깁니다(먼저 끝난 순서가 아닙니다).
        let (a, b) = trpl::join(work("A", 300), work("B", 200)).await;
        println!("    결과 ({a}, {b}), 걸린 시간: 약 {}ms", rounded_ms(start));
        // 출력:
        //     [A] 시작
        //     [B] 시작
        //     [B] 끝 (200ms)
        //     [A] 끝 (300ms)
        //     결과 (300, 200), 걸린 시간: 약 300ms

        println!("3. join!은 개수가 정해진 퓨처 여러 개를 한꺼번에 기다립니다");
        // 출력 타입이 서로 달라도 됩니다. 튜플의 각 자리에 각 퓨처의 결과가 들어갑니다.
        let (ms, text, flag) = trpl::join!(work("C", 100), async { "문자열" }, async { true });
        println!("    결과: ({ms}, {text}, {flag})");
        // 출력:
        //     [C] 시작
        //     [C] 끝 (100ms)
        //     결과: (100, 문자열, true)

        println!("4. spawn_task는 퓨처를 별도의 태스크로 실행합니다");
        let start = Instant::now();
        let name = String::from("배경 작업");
        // 태스크는 이 블록보다 오래 살 수 있으므로 바깥 변수를 빌릴 수 없습니다.
        // thread::spawn에 move 클로저를 넘겼던 것처럼(ch15) async move로 소유권을 넘깁니다.
        let handle = trpl::spawn_task(async move {
            println!("    [{name}] 시작");
            trpl::sleep(Duration::from_millis(200)).await;
            println!("    [{name}] 끝");
            name.len()
        });
        // 일반 퓨처와 달리 태스크는 await하지 않아도 런타임이 곧바로 실행하기 시작합니다.
        trpl::sleep(Duration::from_millis(100)).await;
        println!("    [main] 그동안 100ms짜리 다른 일을 했습니다");
        // JoinHandle도 퓨처입니다. 태스크가 패닉하면 Err가 오므로 출력 타입은 Result입니다.
        let len = handle.await.unwrap();
        println!(
            "    태스크가 돌려준 값: {len}, 걸린 시간: 약 {}ms",
            rounded_ms(start)
        );
        // 출력:
        //     [배경 작업] 시작
        //     [main] 그동안 100ms짜리 다른 일을 했습니다
        //     [배경 작업] 끝
        //     태스크가 돌려준 값: 13, 걸린 시간: 약 200ms
        // ("배경 작업"은 UTF-8로 13바이트입니다.)
    });

    println!("5. block_on이 끝나면 기다리지 않은 태스크는 함께 사라집니다");
    trpl::block_on(async {
        trpl::spawn_task(async {
            trpl::sleep(Duration::from_millis(300)).await;
            println!("    [태스크] 이 줄은 출력되지 않습니다");
        });
        trpl::sleep(Duration::from_millis(50)).await;
        println!("    [main] 50ms 뒤 block_on을 끝냅니다");
    });
    // block_on이 돌아오면 런타임이 정리되면서 아직 끝나지 않은 태스크도 취소됩니다.
    // main이 끝나면 다른 스레드도 끝났던 것(ch15)과 같은 이유로, 끝까지 실행하려면 JoinHandle을 await합니다.
    println!("    block_on이 돌아왔습니다");
    // 출력:
    //     [main] 50ms 뒤 block_on을 끝냅니다
    //     block_on이 돌아왔습니다
}
