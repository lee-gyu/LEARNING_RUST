// 기대하는 에러: E0382 use of moved value: `receiver`
//                 value moved here, in previous iteration of loop
//
// 확인: rustc --edition 2024 compile_fail/e0382_receiver_moved_into_workers.rs --out-dir /tmp/rust-ch20
//
// 스레드 풀을 만들 때 채널의 수신자를 작업자마다 그대로 넘기려는 코드입니다. (The Book Listing 21-17)
// 반복문의 첫 바퀴에서 receiver가 첫 번째 작업자로 이동하므로 두 번째 바퀴부터는 넘길 값이 없습니다.
// mpsc의 Receiver는 복제할 수도 없습니다(수신자는 하나뿐).
// 고치는 법: Arc::new(Mutex::new(receiver))로 감싸고, 작업자마다 Arc::clone(&receiver)를 넘깁니다.

use std::sync::mpsc;
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, receiver));
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: mpsc::Receiver<Job>) -> Worker {
        let thread = thread::spawn(move || {
            while let Ok(job) = receiver.recv() {
                job();
            }
        });
        Worker { id, thread }
    }
}

fn main() {
    let pool = ThreadPool::new(4);
    pool.execute(|| println!("작업 실행"));
    println!("작업자 {}개, 첫 작업자 id {}", pool.workers.len(), pool.workers[0].id);
    let _ = &pool.workers[0].thread;
}
