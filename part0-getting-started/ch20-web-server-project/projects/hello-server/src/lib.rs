//! 정해진 수의 작업자 스레드로 작업을 나눠 실행하는 스레드 풀입니다.
//!
//! The Book 21장의 `ThreadPool`을 따라 만들었습니다. 웹 서버(`src/main.rs`)는 연결 하나를
//! 처리하는 일을 클로저로 만들어 [`ThreadPool::execute`]에 맡깁니다.

use std::fmt;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

/// 작업자 스레드를 미리 만들어 두고, 맡긴 작업을 쉬고 있는 작업자에게 나눠 주는 스레드 풀입니다.
///
/// 작업은 채널 안에서 차례를 기다리다가 먼저 손이 빈 작업자가 꺼내 실행합니다.
/// 풀이 버려질(drop) 때는 새 작업을 더 받지 않고, 이미 맡긴 작업이 모두 끝날 때까지 기다립니다.
///
/// # 예
///
/// ```
/// use hello_server::ThreadPool;
/// use std::sync::{Arc, Mutex};
///
/// let counter = Arc::new(Mutex::new(0));
/// let pool = ThreadPool::new(2);
/// for _ in 0..4 {
///     let counter = Arc::clone(&counter);
///     pool.execute(move || *counter.lock().unwrap() += 1);
/// }
/// drop(pool); // 맡긴 작업이 모두 끝날 때까지 기다립니다.
/// assert_eq!(*counter.lock().unwrap(), 4);
/// ```
pub struct ThreadPool {
    workers: Vec<Worker>,
    // drop에서 송신자를 먼저 버려야 하는데 drop은 &mut self만 받으므로,
    // Option으로 감싸 두고 take()로 꺼냅니다.
    sender: Option<mpsc::Sender<Job>>,
}

// 채널로 보내는 작업 하나입니다. 클로저는 저마다 타입이 달라서 한 채널에 담으려면
// 트레이트 객체로 감싸야 하고, 트레이트 객체는 크기를 알 수 없으므로 Box에 넣습니다.
// Send + 'static은 execute의 바운드와 같은 이유로 붙습니다(다른 스레드로 보내 언제까지든 들고 있음).
type Job = Box<dyn FnOnce() + Send + 'static>;

/// [`ThreadPool::build`]가 스레드 풀을 만들지 못한 이유입니다.
#[derive(Debug, PartialEq)]
pub enum PoolCreationError {
    /// 크기로 0을 받았습니다. 작업자가 없는 풀은 작업을 실행할 수 없습니다.
    ZeroSize,
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolCreationError::ZeroSize => write!(f, "스레드 풀의 크기는 1 이상이어야 합니다"),
        }
    }
}

impl ThreadPool {
    /// 작업자 스레드 `size`개로 이뤄진 스레드 풀을 만듭니다.
    ///
    /// # Panics
    ///
    /// `size`가 0이면 패닉합니다. 크기를 사용자 입력처럼 프로그램 밖에서 받는다면
    /// 패닉 대신 `Err`를 돌려주는 [`ThreadPool::build`]를 쓰세요.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "스레드 풀의 크기는 1 이상이어야 합니다");

        let (sender, receiver) = mpsc::channel();

        // mpsc 채널의 수신자는 하나뿐이고 복제할 수 없습니다. 여러 작업자가 함께 소유하도록
        // Arc로 감싸고, 한 번에 한 작업자만 작업을 꺼내도록 Mutex로 감쌉니다.
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// [`ThreadPool::new`]와 같지만, 크기가 0이면 패닉하는 대신 `Err`를 돌려줍니다.
    ///
    /// # Errors
    ///
    /// `size`가 0이면 [`PoolCreationError::ZeroSize`]를 돌려줍니다.
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::ZeroSize);
        }
        Ok(ThreadPool::new(size))
    }

    /// 작업 `f`를 풀에 맡깁니다.
    ///
    /// 이 메서드는 작업이 끝나기를 기다리지 않고 바로 돌아옵니다. 쉬고 있는 작업자가 없으면
    /// 작업은 채널 안에서 차례를 기다립니다.
    ///
    /// 바운드는 `thread::spawn`과 같은 이유로 붙습니다.
    /// - `FnOnce()`: 작업은 한 번만 실행합니다. 가장 느슨한 바운드라 어떤 클로저든 받을 수 있습니다.
    /// - `Send`: 작업을 다른 스레드(작업자)로 보냅니다.
    /// - `'static`: 작업자가 언제 실행할지 모르므로, 지역 변수를 빌린 클로저는 받을 수 없습니다.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // sender가 None이 되는 것은 drop 안에서뿐이므로 여기서는 항상 Some입니다.
        // as_ref()는 &Option<Sender>를 Option<&Sender>로 바꿔, 송신자를 꺼내지 않고 빌려 씁니다.
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 1. 송신자를 버려 채널을 닫습니다. 그러면 채널에 남은 작업을 다 꺼낸 뒤
        //    작업자의 recv()가 Err를 돌려주고, 작업자의 반복문이 끝납니다.
        //    이 줄이 없으면 작업자는 다음 작업을 영원히 기다리고, 아래의 join도 영원히 기다립니다.
        drop(self.sender.take());

        // 2. 작업자를 벡터에서 하나씩 꺼내(drain) 스레드가 끝나기를 기다립니다.
        //    join은 JoinHandle의 소유권을 가져가는데, &mut self로는 필드를 꺼낼 수 없습니다.
        //    drain(..)은 벡터를 비우면서 원소를 값으로 내주므로 Worker를 통째로 가질 수 있습니다.
        for worker in self.workers.drain(..) {
            println!("작업자 {}의 종료를 기다립니다", worker.id);
            worker.thread.join().unwrap();
        }
    }
}

