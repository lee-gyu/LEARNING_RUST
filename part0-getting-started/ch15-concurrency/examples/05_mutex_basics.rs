// Mutex<T>의 사용법을 스레드 하나에서 먼저 살펴봅니다.
// - lock()이 돌려주는 MutexGuard를 통해서만 안의 값에 접근할 수 있습니다.
// - MutexGuard가 drop되는 순간 잠금이 풀립니다(ch14의 Drop).
// - MutexGuard는 안의 값을 가리키는 스마트 포인터처럼 쓸 수 있습니다(ch14의 Deref).
//
// 실행: cargo run -p p0-ch15-concurrency --example 05_mutex_basics

use std::cell::RefCell;
use std::sync::Mutex;

fn main() {
    println!("1. lock으로 잠그고, 받은 MutexGuard로 값을 바꿉니다");
    // m은 mut가 아닌데도 안의 값을 바꿀 수 있습니다. RefCell처럼 내부 가변성을 제공하기 때문입니다.
    let m = Mutex::new(5);
    println!("    처음:   {m:?}");
    {
        // lock()은 잠금을 얻을 때까지 기다린 뒤 Result<MutexGuard<i32>, _>를 돌려줍니다.
        // Err는 잠금을 쥔 다른 스레드가 패닉했을 때만 나오므로 여기서는 unwrap합니다.
        let mut num = m.lock().unwrap();
        *num = 6; // MutexGuard는 DerefMut을 구현하므로 *로 안의 i32를 바꿀 수 있습니다.
    } // num(MutexGuard)이 여기서 drop되며 잠금이 풀립니다. unlock을 따로 부를 필요가 없습니다.
    println!("    바꾼 뒤: {m:?}");
    // 출력:     처음:   Mutex { data: 5, poisoned: false, .. }
    // 출력:     바꾼 뒤: Mutex { data: 6, poisoned: false, .. }

    println!("2. MutexGuard가 살아 있는 동안은 잠겨 있습니다");
    let m = Mutex::new(String::from("공유 데이터"));
    let guard = m.lock().unwrap();
    // try_lock은 기다리지 않는 lock입니다. 이미 잠겨 있으면 바로 Err를 돌려줍니다.
    println!(
        "    guard가 있을 때 try_lock 성공? {}",
        m.try_lock().is_ok()
    );
    // Mutex의 디버그 형식도 잠겨 있으면 안의 값 대신 "<locked>"를 보여 줍니다.
    println!("    guard가 있을 때: {m:?}");
    drop(guard); // 스코프가 끝나기 전에 일찍 풀고 싶으면 drop을 직접 부릅니다.
    println!("    drop 후 try_lock 성공? {}", m.try_lock().is_ok());
    println!("    drop 후: {m:?}");
    // 출력:     guard가 있을 때 try_lock 성공? false
    // 출력:     guard가 있을 때: Mutex { data: "<locked>", poisoned: false, .. }
    // 출력:     drop 후 try_lock 성공? true
    // 출력:     drop 후: Mutex { data: "공유 데이터", poisoned: false, .. }

    println!("3. MutexGuard로 안의 값의 메서드를 바로 부를 수 있습니다");
    let m = Mutex::new(vec![1, 2, 3]);
    {
        let mut numbers = m.lock().unwrap();
        // numbers의 타입은 MutexGuard<Vec<i32>>지만, 역참조 강제 변환 덕분에
        // Vec<i32>의 메서드를 그대로 부를 수 있습니다.
        numbers.push(4);
        let sum: i32 = numbers.iter().sum();
        println!("    길이 {}, 합 {sum}", numbers.len());
    }
    // 출력:     길이 4, 합 10

    println!("4. 변수에 담지 않은 MutexGuard는 그 문장이 끝날 때 바로 풀립니다");
    // lock()이 만든 MutexGuard를 변수에 담지 않고 바로 쓰면, 그 문장이 끝날 때 drop됩니다.
    m.lock().unwrap().push(5);
    // 그래서 다음 문장에서는 다시 잠글 수 있습니다.
    println!("    다음 문장에서 try_lock 성공? {}", m.try_lock().is_ok());
    println!("    {:?}", m.lock().unwrap());
    // 출력:     다음 문장에서 try_lock 성공? true
    // 출력:     [1, 2, 3, 4, 5]

    println!("5. RefCell과 Mutex는 \"이미 쓰는 중\"을 다르게 다룹니다");
    let cell = RefCell::new(0);
    let borrowed = cell.borrow_mut();
    // RefCell은 이미 가변으로 빌려 간 상태에서 borrow_mut를 또 부르면 패닉합니다.
    // try_borrow_mut는 패닉 대신 Err로 그 상태를 알려 줍니다.
    println!(
        "    RefCell: 빌려 간 상태에서 또 빌리기 성공? {}",
        cell.try_borrow_mut().is_ok()
    );
    drop(borrowed);

    let mutex = Mutex::new(0);
    let locked = mutex.lock().unwrap();
    // Mutex는 다른 스레드가 잠그고 있으면 lock이 풀릴 때까지 기다립니다.
    // 같은 스레드에서 lock을 또 부르면 영원히 기다리거나 패닉할 수 있으므로(교착 상태),
    // 여기서는 기다리지 않는 try_lock으로만 확인합니다.
    println!(
        "    Mutex:   잠긴 상태에서 또 잠그기 성공? {}",
        mutex.try_lock().is_ok()
    );
    drop(locked);
    // 출력:     RefCell: 빌려 간 상태에서 또 빌리기 성공? false
    // 출력:     Mutex:   잠긴 상태에서 또 잠그기 성공? false
}
