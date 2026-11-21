// Q3.5 — 교착 상태와 경쟁 조건은 컴파일된다
//
// 데이터 경쟁은 안전한 코드에서 컴파일되지 않지만(ch1 예제 03), 그보다 넓은 동시성 버그는
// 컴파일러가 막지 않습니다.
//
//   1. 잠금을 쥔 채 같은 `Mutex`를 다시 잠그기: 표준 라이브러리 문서에 따르면 `lock()`은 이때
//      돌아오지 않으므로(패닉하거나 멈춤) `try_lock()`으로 "지금 잠글 수 있는가"만 물어 관찰합니다.
//   2. 확인과 행동을 다른 잠금에서 하기(경쟁 조건): 잔액 100원에서 두 스레드가 70원씩 출금합니다.
//      `Barrier`로 두 스레드가 모두 "확인"을 마친 뒤에 "출금"하게 순서를 일부러 맞춥니다.
//
// 모든 접근이 `Mutex`를 거치므로 데이터 경쟁은 없습니다. 출력은 실행마다 같습니다.
//
// 실행: cargo run -p faq-ch3-safety-limits --example 02_deadlock_and_race

use std::sync::{Barrier, Mutex, TryLockError};
use std::thread;

/// 확인과 출금을 서로 다른 잠금에서 합니다(비권장).
fn withdraw_split(balance: &Mutex<i64>, amount: i64, barrier: &Barrier) -> bool {
    let enough = *balance.lock().unwrap() >= amount; // 확인. 이 문장이 끝나면 잠금이 풀립니다.
    barrier.wait(); // 두 스레드가 모두 확인을 마칠 때까지 기다립니다.
    if enough {
        *balance.lock().unwrap() -= amount; // 출금. 확인한 뒤에 잔액이 바뀌었을 수 있습니다.
    }
    enough
}

/// 확인과 출금을 한 번의 잠금 안에서 합니다(권장).
fn withdraw_atomic(balance: &Mutex<i64>, amount: i64, barrier: &Barrier) -> bool {
    barrier.wait();
    let mut b = balance.lock().unwrap();
    if *b >= amount {
        *b -= amount;
        true
    } else {
        false
    }
}

/// 두 스레드가 동시에 70원씩 출금하고 (성공 수, 최종 잔액)을 돌려줍니다.
fn run(withdraw: fn(&Mutex<i64>, i64, &Barrier) -> bool) -> (usize, i64) {
    let balance = Mutex::new(100);
    let barrier = Barrier::new(2);
    let successes = thread::scope(|s| {
        let a = s.spawn(|| withdraw(&balance, 70, &barrier));
        let b = s.spawn(|| withdraw(&balance, 70, &barrier));
        [a.join().unwrap(), b.join().unwrap()]
            .iter()
            .filter(|&&ok| ok)
            .count()
    });
    (successes, balance.into_inner().unwrap())
}

fn main() {
    println!("1. 잠금을 쥔 채 같은 Mutex를 다시 잠그기");
    let stock = Mutex::new(10);
    let guard = stock.lock().unwrap();
    match stock.try_lock() {
        Err(TryLockError::WouldBlock) => {
            println!("  지금은 잠글 수 없음 (lock()이었다면 이 호출은 돌아오지 않음)")
        }
        Ok(_) => println!("  잠금 성공"),
        Err(TryLockError::Poisoned(_)) => println!("  포이즈닝"),
    }
    // 출력:   지금은 잠글 수 없음 (lock()이었다면 이 호출은 돌아오지 않음)
    drop(guard);
    println!("  잠금을 푼 뒤 다시 잠금: {}", stock.try_lock().is_ok());
    // 출력:   잠금을 푼 뒤 다시 잠금: true

    println!("2. 잔액 100원에서 두 스레드가 70원씩 출금");
    let (ok, left) = run(withdraw_split);
    println!("  [비권장] 확인과 출금을 따로 잠금: 성공 {ok}건, 잔액 {left}원");
    // 출력:   [비권장] 확인과 출금을 따로 잠금: 성공 2건, 잔액 -40원
    let (ok, left) = run(withdraw_atomic);
    println!("  [권장] 한 번의 잠금 안에서 확인과 출금: 성공 {ok}건, 잔액 {left}원");
    // 출력:   [권장] 한 번의 잠금 안에서 확인과 출금: 성공 1건, 잔액 30원
}
