// 채널(mpsc::channel)로 스레드 사이에 값을 보내는 기본 사용법을 보여 줍니다.
// - send는 값의 소유권을 받는 쪽으로 옮깁니다.
// - recv는 값이 올 때까지 기다리고, try_recv는 기다리지 않습니다.
// - 채널의 한쪽 끝이 사라지면 다른 쪽 끝은 Err를 받습니다.
//
// 실행: cargo run -p p0-ch15-concurrency --example 03_channel_basics

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("1. 채널을 만들고 값 하나를 주고받습니다");
    // channel()은 (송신자, 수신자) 튜플을 돌려줍니다. tx는 transmitter, rx는 receiver의 약자입니다.
    // 보낼 값의 타입은 아래에서 send하는 값을 보고 컴파일러가 추론합니다(여기서는 String).
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // 보내려면 송신자를 가지고 있어야 하므로 move로 tx를 새 스레드에 넘겼습니다.
        tx.send(String::from("안녕하세요")).unwrap();
    });
    // recv는 값이 도착할 때까지 main을 멈춰 세웁니다.
    let received = rx.recv().unwrap();
    println!("    받은 값: {received}");
    // 출력:     받은 값: 안녕하세요

    println!("2. send는 값을 복사하지 않고 소유권을 옮깁니다");
    let (tx, rx) = mpsc::channel();
    let message = String::from("힙에 있는 글자");
    let heap_address = message.as_ptr(); // 보내기 전 힙 데이터의 위치를 기록해 둡니다.
    thread::spawn(move || {
        tx.send(message).unwrap();
        // 여기서 message를 다시 쓰면 E0382입니다(compile_fail/e0382_use_after_send.rs).
    });
    let received = rx.recv().unwrap();
    // 받은 String이 같은 힙 데이터를 가리킨다면, 글자를 복사한 것이 아니라 주인만 바뀐 것입니다.
    println!("    받은 값: {received}");
    println!(
        "    보낸 값과 같은 힙 데이터인가? {}",
        received.as_ptr() == heap_address
    );
    // 출력:     받은 값: 힙에 있는 글자
    // 출력:     보낸 값과 같은 힙 데이터인가? true

    println!("3. recv는 값이 올 때까지 기다립니다");
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100)); // 일부러 늦게 보냅니다.
        tx.send(42).unwrap();
    });
    let value = rx.recv().unwrap();
    // 보내는 쪽이 100ms를 잔 뒤에 보냈으므로, 받기까지 최소 100ms가 걸렸을 수밖에 없습니다.
    println!("    받은 값: {value}");
    println!(
        "    100ms 이상 기다렸는가? {}",
        start.elapsed() >= Duration::from_millis(100)
    );
    // 출력:     받은 값: 42
    // 출력:     100ms 이상 기다렸는가? true

    println!("4. 송신자가 모두 사라지면 recv는 Err를 돌려줍니다");
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        tx.send("첫째").unwrap();
        tx.send("둘째").unwrap();
    }); // 클로저가 끝나면서 tx가 drop됩니다. 이제 이 채널에는 송신자가 없습니다.
    handle.join().unwrap(); // 송신자가 확실히 사라진 뒤에 받아 봅니다.
    // 이미 도착한 메시지는 송신자가 사라져도 그대로 받을 수 있습니다.
    println!("    {:?}", rx.recv());
    println!("    {:?}", rx.recv());
    // 남은 메시지도 없고 보낼 사람도 없으니, 기다리지 않고 Err를 돌려줍니다.
    println!("    {:?}", rx.recv());
    // 출력:     Ok("첫째")
    // 출력:     Ok("둘째")
    // 출력:     Err(RecvError)

    println!("5. 수신자가 사라지면 send가 Err를 돌려주고, 보내려던 값을 되돌려줍니다");
    let (tx, rx) = mpsc::channel();
    drop(rx);
    match tx.send(String::from("받을 사람이 없는 편지")) {
        Ok(()) => println!("    보냈습니다"),
        // SendError의 .0에 보내려던 값이 들어 있습니다. 받을 쪽이 없으니 소유권이 돌아온 것입니다.
        Err(error) => println!("    {error}: 돌려받은 값 = {}", error.0),
    }
    // 출력:     sending on a closed channel: 돌려받은 값 = 받을 사람이 없는 편지

    println!("6. try_recv는 기다리지 않고 지금 상태를 바로 알려 줍니다");
    // 스레드 없이 한 스레드 안에서 보내고 받아 봅니다. send는 기다리지 않고 바로 돌아오므로 가능합니다.
    let (tx, rx) = mpsc::channel();
    println!("    아직 아무것도 안 보냄: {:?}", rx.try_recv());
    tx.send(7).unwrap();
    println!("    하나 보낸 뒤:        {:?}", rx.try_recv());
    drop(tx);
    println!("    송신자를 버린 뒤:    {:?}", rx.try_recv());
    // 출력:     아직 아무것도 안 보냄: Err(Empty)
    // 출력:     하나 보낸 뒤:        Ok(7)
    // 출력:     송신자를 버린 뒤:    Err(Disconnected)

    println!("7. try_recv로 기다리는 동안 다른 일을 할 수 있습니다");
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        tx.send("작업 완료").unwrap();
    });
    let mut other_work = 0;
    let result = loop {
        match rx.try_recv() {
            Ok(message) => break message,
            // Empty: 아직 안 왔으니 다른 일을 조금 하고 다시 확인합니다.
            Err(mpsc::TryRecvError::Empty) => {
                other_work += 1;
                thread::sleep(Duration::from_millis(10));
            }
            // Disconnected: 앞으로도 올 일이 없으니 기다림을 그만둡니다.
            Err(mpsc::TryRecvError::Disconnected) => break "송신자가 사라짐",
        }
    };
    println!("    받은 값: {result}");
    println!("    기다리는 동안 다른 일을 {other_work}번 했습니다");
    // 출력:     받은 값: 작업 완료
    // 출력:     기다리는 동안 다른 일을 N번 했습니다
    //       (N은 실행마다 다를 수 있습니다. 이 환경에서 여러 번 실행했을 때는 4나 5였습니다.)
}
