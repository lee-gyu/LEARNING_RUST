// 기대하는 에러: E0728 — `await` is only allowed inside `async` functions and blocks
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0728_await_in_closure.rs
//
// double_all은 async fn이지만, .await가 들어 있는 곳은 map에 넘긴 클로저 |n| ... 안입니다.
// 클로저는 그 자체로 별개의 평범한 함수이고 async가 아니므로, 바깥 함수가 async여도 클로저 안에서는
// .await를 쓸 수 없습니다. 에러의 "this is not `async`" 표시가 바깥 함수가 아니라 클로저를 가리킵니다.
// 해결:
//   1) for 반복문으로 바꿔 async fn 본문에서 직접 await합니다(하나씩 차례로 실행).
//          let mut result = Vec::new();
//          for n in numbers { result.push(double(n).await); }
//   2) map으로 퓨처만 만들고(await하지 않음), 모두 한꺼번에 기다립니다(동시에 실행, trpl 필요).
//          trpl::join_all(numbers.into_iter().map(double)).await

async fn double(n: u32) -> u32 {
    n * 2
}

async fn double_all(numbers: Vec<u32>) -> Vec<u32> {
    numbers.into_iter().map(|n| double(n).await).collect()
}

fn main() {
    // 퓨처를 만들기만 하고 실행하지는 않습니다. 이 파일은 컴파일 에러를 보는 것이 목적입니다.
    let _future = double_all(vec![1, 2, 3]);
}
