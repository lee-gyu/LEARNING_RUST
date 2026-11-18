// 기대하는 에러: (에러 코드 없음)
// 핵심 메시지: future cannot be sent between threads safely
//              future is not `Send` as this value is used across an await
//
// 08 예제의 spawn처럼 Send인 퓨처만 받는 함수에, Rc를 await 너머로 들고 가는 async 블록을 넘깁니다.
// await 너머로 들고 가는 값은 퓨처 안에 저장되므로, 그 값이 Send가 아니면 퓨처도 Send가 아닙니다.
//
// 확인: rustc --edition 2024 compile_fail/future_not_send_across_await.rs --out-dir /tmp/rust-p1-ch13

use std::rc::Rc;

/// 08 예제의 Executor::spawn과 같은 바운드를 가진 함수(본문은 퓨처를 버리기만 함).
fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    drop(future);
}

async fn yield_now() {}

fn main() {
    spawn(async {
        let name = Rc::new(String::from("태스크"));
        yield_now().await; // name이 이 await 지점 너머에서도 쓰임 → 퓨처가 Send가 아님
        println!("{name}");
    });
}
