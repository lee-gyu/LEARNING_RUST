// Instant: 경과 시간을 재는 단조 시계(monotonic clock)의 한 시점입니다.
//
//   1) Instant::now()로 시점을 기록하고 elapsed()로 그때부터 흐른 시간을 얻습니다.
//      thread::sleep은 적어도 요청한 만큼 잠들므로 elapsed()는 요청한 시간보다 짧지 않습니다.
//   2) 코드의 실행 시간을 잽니다. 성능은 릴리스 빌드에서 재고, black_box로 입력을 컴파일러에게서 감춥니다.
//   3) 두 Instant의 차이: 뺄셈, duration_since, checked_duration_since
//   4) 제한 시간(timeout): "시작 + 허용 시간"으로 마감 시점(deadline)을 정해 두고 비교합니다.
//
// 실행: cargo run -p p1-ch10-env-process-time --example 06_instant
//
// 걸린 시간 자체는 실행할 때마다 다르므로, 대부분 "N ms 이상인가" 같은 비교 결과로 출력합니다.
// 2번 구역의 걸린 시간 한 줄만 실행마다 값이 다릅니다.

use std::hint::black_box;
use std::thread;
use std::time::{Duration, Instant};

// 클로저를 한 번 실행하고 결과와 걸린 시간을 함께 돌려줍니다.
fn measure<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let value = work();
    (value, start.elapsed())
}

fn sum_of_squares(n: u64) -> u64 {
    (1..=n).map(|i| i * i).sum()
}

// 세 번째 시도에서야 성공하는 작업을 흉내 냅니다. 시도마다 30ms가 걸립니다.
fn flaky(attempt: u32, succeed_at: u32) -> Result<&'static str, &'static str> {
    thread::sleep(Duration::from_millis(30));
    if attempt >= succeed_at {
        Ok("성공")
    } else {
        Err("아직 준비 안 됨")
    }
}

// 마감 시점까지 다시 시도합니다. 성공하면 (결과, 시도 횟수), 시간을 넘기면 Err(시도 횟수)입니다.
fn retry_until(limit: Duration, succeed_at: u32) -> Result<(&'static str, u32), u32> {
    let deadline = Instant::now() + limit; // Instant + Duration = Instant
    let mut attempt = 0;
    loop {
        attempt += 1;
        if let Ok(value) = flaky(attempt, succeed_at) {
            return Ok((value, attempt));
        }
        if Instant::now() >= deadline {
            return Err(attempt);
        }
    }
}

fn main() {
    println!("1. now()와 elapsed()");
    let start = Instant::now();
    thread::sleep(Duration::from_millis(50));
    let waited = start.elapsed();
    println!(
        "    50ms 이상 지남: {}",
        waited >= Duration::from_millis(50)
    ); // 출력:     50ms 이상 지남: true
    println!("    1초 미만: {}", waited < Duration::from_secs(1)); // 출력:     1초 미만: true

    println!("2. 실행 시간 재기");
    // black_box는 "이 값이 무엇인지, 어디에 쓰일지 모른다"고 컴파일러에게 알리는 힌트입니다.
    // 입력에 씌우면 컴파일할 때 결과를 미리 계산해 두지 못합니다. 다만 반복문을 더 빠른 계산으로
    // 바꾸는 최적화까지 막지는 않아서, --release로 실행하면 걸린 시간이 수십 ns로 줄어듭니다.
    let (total, took) = measure(|| sum_of_squares(black_box(1_000_000)));
    println!("    결과 = {total}"); // 출력:     결과 = 333333833333500000
    println!("    걸린 시간 = {took:.1?}"); // 출력:     걸린 시간 = (실행마다 다름, 디버그 빌드에서 수 ms~십수 ms)
    println!("    1초 미만: {}", took < Duration::from_secs(1)); // 출력:     1초 미만: true

    println!("3. 두 시점의 차이");
    let earlier = Instant::now();
    thread::sleep(Duration::from_millis(10));
    let later = Instant::now();
    println!("    later > earlier: {}", later > earlier); // 출력:     later > earlier: true
    println!(
        "    later - earlier >= 10ms: {}",
        later - earlier >= Duration::from_millis(10)
    ); // 출력:     later - earlier >= 10ms: true
    // 순서를 거꾸로 넣으면: duration_since는 지금은 0으로 멈추지만(문서는 나중에 패닉할 수도 있다고 경고),
    // checked_duration_since는 None으로 실수를 드러냅니다.
    println!(
        "    earlier.duration_since(later) = {:?}",
        earlier.duration_since(later)
    ); // 출력:     earlier.duration_since(later) = 0ns
    println!(
        "    earlier.checked_duration_since(later) = {:?}",
        earlier.checked_duration_since(later)
    ); // 출력:     earlier.checked_duration_since(later) = None

    println!("4. 마감 시점까지 다시 시도하기");
    let start = Instant::now();
    let result = retry_until(Duration::from_secs(1), 3);
    println!("    1초 안에, 3번째에 성공하는 작업: {result:?}"); // 출력:     1초 안에, 3번째에 성공하는 작업: Ok(("성공", 3))
    println!(
        "    90ms 이상 걸림: {}",
        start.elapsed() >= Duration::from_millis(90)
    ); // 출력:     90ms 이상 걸림: true

    let start = Instant::now();
    let result = retry_until(Duration::from_millis(100), 100);
    let gave_up = start.elapsed();
    // 시도 횟수는 잠든 시간의 오차에 따라 달라질 수 있으므로 성공했는지만 출력합니다.
    println!("    100ms 안에 끝나지 않는 작업: 성공 = {}", result.is_ok()); // 출력:     100ms 안에 끝나지 않는 작업: 성공 = false
    // 마감을 넘긴 뒤에 알아채므로, 포기한 시점은 마감보다 조금 늦습니다.
    println!(
        "    100ms 이상 지나서 포기: {}",
        gave_up >= Duration::from_millis(100)
    ); // 출력:     100ms 이상 지나서 포기: true
}