/// 스레드 하나와, 그 스레드가 실행하는 "작업을 받아 실행하는 반복문"을 묶은 것입니다.
///
/// `thread::spawn`은 넘긴 코드를 곧바로 실행하므로, 스레드를 먼저 만들어 두고 일은 나중에 주려면
/// 스레드 안에서 작업을 기다리는 반복문이 필요합니다. 풀 밖에서 알 필요가 없는 구현 세부라서 비공개입니다.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // lock()이 돌려준 MutexGuard는 이 let 문이 끝날 때 버려지므로, 작업을 실행하기 전에
                // 잠금이 풀립니다. while let이나 match의 조사 대상에 바로 쓰면 작업이 끝날 때까지
                // 잠금을 쥐고 있게 되어 작업이 하나씩만 실행됩니다(README 2.8절, 예제 04).
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("작업자 {id}: 작업을 받아 실행합니다");
                        job();
                    }
                    Err(_) => {
                        println!("작업자 {id}: 채널이 닫혀 종료합니다");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn new_creates_requested_number_of_workers() {
        let pool = ThreadPool::new(3);
        assert_eq!(pool.workers.len(), 3);
    }

    #[test]
    #[should_panic(expected = "스레드 풀의 크기는 1 이상이어야 합니다")]
    fn new_panics_on_zero_size() {
        ThreadPool::new(0);
    }

    #[test]
    fn build_returns_err_on_zero_size() {
        assert_eq!(
            ThreadPool::build(0).err(),
            Some(PoolCreationError::ZeroSize)
        );
    }

    #[test]
    fn build_returns_ok_on_positive_size() {
        assert!(ThreadPool::build(4).is_ok());
    }

    #[test]
    fn execute_does_not_wait_for_job() {
        let pool = ThreadPool::new(1);
        let start = Instant::now();
        pool.execute(|| thread::sleep(Duration::from_millis(300)));
        // 작업은 0.3초가 걸리지만 execute는 채널에 넣기만 하고 바로 돌아옵니다.
        assert!(start.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn jobs_run_in_parallel() {
        let pool = ThreadPool::new(4);
        let start = Instant::now();
        for _ in 0..4 {
            pool.execute(|| thread::sleep(Duration::from_millis(200)));
        }
        drop(pool);
        // 작업자 넷이 동시에 실행하면 약 0.2초, 하나씩 실행하면 0.8초 이상 걸립니다.
        assert!(start.elapsed() < Duration::from_millis(600));
    }

    #[test]
    fn drop_waits_for_all_queued_jobs() {
        // 작업자보다 작업이 많아 대부분의 작업은 채널 안에서 기다립니다.
        let pool = ThreadPool::new(2);
        let finished = Arc::new(Mutex::new(Vec::new()));
        for n in 0..8 {
            let finished = Arc::clone(&finished);
            pool.execute(move || {
                thread::sleep(Duration::from_millis(20));
                finished.lock().unwrap().push(n);
            });
        }
        // drop은 채널을 닫은 뒤, 채널에 남은 작업까지 모두 끝나야 돌아옵니다.
        drop(pool);

        let mut finished = finished.lock().unwrap().clone();
        finished.sort();
        assert_eq!(finished, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }
}
