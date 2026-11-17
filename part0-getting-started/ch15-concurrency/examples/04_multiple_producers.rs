// 수신자를 반복자로 쓰는 방법과, 송신자를 복제(clone)해 여러 스레드가 한 수신자에게
// 보내는 방법(multiple producer, single consumer)을 보여 줍니다.
// 반복은 "모든 송신자가 drop될 때" 끝난다는 점이 핵심입니다.
//
// 실행: cargo run -p p0-ch15-concurrency --example 04_multiple_producers
//
// 주의: 2번 구역에서 여러 생산자의 메시지가 섞여 도착하는 순서는 실행마다 다를 수 있습니다.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("1. 수신자를 for 문에 넣으면 메시지를 차례로 꺼냅니다");
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for word in ["채널로", "보낸", "메시지", "네 개"] {
            tx.send(word).unwrap();
            thread::sleep(Duration::from_millis(20));
        }
        // 여기서 클로저가 끝나며 tx가 drop됩니다.
    });
    // for 문은 메시지마다 recv를 부르는 것과 같습니다. 메시지가 없으면 기다리고,
    // 송신자가 모두 사라져 채널이 닫히면 반복을 끝냅니다.
    for received in rx {
        println!("    받음: {received}");
    }
    println!("    → 송신자가 사라져서 반복이 끝났습니다");
    // 송신자가 하나뿐이므로 보낸 순서 그대로 도착합니다.
    // 출력:     받음: 채널로
    // 출력:     받음: 보낸
    // 출력:     받음: 메시지
    // 출력:     받음: 네 개
    // 출력:     → 송신자가 사라져서 반복이 끝났습니다

    println!("2. 송신자를 clone해 여러 생산자가 한 수신자에게 보냅니다");
    let (tx, rx) = mpsc::channel();
    for producer in 1..=3 {
        // 스레드마다 자기 송신자가 필요합니다. 복제한 송신자는 같은 채널로 이어집니다.
        let tx = tx.clone();
        thread::spawn(move || {
            for n in 1..=3 {
                tx.send((producer, n)).unwrap();
                thread::sleep(Duration::from_millis(5));
            }
        });
    }
    // 원래 tx는 아무 스레드에도 주지 않았으므로 main이 직접 버려야 합니다.
    // 이 줄을 지우면 main이 tx를 계속 들고 있어서 채널이 닫히지 않고,
    // 아래 for 문은 아홉 개를 다 받은 뒤에도 다음 메시지를 영원히 기다립니다.
    drop(tx);

    let mut received = Vec::new();
    for (producer, n) in rx {
        println!("    생산자 {producer}의 {n}번째 메시지");
        received.push((producer, n));
    }
    // 출력: "생산자 P의 N번째 메시지" 아홉 줄. 생산자끼리 섞이는 순서는 실행마다 다를 수 있습니다.

    println!("    받은 메시지 수: {}", received.len());
    // 생산자가 여럿이어도 같은 생산자가 보낸 메시지끼리는 보낸 순서대로 도착합니다.
    for producer in 1..=3 {
        let order: Vec<i32> = received
            .iter()
            .filter(|(p, _)| *p == producer)
            .map(|(_, n)| *n)
            .collect();
        println!("    생산자 {producer}의 메시지가 도착한 순서: {order:?}");
    }
    // 출력:     받은 메시지 수: 9
    // 출력:     생산자 1의 메시지가 도착한 순서: [1, 2, 3]
    // 출력:     생산자 2의 메시지가 도착한 순서: [1, 2, 3]
    // 출력:     생산자 3의 메시지가 도착한 순서: [1, 2, 3]

    println!("3. 송신자가 하나라도 남아 있으면 채널은 닫히지 않습니다");
    // for 문 대신 recv_timeout(최대 대기 시간을 정한 recv)으로 받아서 영원히 기다리는 일을 피합니다.
    let (tx, rx) = mpsc::channel();
    let worker_tx = tx.clone();
    thread::spawn(move || {
        worker_tx.send("유일한 메시지").unwrap();
    })
    .join()
    .unwrap(); // 작업 스레드는 끝났고, 그 송신자(worker_tx)도 drop되었습니다.

    let wait = Duration::from_millis(100);
    println!("    {:?}", rx.recv_timeout(wait));
    // main의 tx가 아직 살아 있으므로 "닫힘"이 아니라 "아직 안 옴(Timeout)"입니다.
    // for 문이었다면 여기서 영원히 멈췄을 것입니다.
    println!("    {:?}", rx.recv_timeout(wait));
    drop(tx); // 마지막 송신자를 버리면 그제야 채널이 닫힙니다.
    println!("    {:?}", rx.recv_timeout(wait));
    // 출력:     Ok("유일한 메시지")
    // 출력:     Err(Timeout)
    // 출력:     Err(Disconnected)
}
