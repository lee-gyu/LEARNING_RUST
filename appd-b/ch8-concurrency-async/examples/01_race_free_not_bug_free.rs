// Q8.2 — 데이터 경쟁이 없다는 것과 프로그램이 맞다는 것은 다르다
//
// 세 줄의 단어 수를 스레드 셋이 나눠 세고 합계를 냅니다. Rust는 데이터 경쟁(data race)을 컴파일 시간에
// 막지만, 막는 것은 "같은 메모리를 동시에 고치는 일"뿐이고 계산이 맞는지는 보장하지 않습니다.
//
//   1. `move` 클로저로 합계를 고침: `usize`는 `Copy`라서 스레드마다 사본을 고치고, 원래 합계는 0으로 남음.
//      데이터 경쟁은 없지만 답이 틀립니다. 컴파일러는 경고 두 개("value captured by `total` is never read",
//      "value assigned to `total` is never read")로만 알립니다.
//   2. 원자 타입을 빌려 함께 더함: 모든 스레드가 같은 값 하나를 고칩니다.
//
// 합계를 `move` 없이 빌려 `total += …`로 고치면 둘 이상의 스레드가 같은 값을 동시에 가변으로 빌리므로
// 컴파일되지 않습니다(E0499, Part 1 ch11 compile_fail/e0499_scoped_threads_mut_borrow.rs와 같은 이유).
// 이 예제는 ch1 예제 03 끝 주석이 넘긴 두 변형 가운데 컴파일되는 쪽을 실제로 돌려 봅니다.
//
// 실행: cargo run -p faq-ch8-concurrency-async --example 01_race_free_not_bug_free

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn count_words(line: &str) -> usize {
    line.split_whitespace().count()
}

#[expect(
    unused_assignments,
    reason = "move 클로저가 합계의 사본만 고친다는 것을 보이려고 일부러 남긴 코드"
)]
fn total_with_move(lines: &[&str]) -> usize {
    let mut total = 0;
    thread::scope(|s| {
        for &line in lines {
            s.spawn(move || total += count_words(line)); // 스레드마다 total의 사본을 고침
        }
    });
    total
}

fn total_with_atomic(lines: &[&str]) -> usize {
    let total = AtomicUsize::new(0);
    thread::scope(|s| {
        for &line in lines {
            // move가 없어도 됩니다. total은 스코프 밖에 있어 스레드보다 오래 살고, line은 &str이라
            // 클로저가 line 변수가 아니라 그것이 가리키는 문자열을 빌립니다.
            s.spawn(|| total.fetch_add(count_words(line), Ordering::Relaxed));
        }
    });
    total.into_inner()
}

fn main() {
    let lines = ["봄 여름", "가을", "겨울 그리고 다시 봄"];

    println!("1. move 클로저로 고친 합계");
    println!("  합계: {}", total_with_move(&lines));
    // 출력:   합계: 0

    println!("2. 원자 타입을 빌려 함께 더한 합계");
    println!("  합계: {}", total_with_atomic(&lines));
    // 출력:   합계: 7
}
