// 기대하는 에러: E0277
// 핵심 메시지: `std::sync::mpsc::Receiver<i32>` cannot be shared between threads safely
//
// 작업자 두 명이 수신자 하나를 &로 함께 써서 일감을 나눠 받으려 합니다.
// mpsc의 Receiver는 소비자가 하나뿐이라는 전제로 만들어져 Sync가 아닙니다.
//
// 확인: rustc --edition 2024 compile_fail/e0277_receiver_not_sync.rs --out-dir /tmp/rust-p1-ch11

use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    for job in 1..=6 {
        tx.send(job).unwrap();
    }
    drop(tx);

    let total: i32 = thread::scope(|s| {
        // 두 스코프 스레드가 rx를 동시에 빌림 → E0277
        let a = s.spawn(|| rx.iter().take(3).sum::<i32>());
        let b = s.spawn(|| rx.iter().sum::<i32>());
        a.join().unwrap() + b.join().unwrap()
    });

    println!("합계: {total}");
}
