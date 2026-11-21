// 기대하는 에러: E0368
// 핵심 메시지: binary assignment operation `+=` cannot be applied to type `std::sync::Mutex<u64>`
//
// Go의 `mu sync.Mutex`와 `count int`, Java의 `synchronized`와 필드처럼, 잠금과 데이터를 따로 두는 언어에서는
// 잠그지 않고 데이터를 고쳐도 컴파일됩니다(compare/01_data_race_counter.c). 그 습관으로 `Mutex`에 든 값을
// 바로 고치려 한 코드입니다. Rust의 `Mutex<T>`는 데이터를 소유하고, 안의 값에는 `lock()`이 돌려주는 가드를
// 거쳐야만 닿을 수 있습니다.
//
// 고치는 법: *visits.lock().unwrap() += 1;
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch8-concurrency-async compile_fail/e0368_mutex_used_as_value.rs

use std::sync::Mutex;
use std::thread;

fn main() {
    let visits: Mutex<u64> = Mutex::new(0);
    thread::scope(|s| {
        s.spawn(|| {
            visits += 1; // E0368
        });
    });
    println!("{:?}", visits.lock().unwrap());
}
