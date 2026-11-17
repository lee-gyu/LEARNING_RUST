// 기대하는 에러: E0752 — `main` function is not allowed to be `async`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0752_async_main.rs
//
// async fn main은 "main을 호출하면 퓨처가 나온다"는 뜻입니다. 그런데 퓨처는 누군가 실행해 줘야
// 진행되고(퓨처는 게으릅니다), 그 일을 하는 런타임은 Rust 표준 라이브러리에 들어 있지 않습니다.
// 프로그램의 출발점인 main이 돌려준 퓨처를 실행해 줄 주체가 없으므로 main은 async일 수 없습니다.
// 해결: main은 평범한 fn으로 두고, 안에서 런타임을 시작해 최상위 퓨처를 넘깁니다.
//       fn main() {
//           trpl::block_on(async {
//               let n = fetch_number().await;
//               println!("{n}");
//           });
//       }
//       (tokio를 직접 쓰면 #[tokio::main] 속성이 async fn main을 이런 모양의 평범한 main으로 바꿔 줍니다.)

async fn fetch_number() -> u32 {
    42
}

async fn main() {
    let n = fetch_number().await;
    println!("{n}");
}
