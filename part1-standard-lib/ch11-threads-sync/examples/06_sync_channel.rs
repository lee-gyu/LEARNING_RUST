// 채널 넓히기: 용량이 정해진 채널(sync_channel)과 배압, 용량 0인 랑데부 채널,
// 기다리지 않고 꺼내는 try_iter, 여러 스코프 스레드가 송신자 하나를 &로 함께 쓰기.
//
// 실행: cargo run -p p1-ch11-threads-sync --example 06_sync_channel
//
// 2·3번 구역은 main이 잠깐(50ms) 잔 뒤에 생산자의 진행 상황을 확인합니다.
// 생산자가 어디서 멈춰 있는지는 채널의 용량이 정하므로 결과는 실행할 때마다 같습니다.

use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// 생산자가 send를 몇 번 마쳤는지 50ms 뒤에 확인하고, 그다음 모두 받아 합계를 돌려줍니다.
fn count_after_pause(sender: impl Fn(u32) + Send, receiver: &mpsc::Receiver<u32>) -> (usize, u32) {
    let sent = Mutex::new(0);
    thread::scope(|s| {
        let sent = &sent;
        // sender(송신자를 캡처한 클로저)는 생산자 스레드로 옮기고, sent는 참조로 빌려 줍니다.
        s.spawn(move || {
            for n in 1..=10 {
                sender(n); // 용량이 찼으면 여기서 기다립니다.
                *sent.lock().unwrap() += 1;
            }
        });
        thread::sleep(Duration::from_millis(50)); // 그동안 아무도 받지 않습니다.
        let after_pause = *sent.lock().unwrap();
        let total: u32 = receiver.iter().take(10).sum(); // 이제 10개를 모두 받습니다.
        (after_pause, total)
    })
}

fn main() {
    println!("1. sync_channel(2): 용량이 다 차면 try_send는 Full을 돌려줍니다");
    {
        let (tx, rx) = mpsc::sync_channel::<i32>(2);
        println!("    try_send(1): {:?}", tx.try_send(1));
        println!("    try_send(2): {:?}", tx.try_send(2));
        println!("    try_send(3): {:?}", tx.try_send(3)); // 자리가 없음
        println!("    recv():      {:?}", rx.recv()); // 하나 꺼내면 자리가 생김
        println!("    try_send(3): {:?}", tx.try_send(3));
        drop(rx);
        println!("    수신자를 버린 뒤 try_send(4): {:?}", tx.try_send(4));
        // 출력: try_send(1): Ok(())
        // 출력: try_send(2): Ok(())
        // 출력: try_send(3): Err(TrySendError::Full(..))
        // 출력: recv():      Ok(1)
        // 출력: try_send(3): Ok(())
        // 출력: 수신자를 버린 뒤 try_send(4): Err(TrySendError::Disconnected(..))
    }

    println!("2. 배압: 받는 쪽이 느리면 보내는 쪽이 기다립니다");
    {
        // channel(): 용량 제한이 없으므로 받는 쪽이 멈춰 있어도 10개를 모두 보냅니다.
        let (tx, rx) = mpsc::channel();
        let (sent, total) = count_after_pause(move |n| tx.send(n).unwrap(), &rx);
        println!("    channel():         50ms 동안 보낸 개수 {sent:>2}, 받은 합계 {total}");

        // sync_channel(3): 3개가 쌓이면 4번째 send에서 멈춥니다.
        let (tx, rx) = mpsc::sync_channel(3);
        let (sent, total) = count_after_pause(move |n| tx.send(n).unwrap(), &rx);
        println!("    sync_channel(3):   50ms 동안 보낸 개수 {sent:>2}, 받은 합계 {total}");
        // 출력: channel():         50ms 동안 보낸 개수 10, 받은 합계 55
        // 출력: sync_channel(3):   50ms 동안 보낸 개수  3, 받은 합계 55
    }

    println!("3. sync_channel(0): 받는 쪽이 받을 때까지 send가 끝나지 않습니다(랑데부)");
    {
        let (tx, rx) = mpsc::sync_channel(0);
        println!("    아무도 기다리지 않을 때 try_send: {:?}", tx.try_send(0));
        let (sent, total) = count_after_pause(move |n| tx.send(n).unwrap(), &rx);
        println!("    50ms 동안 보낸 개수 {sent}, 받은 합계 {total}");
        // 출력: 아무도 기다리지 않을 때 try_send: Err(TrySendError::Full(..))
        // 출력: 50ms 동안 보낸 개수 0, 받은 합계 55
    }

    println!("4. try_iter는 지금 쌓여 있는 것만 꺼내고 기다리지 않습니다");
    {
        let (tx, rx) = mpsc::channel();
        for n in [5, 6, 7] {
            tx.send(n).unwrap();
        }
        // 송신자 tx가 아직 살아 있으므로 iter()였다면 7 다음에서 영원히 기다립니다.
        let pending: Vec<i32> = rx.try_iter().collect();
        println!(
            "    try_iter로 꺼낸 값: {pending:?}, 남은 것: {:?}",
            rx.try_recv()
        );
        // 출력: try_iter로 꺼낸 값: [5, 6, 7], 남은 것: Err(Empty)
    }

    println!("5. Sender는 Sync라서 스코프 스레드들이 &tx로 함께 보낼 수 있습니다");
    {
        let (tx, rx) = mpsc::channel();
        thread::scope(|s| {
            for worker in 1..=3 {
                let tx = &tx; // 복제(clone) 없이 참조만 나눠 줍니다.
                s.spawn(move || tx.send(worker * worker).unwrap());
            }
        });
        drop(tx); // 마지막 송신자를 버려야 아래 for 문이 끝납니다.
        let mut squares: Vec<i32> = rx.iter().collect();
        squares.sort(); // 도착 순서는 실행마다 다를 수 있습니다.
        println!("    받은 값(정렬): {squares:?}");
        // 출력: 받은 값(정렬): [1, 4, 9]
    }
}
