// 기대하는 에러: E0507 cannot move out of `worker.thread` which is behind a mutable reference
//                 `std::thread::JoinHandle::<T>::join` takes ownership of the receiver `self`, which moves `worker.thread`
//
// 확인: rustc --edition 2024 compile_fail/e0507_join_in_drop.rs --out-dir /tmp/rust-ch20
//
// ThreadPool의 Drop에서 작업자 스레드를 join하려는 첫 시도입니다. (The Book Listing 21-22)
// join(self)는 JoinHandle의 소유권을 가져가는데, drop은 &mut self만 받으므로
// &mut self.workers를 돌며 얻은 worker도 &mut Worker입니다. 빌린 것에서 필드를 꺼내 옮길 수는 없습니다.
// 고치는 법: for worker in self.workers.drain(..)로 벡터를 비우며 Worker를 값으로 꺼냅니다.
//           (또는 thread 필드를 Option<JoinHandle<()>>로 바꾸고 take()로 꺼냅니다.)

use std::thread;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

struct ThreadPool {
    workers: Vec<Worker>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("작업자 {}의 종료를 기다립니다", worker.id);
            worker.thread.join().unwrap();
        }
    }
}

fn main() {
    let mut workers = Vec::new();
    for id in 0..2 {
        let thread = thread::spawn(move || println!("작업자 {id} 실행"));
        workers.push(Worker { id, thread });
    }
    let _pool = ThreadPool { workers };
}
