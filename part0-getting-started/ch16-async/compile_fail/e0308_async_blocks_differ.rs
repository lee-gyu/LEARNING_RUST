// 기대하는 에러: E0308 — mismatched types (expected `async` block, found a different `async` block)
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0308_async_blocks_differ.rs
//
// 컴파일러는 async 블록마다 이름 없는 타입(상태 기계)을 새로 만듭니다. 두 블록의 내용이 글자 하나까지
// 같아도 서로 다른 타입이므로, 원소의 타입이 하나여야 하는 Vec에 함께 담을 수 없습니다.
// 해결: 공통 타입으로 바꿔 담습니다. Box::pin으로 고정하고 트레이트 객체(dyn, ch17)로 다룹니다.
//       use std::pin::Pin;
//       let futures: Vec<Pin<Box<dyn Future<Output = ()>>>> = vec![Box::pin(hello), Box::pin(bye)];
//       (examples/07_pin_and_join_all.rs의 3번 참고. 같은 async fn을 여러 번 호출해 만든 퓨처들은
//        타입이 같으므로 이렇게 할 필요가 없습니다.)

fn main() {
    let hello = async {
        println!("안녕하세요");
    };
    let bye = async {
        println!("안녕히 가세요");
    };

    let futures = vec![hello, bye];
    println!("퓨처 {}개", futures.len());
}
