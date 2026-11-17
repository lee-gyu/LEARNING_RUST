// 기대하는 에러: E0310 the parameter type `F` may not live long enough
//                 the parameter type `F` must be valid for the static lifetime...
//
// 확인: rustc --edition 2024 compile_fail/e0310_execute_without_static.rs --out-dir /tmp/rust-ch20
//
// execute의 바운드에서 'static을 빠뜨렸습니다. 작업은 채널을 거쳐 작업자 스레드로 가고,
// 작업자가 언제 실행할지는 알 수 없습니다. 그래서 채널에 넣는 Job은 'static이어야 하는데,
// F가 'static이라는 약속이 없으면 지역 변수를 빌린 클로저도 들어올 수 있습니다.
// 고치는 법: where F: FnOnce() + Send + 'static (컴파일러의 help가 정확히 이 수정을 제안합니다)

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

fn main() {
    let (sender, receiver) = mpsc::channel::<Job>();
    let receiver = Arc::new(Mutex::new(receiver));
    let worker = thread::spawn(move || {
        loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => job(),
                Err(_) => break,
            }
        }
    });

    let pool = ThreadPool { sender };
    pool.execute(|| println!("작업 실행"));
    drop(pool);
    worker.join().unwrap();
}
