// 작업자 스레드의 반복문을 어떻게 쓰느냐에 따라 잠금이 풀리는 시점이 달라집니다. (The Book 21.2, Listing 21-20·21-21)
//
// 실행: cargo run -p p0-ch20-web-server-project --example 04_lock_scope
//
// 스레드 풀의 핵심 구조(채널 + Arc<Mutex<Receiver<Job>>> + 작업자 스레드 2개)만 떼어 와서,
// 0.2초짜리 작업 4개를 맡깁니다. 작업자 둘이 동시에 일하면 0.4초, 하나씩 일하면 0.8초가 걸립니다.
// 네트워크는 쓰지 않습니다. 걸린 시간은 0.1초 단위로 반올림해 출력하며, 컴퓨터가 바쁘면 조금 다를 수 있습니다.

use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

type Job = Box<dyn FnOnce() + Send + 'static>;
type SharedReceiver = Arc<Mutex<mpsc::Receiver<Job>>>;

fn main() {
    println!("1) let 문으로 받기");
    run(worker_with_let);
    // 출력: 1) let 문으로 받기
    // 출력:     동시에 실행된 작업 수(최대): 2, 걸린 시간: 0.4초

    println!("2) while let으로 받기");
    run(worker_with_while_let);
    // 출력: 2) while let으로 받기
    // 출력:     동시에 실행된 작업 수(최대): 1, 걸린 시간: 0.8초

    println!("3) match의 조사 대상에서 바로 받기");
    run(worker_with_match);
    // 출력: 3) match의 조사 대상에서 바로 받기
    // 출력:     동시에 실행된 작업 수(최대): 1, 걸린 시간: 0.8초
}

/// [1] 완성판의 작업자 반복문입니다.
/// receiver.lock()이 돌려준 MutexGuard는 이름 없는 임시값입니다. let 문에 쓰인 임시값은
/// 그 문장의 세미콜론에서 drop되므로, job()을 실행하기 전에 잠금이 풀립니다.
fn worker_with_let(receiver: SharedReceiver) {
    loop {
        let message = receiver.lock().unwrap().recv(); // ← 여기서 잠금이 풀립니다.
        match message {
            Ok(job) => job(), // 잠금 없이 실행하므로 그동안 다른 작업자가 다음 작업을 받습니다.
            Err(_) => break,  // 송신자가 모두 사라지고 남은 작업도 없으면 끝냅니다.
        }
    }
}

/// [2] Listing 21-21의 반복문입니다. 컴파일도 되고 결과도 맞지만 작업이 하나씩만 실행됩니다.
/// while let의 조사 대상(= 오른쪽 표현식)에서 생긴 임시값은 반복문 본문이 끝날 때까지 살아 있습니다.
/// 그래서 job()을 실행하는 내내 MutexGuard가 잠금을 쥐고 있고, 다른 작업자는 lock()에서 기다립니다.
fn worker_with_while_let(receiver: SharedReceiver) {
    while let Ok(job) = receiver.lock().unwrap().recv() {
        job(); // ← 아직 잠금을 쥐고 있습니다.
    } // ← 여기서야 잠금이 풀립니다.
}

/// [3] match도 마찬가지입니다. 조사 대상의 임시값은 match 전체가 끝날 때까지 살아 있습니다.
/// (if let도 then 블록이 끝날 때까지 살아 있어 같은 결과가 나옵니다.)
// clippy는 이 모양을 보면 "while let으로 쓸 수 있다"(while_let_loop)고 알려 줍니다. 그렇게 바꿔도
// [2]와 같아질 뿐 잠금 문제는 그대로입니다. 비교하려고 일부러 이 모양으로 두었으므로 경고를 예상된 것으로 표시합니다.
#[expect(
    clippy::while_let_loop,
    reason = "match 조사 대상의 임시값 수명을 보여 주려는 코드"
)]
fn worker_with_match(receiver: SharedReceiver) {
    loop {
        match receiver.lock().unwrap().recv() {
            Ok(job) => job(), // ← 아직 잠금을 쥐고 있습니다.
            Err(_) => break,
        }
    }
}

/// 작업자 스레드 2개를 띄우고 0.2초짜리 작업 4개를 보낸 뒤, 동시에 실행된 작업 수의 최댓값과
/// 전체 걸린 시간을 출력합니다. worker는 위의 세 함수 중 하나입니다(함수 포인터, ch19).
fn run(worker: fn(SharedReceiver)) {
    let (sender, receiver) = mpsc::channel::<Job>();
    // Receiver는 복제할 수 없으므로(mpsc = 수신자 하나) Arc로 함께 소유하고, Mutex로 한 번에
    // 한 작업자만 꺼내게 합니다.
    let receiver = Arc::new(Mutex::new(receiver));

    let start = Instant::now();
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let receiver = Arc::clone(&receiver);
            thread::spawn(move || worker(receiver))
        })
        .collect();

    // (지금 실행 중인 작업 수, 그 최댓값)
    let running = Arc::new(Mutex::new((0, 0)));
    for _ in 0..4 {
        let running = Arc::clone(&running);
        let job: Job = Box::new(move || {
            {
                let mut counts = running.lock().unwrap();
                counts.0 += 1;
                counts.1 = counts.1.max(counts.0);
            }
            thread::sleep(Duration::from_millis(200));
            running.lock().unwrap().0 -= 1;
        });
        sender.send(job).unwrap();
    }

    // 송신자를 버려 채널을 닫아야 작업자의 recv()가 Err를 돌려주고 반복문이 끝납니다.
    drop(sender);
    for handle in workers {
        handle.join().unwrap();
    }

    let max_running = running.lock().unwrap().1;
    let elapsed = (start.elapsed().as_secs_f64() * 10.0).round() / 10.0;
    println!("    동시에 실행된 작업 수(최대): {max_running}, 걸린 시간: {elapsed:.1}초");
}
