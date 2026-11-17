// 두 퓨처 중 먼저 끝나는 쪽을 고르는 trpl::select와, 그것으로 timeout을 직접 만드는 방법을 보여 줍니다.
// - select는 먼저 끝난 퓨처의 결과를 Either::Left / Either::Right로 돌려주고, 진 퓨처는 버립니다(취소).
// - select는 공정하지 않습니다. 매번 첫 번째 인자부터 확인하므로 둘 다 준비돼 있으면 왼쪽이 이깁니다.
// - 퓨처는 값이므로 함수 인자로 받아 새 퓨처를 만들 수 있습니다. timeout은 select와 sleep의 조합입니다.
//
// 실행: cargo run -p p0-ch16-async --example 03_select_and_timeout
//
// 주의: 1번과 3번의 승패는 타이머로 정해집니다. 대기 시간을 100ms 이상 벌려 두어 보통 아래 주석과
// 같지만, 컴퓨터가 몹시 바쁘면 달라질 수 있습니다. 2번은 타이머가 없어 항상 같습니다.

use std::time::Duration;

use trpl::Either;

async fn work(name: &str, ms: u64) -> String {
    trpl::sleep(Duration::from_millis(ms)).await;
    println!("    [{name}] 끝 ({ms}ms)");
    format!("{name}의 결과")
}

// The Book 17.3절의 timeout입니다. 어떤 퓨처든(F: Future) 받아서,
// 제한 시간 안에 끝나면 Ok(그 퓨처의 출력), 아니면 Err(제한 시간)을 돌려줍니다.
// F::Output은 "F가 끝나면 내놓는 값의 타입"입니다(Future 트레이트의 연관 타입).
async fn timeout<F: Future>(future_to_try: F, max_time: Duration) -> Result<F::Output, Duration> {
    // 시도할 퓨처를 첫 번째 인자로 둡니다. select는 왼쪽부터 확인하므로,
    // 같은 순간에 둘 다 끝나 있으면 시간 초과보다 성공을 먼저 인정하게 됩니다.
    match trpl::select(future_to_try, trpl::sleep(max_time)).await {
        Either::Left(output) => Ok(output),
        Either::Right(()) => Err(max_time),
    }
}

fn main() {
    trpl::block_on(async {
        println!("1. select는 먼저 끝난 쪽의 결과만 돌려줍니다");
        let slow = work("느린 작업", 300);
        let fast = work("빠른 작업", 100);
        // 둘 다 이길 수 있으므로 Result가 아니라 Either를 씁니다. Left는 첫 번째 인자, Right는 두 번째 인자입니다.
        match trpl::select(slow, fast).await {
            Either::Left(result) => println!("    왼쪽(느린 작업)이 이겼습니다: {result}"),
            Either::Right(result) => println!("    오른쪽(빠른 작업)이 이겼습니다: {result}"),
        }
        // 진 퓨처(느린 작업)는 select 안에서 버려졌습니다. 더 기다려도 그 퓨처의 "끝" 줄은 나오지 않습니다.
        trpl::sleep(Duration::from_millis(300)).await;
        println!("    300ms를 더 기다렸지만 [느린 작업] 끝 줄은 나오지 않았습니다");
        // 출력:
        //     [빠른 작업] 끝 (100ms)
        //     오른쪽(빠른 작업)이 이겼습니다: 빠른 작업의 결과
        //     300ms를 더 기다렸지만 [느린 작업] 끝 줄은 나오지 않았습니다

        println!("2. 둘 다 바로 끝나는 퓨처라면 항상 첫 번째 인자가 이깁니다");
        for round in 1..=3 {
            let winner = match trpl::select(async { "왼쪽" }, async { "오른쪽" }).await {
                Either::Left(left) => left,
                Either::Right(right) => right,
            };
            println!("    {round}회: {winner}");
        }
        // 출력:
        //     1회: 왼쪽
        //     2회: 왼쪽
        //     3회: 왼쪽

        println!("3. select와 sleep을 조합해 timeout을 만듭니다");
        match timeout(work("짧은 작업", 100), Duration::from_millis(300)).await {
            Ok(message) => println!("    성공: {message}"),
            Err(limit) => println!("    {}ms 안에 끝나지 않아 포기했습니다", limit.as_millis()),
        }
        match timeout(work("긴 작업", 500), Duration::from_millis(200)).await {
            Ok(message) => println!("    성공: {message}"),
            Err(limit) => println!("    {}ms 안에 끝나지 않아 포기했습니다", limit.as_millis()),
        }
        // 출력:
        //     [짧은 작업] 끝 (100ms)
        //     성공: 짧은 작업의 결과
        //     200ms 안에 끝나지 않아 포기했습니다
    });
}
