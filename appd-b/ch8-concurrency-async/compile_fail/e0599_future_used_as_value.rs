// 기대하는 에러: E0599
// 핵심 메시지: no method named `len` found for opaque type `impl Future<Output = String>` in the current scope
//
// JavaScript에서 async 함수를 부르면 결과 대신 Promise를 받는 것처럼, Rust에서 async 함수를 부르면 결과가
// 아니라 퓨처(future)를 받습니다. 일반 함수 main에서 async 함수를 불러 그 결과를 바로 쓰려 한 코드입니다.
// JavaScript와 달리 이 퓨처는 아직 본문을 한 줄도 실행하지 않았습니다.
//
// 컴파일러는 `.await`를 붙이라고 제안하지만, main은 async 함수가 아니므로 그대로 따르면 E0728이 납니다
// (Part 0 ch16 compile_fail/e0728_await_outside_async.rs). 일반 함수에서는 런타임에 퓨처를 넘겨 실행합니다.
//
// 고치는 법(Part 0 ch16처럼 trpl을 쓴다면): let title = trpl::block_on(fetch_title(1));
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch8-concurrency-async compile_fail/e0599_future_used_as_value.rs

async fn fetch_title(id: u32) -> String {
    format!("글 {id}")
}

fn main() {
    let title = fetch_title(1);
    println!("제목 길이: {}", title.len()); // E0599
}
