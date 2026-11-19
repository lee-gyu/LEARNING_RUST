// 기대하는 에러: E0277
// 핵심 메시지: `*mut i32` cannot be sent between threads safely
//              note: required because `px.0` is used within this closure
// 에디션: 2021, 2024에서 실패 / 2018에서 통과(실행하면 15)
//
// 원시 포인터는 Send가 아니므로, 스레드로 넘기려고 감싼 구조체에 unsafe impl Send를 붙이는 일이 흔합니다.
// 2018 에디션의 클로저는 px 전체(Ptr, Send)를 캡처했지만, 2021 에디션부터는 본문이 쓰는
// px.0(*mut i32, Send 아님)만 캡처하므로 감싼 의미가 사라집니다.
// 고치려면 클로저 본문 첫 줄에 let _ = &px; 를 넣어 px 전체를 캡처하게 합니다.
//
// 확인: rustc --edition 2024 compile_fail/e0277_disjoint_capture_not_send.rs --out-dir /tmp/rust-p2-ch3
//       rustc --edition 2018 compile_fail/e0277_disjoint_capture_not_send.rs --out-dir /tmp/rust-p2-ch3

use std::thread;

struct Ptr(*mut i32);

// SAFETY: 이 프로그램에서 Ptr이 가리키는 값은 넘겨받은 스레드 하나만 접근하고,
// main은 join으로 그 스레드가 끝난 뒤에야 값을 다시 읽습니다.
unsafe impl Send for Ptr {}

fn main() {
    let mut x = 5;
    let px = Ptr(&mut x as *mut i32);
    let handle = thread::spawn(move || {
        // → E0277: 이 클로저가 캡처하는 것은 px.0입니다.
        // SAFETY: 포인터는 살아 있는 x를 가리키고, 지금은 이 스레드만 x에 접근합니다.
        unsafe { *(px.0) += 10 };
    });
    handle.join().unwrap();
    println!("{x}");
}
