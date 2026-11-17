// 기대하는 에러: E0277
// error[E0277]: `RefCell<i32>` cannot be shared between threads safely
//   = help: the trait `Sync` is not implemented for `RefCell<i32>`
//
// 확인: rustc --edition 2024 compile_fail/e0277_refcell_is_not_sync.rs
//
// Arc로 Rc 문제를 해결했으니 이번에는 Mutex 대신 RefCell을 넣어 봅니다.
// RefCell은 "지금 빌려 간 사람이 있는가"를 평범한 정수로 세고, 한 스레드에서만 쓰인다고
// 가정합니다. 두 스레드가 동시에 borrow_mut를 부르면 둘 다 "아무도 안 빌렸다"고 보고
// 가변 참조를 하나씩 얻을 수 있습니다. 그것이 바로 데이터 경쟁입니다. 그래서 RefCell은
// Sync가 아니고, Arc<RefCell<i32>>는 다른 스레드로 보낼 수 없습니다.

use std::cell::RefCell;
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(RefCell::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            *counter.borrow_mut() += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("결과: {}", counter.borrow());
}
