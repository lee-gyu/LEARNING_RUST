// 퓨처가 상태 기계라는 점, 그래서 Pin이 필요하다는 점, 여러 퓨처를 컬렉션에 모아 join_all로 기다리는 방법을 보여 줍니다.
// - async 블록은 컴파일러가 만든 이름 없는 타입의 값입니다. await 너머로 쓰는 지역 변수는 그 값 안에 저장됩니다.
// - 그래서 퓨처는 자기 안의 데이터를 가리키는 참조를 품을 수 있고(자기 참조), 실행을 시작한 뒤에는 옮기면 안 됩니다.
//   Pin은 "이 값은 이제 제자리에서 움직이지 않는다"는 약속이고, Unpin은 "옮겨도 괜찮은 타입"이라는 표시입니다.
// - 같은 async fn이 만든 퓨처들은 타입이 같아서 Vec에 그대로 담아 join_all로 기다릴 수 있습니다.
// - 서로 다른 async 블록은 타입이 모두 다르므로 Box::pin이나 pin!으로 고정한 뒤 dyn Future로 묶습니다.
//   (dyn은 트레이트 객체 문법으로 ch17-trait-objects-oop에서 다룹니다.)
//
// 실행: cargo run -p p0-ch16-async --example 07_pin_and_join_all
//
// 주의: 2번에서 퓨처가 끝나는 순서는 타이머(50ms 간격)로 정해집니다. 보통 아래 주석과 같지만,
// 컴퓨터가 몹시 바쁘면 달라질 수 있습니다. 결과 Vec의 순서는 항상 같습니다.

use std::mem::size_of_val;
use std::pin::{Pin, pin};
use std::time::Duration;

async fn double_later(n: u64) -> u64 {
    // 큰 수일수록 일찍 끝나게 해서(1은 150ms, 3은 50ms), 끝난 순서와 결과의 순서가 다르다는 것을 보입니다.
    trpl::sleep(Duration::from_millis(50 * (4 - n))).await;
    println!("    [double_later({n})] 끝");
    n * 2
}

fn main() {
    trpl::block_on(async {
        println!("1. 퓨처는 await 너머로 쓰는 지역 변수를 자기 안에 저장합니다");
        // data를 await 뒤에서도 쓰므로, 멈춰 있는 동안 data와 그 참조 first를 퓨처가 들고 있어야 합니다.
        // first는 퓨처 자신 안의 data를 가리킵니다. 이 퓨처를 다른 주소로 옮기면 first는 옛 주소를 가리키게 됩니다.
        let keeps_data = async {
            let data = [7u8; 1024];
            let first = &data[0];
            trpl::yield_now().await;
            println!("    [keeps_data] await 뒤에도 data를 씁니다: {first}");
        };
        // data를 await 전에 다 쓰므로, 멈춰 있는 동안 들고 있을 필요가 없습니다.
        let drops_data = async {
            let data = [7u8; 1024];
            let first = data[0];
            trpl::yield_now().await;
            println!("    [drops_data] await 뒤에는 복사한 값만 씁니다: {first}");
        };
        println!("    keeps_data의 크기: {}바이트", size_of_val(&keeps_data));
        println!("    drops_data의 크기: {}바이트", size_of_val(&drops_data));
        // 직접 await하거나 join에 넘기면 런타임이 알아서 퓨처를 고정(pin)한 뒤 실행합니다.
        trpl::join(keeps_data, drops_data).await;
        // 출력 (크기는 rustc 1.98.1, trpl 0.3.0에서 확인한 값이며 버전에 따라 달라질 수 있습니다.
        //       data 1024바이트가 들어 있느냐 없느냐의 차이가 핵심입니다):
        //     keeps_data의 크기: 1056바이트
        //     drops_data의 크기: 24바이트
        //     [keeps_data] await 뒤에도 data를 씁니다: 7
        //     [drops_data] await 뒤에는 복사한 값만 씁니다: 7

        println!("2. 같은 async fn이 만든 퓨처는 타입이 같아 Vec에 담을 수 있습니다");
        let futures: Vec<_> = (1..=3).map(double_later).collect();
        // join_all은 개수가 실행 중에 정해지는 퓨처 모음을 모두 기다립니다. 결과는 넣은 순서대로입니다.
        let results = trpl::join_all(futures).await;
        println!("    결과: {results:?}");
        // 출력:
        //     [double_later(3)] 끝
        //     [double_later(2)] 끝
        //     [double_later(1)] 끝
        //     결과: [2, 4, 6]

        println!("3. 서로 다른 async 블록은 Box::pin으로 고정해 한 Vec에 담습니다");
        // async 블록마다 타입이 달라서 vec![a, b]는 E0308입니다(compile_fail/e0308_async_blocks_differ.rs).
        // Box::pin은 퓨처를 힙에 옮겨 놓고 그 자리에 고정합니다. 이후 Box 자체는 옮겨져도 안의 퓨처는 그대로입니다.
        let greet = async {
            println!("    [greet] 안녕하세요");
        };
        let count = async {
            for i in 1..=2 {
                trpl::yield_now().await;
                println!("    [count] {i}");
            }
        };
        let boxed: Vec<Pin<Box<dyn Future<Output = ()>>>> = vec![Box::pin(greet), Box::pin(count)];
        trpl::join_all(boxed).await;
        // 출력:
        //     [greet] 안녕하세요
        //     [count] 1
        //     [count] 2

        println!("4. pin!은 힙 할당 없이 지금 자리에 고정합니다");
        // pin!(퓨처)는 Pin<&mut 퓨처>를 돌려줍니다. 퓨처는 이 블록의 지역 변수 자리에 고정되므로
        // 이 블록 밖으로 내보낼 수는 없지만, 여기서 join_all로 기다리기에는 충분합니다.
        let first = pin!(async {
            println!("    [first] 실행");
        });
        let second = pin!(async {
            println!("    [second] 실행");
        });
        let pinned: Vec<Pin<&mut dyn Future<Output = ()>>> = vec![first, second];
        trpl::join_all(pinned).await;
        // 출력:
        //     [first] 실행
        //     [second] 실행
    });
}
