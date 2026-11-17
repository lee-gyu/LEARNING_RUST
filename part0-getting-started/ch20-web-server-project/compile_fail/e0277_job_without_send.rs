// 기대하는 에러: E0277 `(dyn FnOnce() + 'static)` cannot be sent between threads safely
//                 the trait `Send` is not implemented for `(dyn FnOnce() + 'static)`
//
// 확인: rustc --edition 2024 compile_fail/e0277_job_without_send.rs --out-dir /tmp/rust-ch20
//
// 작업 타입 별칭 Job에서 Send를 빠뜨렸습니다. 작업자 스레드는 Arc<Mutex<Receiver<Job>>>를
// 캡처하는데, 이 타입이 Send이려면 결국 Job 안의 dyn FnOnce()가 Send여야 합니다.
// "어떤 클로저든 들어올 수 있는 상자"에 Send가 약속되어 있지 않으면, 그 상자를 꺼낼 수 있는
// 수신자를 다른 스레드로 보낼 수 없습니다.
// 고치는 법: type Job = Box<dyn FnOnce() + Send + 'static>;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + 'static>;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => job(),
                    Err(_) => break,
                }
            }
        });
        Worker { id, thread }
    }
}

fn main() {
    let (sender, receiver) = mpsc::channel::<Job>();
    let receiver = Arc::new(Mutex::new(receiver));
    let worker = Worker::new(0, Arc::clone(&receiver));

    sender.send(Box::new(|| println!("작업 실행"))).unwrap();
    drop(sender);

    println!("작업자 {}의 종료를 기다립니다", worker.id);
    worker.thread.join().unwrap();
}
