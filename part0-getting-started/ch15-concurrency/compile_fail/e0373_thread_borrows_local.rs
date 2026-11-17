// 기대하는 에러: E0373
// error[E0373]: closure may outlive the current function, but it borrows `v`,
//               which is owned by the current function
//
// 확인: rustc --edition 2024 compile_fail/e0373_thread_borrows_local.rs
//
// 클로저 안의 println!은 v를 읽기만 하므로, Rust는 v를 빌리는(&v) 쪽으로 캡처합니다.
// 그런데 새 스레드가 언제까지 실행될지는 컴파일러도 알 수 없습니다. 만약 main이 먼저
// v를 버린다면(drop) 새 스레드는 해제된 메모리를 읽게 됩니다. 그래서 거부합니다.

use std::thread;

fn main() {
    let v = vec![1, 2, 3];

    let handle = thread::spawn(|| {
        println!("새 스레드가 받은 벡터: {v:?}");
    });

    handle.join().unwrap();
}
