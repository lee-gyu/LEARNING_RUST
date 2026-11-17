// 여러 스레드가 하나의 카운터를 함께 고치는 프로그램을 Arc<Mutex<T>>로 만듭니다.
// - 한 스레드에서 쓰던 Rc<RefCell<T>>와 구조가 똑같고, 타입만 Arc<Mutex<T>>로 바뀝니다.
// - Mutex는 데이터 경쟁을 막아 주지만, 잠금을 잘못 나누면 생기는 논리적인 경쟁 조건까지
//   막아 주지는 않습니다.
//
// 실행: cargo run -p p0-ch15-concurrency --example 06_shared_counter
//
// 주의: 3번 구역의 결과 값은 실행마다 달라집니다. 일부러 잘못 만든 코드이기 때문입니다.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

const THREADS: usize = 10;
const INCREMENTS: usize = 1000;

fn main() {
    println!("1. (복습) 한 스레드 안에서는 Rc<RefCell<T>>로 값을 함께 소유하고 바꿉니다");
    let counter = Rc::new(RefCell::new(0));
    let owners: Vec<Rc<RefCell<i32>>> = (0..3).map(|_| Rc::clone(&counter)).collect();
    for owner in &owners {
        *owner.borrow_mut() += 1; // 소유자 셋이 각각 1씩 더합니다.
    }
    println!("    결과: {}", counter.borrow());
    // 출력:     결과: 3

    println!("2. 여러 스레드에서는 Arc<Mutex<T>>로 같은 일을 합니다");
    // Arc: 여러 스레드가 함께 소유하게 해 줍니다(Rc의 스레드 안전 버전).
    // Mutex: 한 번에 한 스레드만 안의 값을 바꾸게 해 줍니다(RefCell의 스레드 안전 버전).
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    for _ in 0..THREADS {
        // 스레드마다 Arc의 복제본을 하나씩 만들어 move로 넘깁니다.
        // 복제되는 것은 포인터와 참조 카운트뿐이고, Mutex와 그 안의 값은 하나뿐입니다.
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..INCREMENTS {
                // 읽고, 1을 더하고, 쓰는 세 단계가 모두 잠금 안에서 일어납니다.
                // MutexGuard는 이 문장이 끝날 때 drop되므로 잠금도 여기서 풀립니다.
                *counter.lock().unwrap() += 1;
            }
        })); // 클로저가 끝나면 그 안의 Arc 복제본이 drop되어 참조 카운트가 1 줄어듭니다.
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!(
        "    결과: {} (기대값 {})",
        *counter.lock().unwrap(),
        THREADS * INCREMENTS
    );
    // 모든 스레드가 끝났으므로 스레드에 넘긴 복제본은 모두 drop되었고, main의 것 하나만 남습니다.
    println!("    남은 Arc 소유자 수: {}", Arc::strong_count(&counter));
    // 출력:     결과: 10000 (기대값 10000)
    // 출력:     남은 Arc 소유자 수: 1

    println!("3. 잠금을 둘로 나누면 컴파일은 되지만 결과가 틀립니다");
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    for _ in 0..THREADS {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..INCREMENTS {
                // 잘못된 코드: 읽을 때 한 번 잠그고(문장이 끝나면 풀림), 쓸 때 다시 잠급니다.
                // 두 문장 사이에 다른 스레드가 끼어들어 같은 값을 읽고 쓸 수 있습니다.
                // 그러면 두 스레드가 모두 "5를 읽고 6을 쓰는" 식으로 더하기 하나가 사라집니다.
                let current = *counter.lock().unwrap();
                *counter.lock().unwrap() = current + 1;
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!(
        "    결과: {} (기대값 {})",
        *counter.lock().unwrap(),
        THREADS * INCREMENTS
    );
    // 출력:     결과: N (기대값 10000)
    //       N은 실행마다 다릅니다. 이 환경에서 여러 번 실행했을 때는 모두 10000보다 훨씬 작았습니다.
}
