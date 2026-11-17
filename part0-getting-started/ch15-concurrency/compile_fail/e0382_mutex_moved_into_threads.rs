// 기대하는 에러: E0382
// error[E0382]: borrow of moved value: `counter`
//   ... value moved into closure here, in previous iteration of loop
//
// 확인: rustc --edition 2024 compile_fail/e0382_mutex_moved_into_threads.rs
//
// Mutex는 여러 스레드가 번갈아 쓰게 해 주지만, Mutex 값 자체의 소유자는 여전히 하나입니다.
// 반복문의 첫 바퀴에서 counter가 첫 번째 스레드의 클로저로 이동하므로, 두 번째 바퀴부터는
// 옮길 counter가 남아 있지 않습니다. 여러 스레드가 함께 소유하려면 Arc가 필요합니다.

use std::sync::Mutex;
use std::thread;

fn main() {
    let counter = Mutex::new(0);
    let mut handles = vec![];

    for _ in 0..10 {
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("결과: {}", *counter.lock().unwrap());
}
