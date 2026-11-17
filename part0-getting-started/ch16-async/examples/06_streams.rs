// 스트림(stream), 즉 시간에 걸쳐 값을 하나씩 내놓는 "비동기 반복자"를 보여 줍니다.
// - 반복자의 next()는 값을 바로 돌려주지만, 스트림의 next()는 퓨처라서 .await로 기다립니다.
// - next, filter, map, collect 같은 메서드는 StreamExt 트레이트에 있으므로 use trpl::StreamExt;가 필요합니다.
// - 채널 수신자를 ReceiverStream으로 감싸면 메시지가 도착할 때마다 하나씩 나오는 스트림이 됩니다.
// - timeout처럼 시간을 다루는 어댑터도 있습니다. 이런 스트림은 Unpin이 아니어서 pin!으로 고정한 뒤 next를 부릅니다.
//
// 실행: cargo run -p p0-ch16-async --example 06_streams
//
// 주의: 3번의 시각은 100ms 단위로 반올림해 출력합니다. 3·4번은 타이머로 정해지는 출력이라
// 보통 아래 주석과 같지만, 컴퓨터가 몹시 바쁘면 달라질 수 있습니다.

use std::pin::pin;
use std::time::{Duration, Instant};

use trpl::{ReceiverStream, StreamExt};

fn rounded_ms(start: Instant) -> u128 {
    (start.elapsed().as_millis() + 50) / 100 * 100
}

fn main() {
    trpl::block_on(async {
        println!("1. 반복자로 스트림을 만들고 next().await로 하나씩 꺼냅니다");
        let mut stream = trpl::stream_from_iter(["사과", "배", "감"]);
        // next()는 Option을 내놓는 퓨처입니다. 반복자처럼 None이 나오면 끝입니다.
        while let Some(fruit) = stream.next().await {
            println!("    {fruit}");
        }
        // 출력:
        //     사과
        //     배
        //     감

        println!("2. 반복자처럼 어댑터를 이어 붙이고 collect로 모읍니다");
        // filter의 클로저는 항목의 참조(&i32)를, map의 클로저는 항목 자체(i32)를 받습니다.
        // collect도 퓨처를 돌려주므로 .await해야 Vec이 나옵니다.
        let squares: Vec<i32> = trpl::stream_from_iter(1..=10)
            .filter(|n| n % 2 == 0)
            .map(|n| n * n)
            .collect()
            .await;
        println!("    짝수의 제곱: {squares:?}");
        // 출력:
        //     짝수의 제곱: [4, 16, 36, 64, 100]

        println!("3. 채널 수신자를 스트림으로 감싸면 메시지가 도착하는 대로 나옵니다");
        let (tx, rx) = trpl::channel();
        trpl::spawn_task(async move {
            for word in ["하나", "둘", "셋"] {
                trpl::sleep(Duration::from_millis(100)).await;
                tx.send(word).unwrap();
            }
            // 태스크가 끝나면 tx가 버려지고, 스트림도 None으로 끝납니다.
        });
        let start = Instant::now();
        let mut messages = ReceiverStream::new(rx);
        while let Some(word) = messages.next().await {
            println!("    약 {}ms에 도착: {word}", rounded_ms(start));
        }
        println!("    송신자가 버려져 스트림이 끝났습니다");
        // 출력:
        //     약 100ms에 도착: 하나
        //     약 200ms에 도착: 둘
        //     약 300ms에 도착: 셋
        //     송신자가 버려져 스트림이 끝났습니다

        println!("4. timeout 어댑터는 다음 항목이 늦으면 Err를 내놓고 계속 기다립니다");
        let (tx, rx) = trpl::channel();
        trpl::spawn_task(async move {
            for (word, delay) in [
                ("빠른 메시지 1", 50),
                ("늦은 메시지", 300),
                ("빠른 메시지 2", 50),
            ] {
                trpl::sleep(Duration::from_millis(delay)).await;
                tx.send(word).unwrap();
            }
        });
        // timeout 스트림은 안에 타이머 퓨처를 품고 있어 Unpin이 아닙니다. next()는 Self: Unpin을 요구하므로
        // pin!으로 이 자리에 고정해 Pin<&mut ...>으로 만든 뒤 씁니다(Pin과 Unpin은 07 예제에서 다룹니다).
        let mut messages = pin!(ReceiverStream::new(rx).timeout(Duration::from_millis(200)));
        while let Some(result) = messages.next().await {
            match result {
                Ok(word) => println!("    받음: {word}"),
                Err(_) => println!("    200ms 동안 아무것도 오지 않았습니다"),
            }
        }
        // 출력:
        //     받음: 빠른 메시지 1
        //     200ms 동안 아무것도 오지 않았습니다
        //     받음: 늦은 메시지
        //     받음: 빠른 메시지 2
    });
}
