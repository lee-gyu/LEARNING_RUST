// 기대하는 에러: E0277
// 핵심 메시지: `std::sync::MutexGuard<'_, Vec<&str>>` cannot be sent between threads safely
//
// main 스레드에서 잠근 MutexGuard를 다른 스레드로 넘겨 그 스레드가 쓰고 풀게 하려 합니다.
// MutexGuard는 Send가 아니므로, 잠근 스레드가 직접 풀어야 합니다.
//
// 확인: rustc --edition 2024 compile_fail/e0277_mutex_guard_not_send.rs --out-dir /tmp/rust-p1-ch11

use std::sync::Mutex;
use std::thread;

fn main() {
    let log = Mutex::new(Vec::new());
    let mut guard = log.lock().unwrap(); // main 스레드가 잠금

    thread::scope(|s| {
        // 가드를 작업 스레드로 옮겨, 거기서 쓰고 풀게 하려 함 → E0277
        s.spawn(move || guard.push("작업자가 기록"));
    });

    println!("{:?}", log.lock().unwrap());
}
