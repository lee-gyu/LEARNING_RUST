// 기대하는 에러: E0382
// error[E0382]: borrow of moved value: `val`
//
// 확인: rustc --edition 2024 compile_fail/e0382_use_after_send.rs
//
// send는 값의 소유권을 채널로 옮깁니다. 보낸 순간부터 그 값의 주인은 받는 쪽이고,
// 받는 스레드는 그 값을 바꾸거나 버릴 수 있습니다. 보낸 쪽이 계속 읽을 수 있다면
// 두 스레드가 동기화 없이 같은 데이터에 접근하게 되므로 거부합니다.

use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let val = String::from("안녕");
        tx.send(val).unwrap();
        println!("보낸 값: {val}");
    });

    let received = rx.recv().unwrap();
    println!("받은 값: {received}");
}
