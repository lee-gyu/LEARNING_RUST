// 비동기 채널(trpl::channel)로 퓨처 사이에 메시지를 주고받는 방법을 보여 줍니다.
// - send는 기다리지 않고 바로 넣습니다(크기 제한이 없는 채널). recv는 메시지가 올 때까지 기다리는 퓨처입니다.
// - recv는 모든 송신자가 버려져 채널이 닫혀야 None을 돌려줍니다. 송신 퓨처를 async move로 만드는 이유입니다.
// - tx.clone()으로 송신자를 늘리면 여러 생산자가 한 수신자에게 보낼 수 있습니다(ch15의 mpsc 채널과 같습니다).
//
// 실행: cargo run -p p0-ch16-async --example 04_async_channel
//
// 주의: 4번에서 두 생산자의 메시지가 섞이는 순서는 타이머로 정해집니다. 보내는 시각을 50ms씩 벌려 두어
// 보통 아래 주석과 같지만, 컴퓨터가 몹시 바쁘면 달라질 수 있습니다.

use std::time::Duration;

use trpl::Either;

fn main() {
    trpl::block_on(async {
        println!("1. send는 바로 넣고, recv는 .await로 기다립니다");
        // tx는 송신자(Sender), rx는 수신자(Receiver)입니다. recv가 &mut self를 받으므로 rx는 mut입니다.
        let (tx, mut rx) = trpl::channel();
        tx.send(String::from("안녕")).unwrap();
        // recv().await는 Option을 돌려줍니다. 메시지가 있으면 Some, 채널이 닫혔으면 None입니다.
        let received = rx.recv().await.unwrap();
        println!("    받음: {received}");
        // 출력:
        //     받음: 안녕

        println!("2. 보내는 퓨처와 받는 퓨처를 join하면 번갈아 진행됩니다");
        let (tx, mut rx) = trpl::channel();
        // async move: tx의 소유권이 이 블록으로 들어옵니다. 블록이 끝나면 tx가 버려지고 채널이 닫힙니다.
        let tx_fut = async move {
            for message in ["하나", "둘", "셋"] {
                println!("    보냄: {message}");
                tx.send(message).unwrap();
                trpl::sleep(Duration::from_millis(100)).await;
            }
        };
        let rx_fut = async {
            // while let: Some이 오는 동안 반복하고, None(채널 닫힘)이 오면 끝냅니다.
            while let Some(message) = rx.recv().await {
                println!("    받음: {message}");
            }
            println!("    채널이 닫혀 수신 루프가 끝났습니다");
        };
        trpl::join(tx_fut, rx_fut).await;
        // 출력:
        //     보냄: 하나
        //     받음: 하나
        //     보냄: 둘
        //     받음: 둘
        //     보냄: 셋
        //     받음: 셋
        //     채널이 닫혀 수신 루프가 끝났습니다

        println!("3. move를 빼면 tx가 살아 있어 수신 루프가 끝나지 않습니다");
        let (tx, mut rx) = trpl::channel();
        // move가 없으므로 이 블록은 tx를 빌리기만 합니다. 블록이 끝나도 tx는 바깥 블록에 그대로 남습니다.
        let tx_fut = async {
            for message in ["하나", "둘"] {
                tx.send(message).unwrap();
                trpl::sleep(Duration::from_millis(100)).await;
            }
        };
        let rx_fut = async {
            while let Some(message) = rx.recv().await {
                println!("    받음: {message}");
            }
            println!("    이 줄은 출력되지 않습니다");
        };
        // 그대로 await하면 프로그램이 영원히 멈추므로, 03 예제의 timeout처럼 500ms 제한을 겁니다.
        match trpl::select(
            trpl::join(tx_fut, rx_fut),
            trpl::sleep(Duration::from_millis(500)),
        )
        .await
        {
            Either::Left(_) => println!("    정상적으로 끝났습니다"),
            Either::Right(()) => println!("    500ms가 지나도 끝나지 않아 포기했습니다"),
        }
        // 출력:
        //     받음: 하나
        //     받음: 둘
        //     500ms가 지나도 끝나지 않아 포기했습니다

        println!("4. tx.clone()으로 생산자를 둘로 늘립니다");
        let (tx, mut rx) = trpl::channel();
        let tx1 = tx.clone();
        // 두 생산자 모두 async move여야 합니다. 송신자가 하나라도 남아 있으면 채널이 닫히지 않습니다.
        let producer_a = async move {
            for i in 1..=3 {
                tx1.send(format!("A{i}")).unwrap();
                trpl::sleep(Duration::from_millis(100)).await;
            }
        };
        let producer_b = async move {
            // A보다 50ms 늦게 시작해서, A와 B가 50ms 간격으로 번갈아 보내게 합니다.
            trpl::sleep(Duration::from_millis(50)).await;
            for i in 1..=3 {
                tx.send(format!("B{i}")).unwrap();
                trpl::sleep(Duration::from_millis(100)).await;
            }
        };
        let consumer = async {
            let mut received = Vec::new();
            while let Some(message) = rx.recv().await {
                received.push(message);
            }
            println!("    받은 순서: {received:?}");
        };
        // 개수가 셋 이상이면 join! 매크로를 씁니다.
        trpl::join!(producer_a, producer_b, consumer);
        // 출력:
        //     받은 순서: ["A1", "B1", "A2", "B2", "A3", "B3"]
    });
}
