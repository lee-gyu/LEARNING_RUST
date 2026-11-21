// Q1.1 · Q1.8 — 스레드 사이의 공유를 컴파일러가 검사한다
//
// 여러 줄의 텍스트를 네 조각으로 나눠 스레드 네 개가 단어 수를 셉니다.
//
//   1. 읽기만 하는 공유: 스레드마다 원본의 다른 조각을 빌려 읽고, 결과는 돌려줌
//   2. 함께 고치는 공유: 스레드들이 합계 하나를 고침. `Mutex` 없이는 컴파일되지 않음
//
// `thread::scope`는 스코프가 끝나기 전에 안에서 만든 스레드가 모두 끝난다고 보장합니다. 그래서
// 스레드가 지역 변수를 `move`나 `Arc` 없이 빌릴 수 있습니다(Part 1 ch11에서 자세히 다룹니다).
// 결과는 `main`에서 모아 출력하므로 스레드가 끝나는 순서와 상관없이 출력은 실행마다 같습니다.
//
// 실행: cargo run -p faq-ch1-why-rust --example 03_checked_thread_sharing

use std::sync::Mutex;
use std::thread;

fn count_words(lines: &[String]) -> usize {
    lines
        .iter()
        .map(|line| line.split_whitespace().count())
        .sum()
}

fn main() {
    // i번째 줄에는 단어가 i개 있습니다(1~8개, 모두 36개).
    let lines: Vec<String> = (1..=8).map(|i| "단어 ".repeat(i)).collect();

    println!("1. 읽기만 하는 공유: 조각을 빌려 읽고 결과를 돌려줌");
    let partial: Vec<usize> = thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(2) // 원본을 복사하지 않고 두 줄씩 가리키는 슬라이스 네 개
            .map(|chunk| s.spawn(|| count_words(chunk)))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    println!(
        "  조각별 단어 수 {partial:?}, 합계 {}",
        partial.iter().sum::<usize>()
    );
    // 출력:   조각별 단어 수 [3, 7, 11, 15], 합계 36
    println!("  스레드가 끝난 뒤에도 원본은 그대로: {}줄", lines.len());
    // 출력:   스레드가 끝난 뒤에도 원본은 그대로: 8줄
    // 스레드들은 원본을 빌렸을 뿐이고, 빌린 것은 스코프가 끝나며 모두 돌아왔습니다.

    println!("2. 함께 고치는 공유: 합계 하나를 Mutex로 보호");
    let total = Mutex::new(0);
    thread::scope(|s| {
        for chunk in lines.chunks(2) {
            s.spawn(|| {
                let n = count_words(chunk);
                *total.lock().unwrap() += n; // 잠근 동안에만 합계에 닿을 수 있음
            });
        }
    });
    // 스레드가 모두 끝났으므로 `Mutex`를 풀어 안의 값을 꺼냅니다(`into_inner`).
    println!("  합계 {}", total.into_inner().unwrap());
    // 출력:   합계 36
    // `Mutex` 없이 `let mut total = 0;`을 두면 어느 쪽으로 써도 데이터 경쟁은 생기지 않습니다.
    //   - 스레드마다 `s.spawn(|| total += count_words(chunk))`처럼 빌려 고치면: 둘 이상의 스레드가 같은 값을
    //     동시에 가변으로 빌리게 되므로 컴파일되지 않습니다(E0499).
    //   - `move` 클로저로 고치면: `usize` 값이 스레드마다 복사되어 각자 제 사본만 고치고,
    //     합계는 0으로 남습니다. 컴파일러가 "value captured by `total` is never read" 경고로 알려 줍니다.
    // 두 경우의 자세한 이야기는 ch8 `ch8-concurrency-async`에서 다룹니다.
}
