// 기대하는 에러: E0277
// error[E0277]: `Rc<Mutex<i32>>` cannot be sent between threads safely
//   = help: ... the trait `Send` is not implemented for `Rc<Mutex<i32>>`
//
// 확인: rustc --edition 2024 compile_fail/e0277_rc_is_not_send.rs
//
// Rc는 참조 카운트를 평범한 정수 연산으로 늘리고 줄입니다. 두 스레드가 같은 Rc의 복제본을
// 동시에 clone하거나 drop하면 카운트 갱신이 서로 덮어써져 틀린 값이 될 수 있고, 그러면
// 값이 너무 일찍 해제되거나 영영 해제되지 않습니다. 그래서 Rc는 Send가 아니고,
// thread::spawn은 Send가 아닌 값을 담은 클로저를 받지 않습니다.

use std::rc::Rc;
use std::sync::Mutex;
use std::thread;

fn main() {
    let counter = Rc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Rc::clone(&counter);
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
