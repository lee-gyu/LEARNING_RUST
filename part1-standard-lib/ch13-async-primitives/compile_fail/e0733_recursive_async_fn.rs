// 기대하는 에러: E0733
// 핵심 메시지: recursion in an async fn requires boxing
//
// async fn이 자기 자신을 .await합니다. 퓨처는 await 너머로 들고 가는 값을 자기 안에 저장하는데,
// 그 값이 같은 타입의 퓨처이면 크기가 끝없이 커집니다.
//
// 확인: rustc --edition 2024 compile_fail/e0733_recursive_async_fn.rs --out-dir /tmp/rust-p1-ch13

async fn sum_to(n: u64) -> u64 {
    if n == 0 { 0 } else { n + sum_to(n - 1).await } // 자기 자신을 그대로 await → E0733
}

fn main() {
    let _future = sum_to(100);
}
