// async fn이 무엇으로 바뀌는지, 퓨처(future)가 게으르다는 것, 런타임이 왜 필요한지를 보여 줍니다.
// - async fn을 호출하면 본문이 실행되지 않고 "나중에 값을 내놓을" 퓨처만 만들어집니다.
// - 퓨처는 .await하거나 런타임(trpl::block_on)에 넘겨야 실행됩니다. 그냥 버리면 끝내 실행되지 않습니다.
// - main은 async일 수 없으므로(compile_fail/e0752_async_main.rs) block_on으로 런타임을 시작합니다.
//
// 실행: cargo run -p p0-ch16-async --example 01_async_fn_and_lazy_futures

use std::time::Duration;

// async fn: 반환 타입에는 u32라고 적지만, 호출한 쪽이 받는 것은 u32가 아니라
// "다 끝나면 u32를 내놓는 퓨처"입니다. 정확한 타입은 impl Future<Output = u32>입니다.
async fn slow_add(a: u32, b: u32) -> u32 {
    println!("    [slow_add] 본문 시작: {a} + {b}");
    // trpl::sleep은 스레드를 재우지 않습니다. "100ms 뒤에 깨워 달라"고 런타임에 맡기고
    // 이 퓨처를 잠시 멈출 뿐입니다. 이렇게 멈출 수 있는 자리를 await 지점이라고 부릅니다.
    trpl::sleep(Duration::from_millis(100)).await;
    println!("    [slow_add] 본문 끝");
    a + b
}

// 컴파일러가 위의 async fn을 바꾸는 모양을 손으로 쓴 것입니다.
// 평범한 함수가 본문 전체를 async move 블록에 담아 "퓨처를 돌려줄" 뿐이고, 본문은 여기서 실행되지 않습니다.
// (Edition 2024에서는 Future 트레이트가 프렐류드에 있어서 use 없이 쓸 수 있습니다.)
// Clippy는 이런 함수를 보면 "async fn으로 쓰라"고 경고합니다(manual_async_fn). 두 형태가 같은 뜻이라는
// 증거이기도 합니다. 여기서는 비교하려고 일부러 풀어 썼으므로 그 경고를 예상된 것으로 표시합니다.
#[expect(
    clippy::manual_async_fn,
    reason = "async fn이 바뀌는 모양을 보여 주려고 일부러 풀어 씀"
)]
fn slow_add_desugared(a: u32, b: u32) -> impl Future<Output = u32> {
    // move: 매개변수 a, b의 소유권을 블록 안으로 옮깁니다. 함수는 블록을 돌려주고 바로 끝나므로,
    // 블록이 나중에 실행될 때 함수의 지역 변수를 빌려서는 안 됩니다.
    async move {
        println!("    [slow_add_desugared] 본문 시작: {a} + {b}");
        trpl::sleep(Duration::from_millis(100)).await;
        println!("    [slow_add_desugared] 본문 끝");
        a + b
    }
}

fn main() {
    println!("1. async fn을 호출하면 퓨처만 만들어집니다");
    let future = slow_add(1, 2);
    println!("    slow_add(1, 2)를 호출했지만 본문은 아직 한 줄도 실행되지 않았습니다");
    // 출력:
    //     slow_add(1, 2)를 호출했지만 본문은 아직 한 줄도 실행되지 않았습니다
    // ([slow_add] 줄이 이 시점에는 나오지 않습니다.)

    println!("2. block_on에 넘기면 그제야 실행됩니다");
    // block_on은 런타임을 만들고, 넘겨받은 퓨처가 끝날 때까지 현재 스레드를 블로킹(blocking)한 채
    // 퓨처를 실행합니다. 동기 코드(main)와 비동기 코드가 만나는 경계입니다.
    let sum = trpl::block_on(future);
    println!("    block_on이 돌려준 값: {sum}");
    // 출력:
    //     [slow_add] 본문 시작: 1 + 2
    //     [slow_add] 본문 끝
    //     block_on이 돌려준 값: 3

    println!("3. async 블록 안에서는 .await로 다른 퓨처의 결과를 기다립니다");
    // async 블록도 퓨처입니다. 블록 안의 .await는 적힌 순서대로 하나씩 진행됩니다.
    // 첫 번째 .await가 끝나야 두 번째 퓨처가 시작되므로 여기서는 동시에 실행되지 않습니다.
    let total = trpl::block_on(async {
        let x = slow_add(1, 2).await;
        let y = slow_add_desugared(x, 10).await;
        y * 2
    });
    println!("    결과: {total}");
    // 출력:
    //     [slow_add] 본문 시작: 1 + 2
    //     [slow_add] 본문 끝
    //     [slow_add_desugared] 본문 시작: 3 + 10
    //     [slow_add_desugared] 본문 끝
    //     결과: 26

    println!("4. async 블록도 만들기만 해서는 실행되지 않습니다");
    let block = async {
        println!("    [async 블록] 실행됨");
        "블록의 결과"
    };
    println!("    async 블록을 변수에 담았습니다");
    let result = trpl::block_on(block);
    println!("    block_on이 돌려준 값: {result}");
    // 출력:
    //     async 블록을 변수에 담았습니다
    //     [async 블록] 실행됨
    //     block_on이 돌려준 값: 블록의 결과

    println!("5. 기다리지 않고 버린 퓨처는 끝내 실행되지 않습니다");
    trpl::block_on(async {
        let forgotten = slow_add(100, 200);
        // 한 번도 실행되지 않은 퓨처를 버립니다(drop은 ch14). 스레드는 만들자마자 실행되기 시작하지만,
        // 퓨처는 누군가 기다려 주지 않으면 아무 일도 하지 않습니다.
        drop(forgotten);
        println!("    퓨처를 버렸습니다. [slow_add] 줄은 출력되지 않습니다");
    });
    // 출력:
    //     퓨처를 버렸습니다. [slow_add] 줄은 출력되지 않습니다
}
