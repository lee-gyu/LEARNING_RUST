// 기대하는 에러: E0728 — `await` is only allowed inside `async` functions and blocks
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0728_await_outside_async.rs
//
// .await는 "결과가 준비될 때까지 이 퓨처를 멈추고 런타임에 차례를 넘긴다"는 뜻입니다.
// 멈췄다가 다시 이어 가려면 그 함수 자체가 퓨처(상태 기계)여야 하는데, 평범한 fn은 퓨처가 아니므로
// 멈출 방법이 없습니다. 그래서 .await는 async fn이나 async 블록 안에서만 쓸 수 있습니다.
// 해결: 동기 코드에서 퓨처의 결과가 필요하면 런타임에 맡깁니다.
//       let n = trpl::block_on(fetch_number());
//       또는 trpl::block_on(async { let n = fetch_number().await; ... });

async fn fetch_number() -> u32 {
    42
}

fn main() {
    let n = fetch_number().await;
    println!("{n}");
}
