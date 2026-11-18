// 기대하는 에러: E0499
// 핵심 메시지: cannot borrow `scores` as mutable more than once at a time
//
// 스코프 스레드는 지역 변수를 빌릴 수 있지만, 대여 규칙은 그대로 적용됩니다.
// 두 스레드가 같은 Vec을 동시에 가변으로 빌리려 하면 컴파일러가 거부합니다.
//
// 확인: rustc --edition 2024 compile_fail/e0499_scoped_threads_mut_borrow.rs --out-dir /tmp/rust-p1-ch11

use std::thread;

fn main() {
    let mut scores = vec![10, 20, 30];

    thread::scope(|s| {
        s.spawn(|| scores.push(40)); // 첫 번째 가변 대여
        s.spawn(|| scores.push(50)); // 두 번째 가변 대여 → E0499
    });

    println!("{scores:?}");
}
