// Q5.6 — try/finally, defer, with, using 자리에 오는 것: 잠금이 풀리는 때
//
// 잠금은 어떤 길로 빠져나가든 풀어야 하는 자원입니다. 다른 언어라면 이렇게 씁니다.
//
//   Java    lock.lock(); try { … } finally { lock.unlock(); }
//   Go      mu.Lock(); defer mu.Unlock()
//   Python  with lock: …
//
// Rust의 `Mutex::lock`은 잠금 가드(`MutexGuard`)를 돌려주고, 가드가 스코프를 벗어나 버려질 때(`Drop`)
// 잠금이 풀립니다. 그래서 푸는 코드를 따로 쓰지 않습니다.
//
//   1·2. 가드를 쥔 함수가 정상으로 끝나든 `?`로 일찍 돌아가든, 돌아온 뒤에는 다시 잠글 수 있음
//   3.   가드는 함수가 아니라 블록이 끝날 때 버려지므로, 반복문 안에서 잠그면 한 바퀴마다 풀림
//        (Go의 `defer`는 함수가 돌아갈 때 실행됨)
//
// 잠겼는지는 Part 0 ch15 예제 05처럼 기다리지 않는 `try_lock`으로 확인합니다. 블록 끝이나
// `drop(guard)`에서 가드가 풀리는 것은 그 예제가, 일찍 돌아가도 값이 버려지는 것은 ch1 예제 01(메모리)이
// 이미 보였습니다. 여기서 더하는 것은 `?`로 일찍 돌아가는 길에서도 잠금이 풀린다는 것과, 정리가 붙는
// 단위가 Go의 `defer`(함수)와 달리 블록이라는 것입니다.
//
// 실행: cargo run -p faq-ch5-no-null-no-exceptions --example 03_cleanup_like_finally

use std::sync::Mutex;

fn parse_size(text: &str) -> Result<u32, String> {
    text.parse()
        .map_err(|_| format!("크기가 숫자가 아님: {text:?}"))
}

/// 크기를 읽어 기록에 덧붙이고, 지금까지 쌓인 기록 수를 돌려줍니다.
fn record(log: &Mutex<Vec<u32>>, size_text: &str) -> Result<usize, String> {
    let mut entries = log.lock().unwrap(); // 잠금. 푸는 코드는 따로 쓰지 않습니다.
    println!(
        "    잠근 동안 다시 잠글 수 있나: {}",
        log.try_lock().is_ok()
    );
    let size = parse_size(size_text)?; // 실패하면 여기서 돌아가며, entries가 버려져 잠금이 풀립니다.
    entries.push(size);
    Ok(entries.len())
} // 성공해도 여기서 entries가 버려져 잠금이 풀립니다.

fn main() {
    let log = Mutex::new(Vec::new());

    println!("1. 정상적으로 끝남");
    let result = record(&log, "10");
    println!(
        "  결과: {result:?}, 돌아온 뒤 잠글 수 있나: {}",
        log.try_lock().is_ok()
    );
    // 출력:     잠근 동안 다시 잠글 수 있나: false
    // 출력:   결과: Ok(1), 돌아온 뒤 잠글 수 있나: true

    println!("2. ?로 일찍 돌아감");
    let result = record(&log, "열");
    println!(
        "  결과: {result:?}, 돌아온 뒤 잠글 수 있나: {}",
        log.try_lock().is_ok()
    );
    // 출력:     잠근 동안 다시 잠글 수 있나: false
    // 출력:   결과: Err("크기가 숫자가 아님: \"열\""), 돌아온 뒤 잠글 수 있나: true

    println!("3. 반복문 안에서 잠그면 한 바퀴마다 풀림");
    for size in [20, 30] {
        // 앞 바퀴의 가드가 남아 있었다면, 이미 잠근 스레드가 다시 lock을 부른 것이라 돌아오지 않습니다.
        let mut entries = log.lock().unwrap();
        entries.push(size);
        println!("  {size} 넣음: 기록 {}개", entries.len());
    } // 한 바퀴가 끝날 때마다 entries가 버려집니다.
    println!("  반복문 뒤 기록: {:?}", log.lock().unwrap());
    // 출력:   20 넣음: 기록 2개
    // 출력:   30 넣음: 기록 3개
    // 출력:   반복문 뒤 기록: [10, 20, 30]
}
