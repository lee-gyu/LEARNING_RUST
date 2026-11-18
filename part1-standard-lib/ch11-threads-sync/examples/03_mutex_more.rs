// Mutex 더 쓰기: try_lock, 가드가 살아 있는 범위 줄이기, 잠그는 순서로 교착 상태 피하기,
// 포이즈닝을 만났을 때의 선택지, &mut나 소유권이 있을 때의 get_mut/into_inner.
//
// 실행: cargo run -p p1-ch11-threads-sync --example 03_mutex_more
//
// 2·3번 구역은 Barrier로 두 스레드의 진행 시점을 맞춰서, 실행할 때마다 같은 결과가 나옵니다.
// 4번 구역은 일부러 스레드 하나를 패닉시킵니다. 기본 패닉 훅이 표준 에러에 메시지를 남기고
// (스레드 ID는 실행마다 다름), 패닉은 join으로 받으므로 종료 코드는 0입니다.

use std::sync::{Arc, Barrier, Mutex, TryLockError};
use std::thread;

struct Account {
    id: u32,
    balance: Mutex<i64>,
}

// 두 계좌를 잠글 때 항상 id가 작은 쪽부터 잠급니다.
// 모든 스레드가 같은 순서를 따르면 "서로 상대가 쥔 잠금을 기다리는" 상황이 생기지 않습니다.
fn transfer(from: &Account, to: &Account, amount: i64) {
    let (first, second) = if from.id < to.id {
        (from, to)
    } else {
        (to, from)
    };
    let mut first_balance = first.balance.lock().unwrap();
    let mut second_balance = second.balance.lock().unwrap();
    if first.id == from.id {
        *first_balance -= amount;
        *second_balance += amount;
    } else {
        *second_balance -= amount;
        *first_balance += amount;
    }
}

// 불변식: entries의 합 == total
#[derive(Debug)]
struct Ledger {
    entries: Vec<i64>,
    total: i64,
}

fn describe<T>(result: Result<T, TryLockError<T>>) -> &'static str {
    match result {
        Ok(_) => "Ok(가드)",
        Err(TryLockError::WouldBlock) => "Err(WouldBlock)",
        Err(TryLockError::Poisoned(_)) => "Err(Poisoned)",
    }
}

fn main() {
    println!("1. try_lock은 기다리지 않고 지금 잠글 수 있는지만 알려 줍니다");
    {
        let m = Mutex::new(0);
        println!("    아무도 안 잠갔을 때: {}", describe(m.try_lock()));
        let guard = m.lock().unwrap();
        println!("    가드가 살아 있을 때: {}", describe(m.try_lock()));
        drop(guard);
        println!("    가드를 버린 뒤:      {}", describe(m.try_lock()));
        // 출력: 아무도 안 잠갔을 때: Ok(가드)
        // 출력: 가드가 살아 있을 때: Err(WouldBlock)
        // 출력: 가드를 버린 뒤:      Ok(가드)
    }

    println!("2. 오래 걸리는 일은 잠금 밖에서 합니다");
    {
        let config = Mutex::new(vec![String::from("theme=dark"), String::from("lang=ko")]);
        let barrier = Barrier::new(2);

        thread::scope(|s| {
            // (가) 가드를 쥔 채로 일하는 스레드
            s.spawn(|| {
                let guard = config.lock().unwrap();
                barrier.wait(); // 가드를 쥔 상태에서 main과 만납니다.
                let _work = guard.join(";"); // 오래 걸리는 일이라고 생각합니다.
                barrier.wait(); // main이 확인을 마칠 때까지 기다린 뒤 가드를 버립니다.
            });
            barrier.wait();
            println!(
                "    (가) 가드를 쥔 채 일하는 동안 다른 스레드: {}",
                describe(config.try_lock())
            );
            barrier.wait();
        });

        thread::scope(|s| {
            // (나) 필요한 것만 복제해 두고 가드를 바로 버리는 스레드
            s.spawn(|| {
                let snapshot = config.lock().unwrap().clone(); // 가드는 이 문장 끝에서 버려짐
                barrier.wait();
                let _work = snapshot.join(";");
                barrier.wait();
            });
            barrier.wait();
            println!(
                "    (나) 복사본으로 일하는 동안 다른 스레드:   {}",
                describe(config.try_lock())
            );
            barrier.wait();
        });
        // 출력: (가) 가드를 쥔 채 일하는 동안 다른 스레드: Err(WouldBlock)
        // 출력: (나) 복사본으로 일하는 동안 다른 스레드:   Ok(가드)
    }

    println!("3. 여러 잠금을 서로 다른 순서로 잡으면 교착 상태가 됩니다");
    {
        let a = Mutex::new("A");
        let b = Mutex::new("B");
        let barrier = Barrier::new(2);

        // 스레드 1은 a를, 스레드 2는 b를 먼저 잡은 상태에서 상대 잠금을 시도합니다.
        // lock()이었다면 둘 다 영원히 기다리므로, 기다리지 않는 try_lock으로 상황만 관찰합니다.
        let (r1, r2) = thread::scope(|s| {
            let t1 = s.spawn(|| {
                let _a = a.lock().unwrap();
                barrier.wait(); // 두 스레드가 각자 첫 잠금을 쥔 순간
                let r = describe(b.try_lock());
                barrier.wait(); // 상대도 시도를 마칠 때까지 첫 잠금을 쥐고 있습니다.
                r
            });
            let t2 = s.spawn(|| {
                let _b = b.lock().unwrap();
                barrier.wait();
                let r = describe(a.try_lock());
                barrier.wait();
                r
            });
            (t1.join().unwrap(), t2.join().unwrap())
        });
        println!("    a를 쥔 스레드 1이 b를 시도: {r1}");
        println!("    b를 쥔 스레드 2가 a를 시도: {r2}");
        // 출력: a를 쥔 스레드 1이 b를 시도: Err(WouldBlock)
        // 출력: b를 쥔 스레드 2가 a를 시도: Err(WouldBlock)

        // 해결: 모든 스레드가 같은 순서(여기서는 id 순서)로 잠급니다.
        let x = Account {
            id: 1,
            balance: Mutex::new(1000),
        };
        let y = Account {
            id: 2,
            balance: Mutex::new(1000),
        };
        thread::scope(|s| {
            s.spawn(|| (0..10_000).for_each(|_| transfer(&x, &y, 1)));
            s.spawn(|| (0..10_000).for_each(|_| transfer(&y, &x, 1)));
        });
        let (bx, by) = (*x.balance.lock().unwrap(), *y.balance.lock().unwrap());
        println!(
            "    순서를 지킨 이체 2만 번 뒤: x = {bx}, y = {by}, 합계 = {}",
            bx + by
        );
        // 출력: 순서를 지킨 이체 2만 번 뒤: x = 1000, y = 1000, 합계 = 2000
    }

    println!("4. 포이즈닝: 가드를 쥔 스레드가 패닉하면 그 뒤의 잠금은 Err를 돌려줍니다");
    {
        let ledger = Mutex::new(Ledger {
            entries: vec![100, 200],
            total: 300,
        });

        let joined = thread::scope(|s| {
            s.spawn(|| {
                let mut guard = ledger.lock().unwrap();
                guard.entries.push(50); // 고치는 도중에
                let fee: i64 = "수수료".parse().unwrap(); // 패닉! total은 아직 고치지 못함
                guard.total += 50 + fee;
            })
            .join()
        });
        println!(
            "    작업 스레드의 join: {}",
            if joined.is_err() { "Err(패닉)" } else { "Ok" }
        );
        println!("    is_poisoned: {}", ledger.is_poisoned());
        println!("    try_lock: {}", describe(ledger.try_lock()));

        // 선택지 1: lock().unwrap()으로 패닉을 이어 간다 → 이 스레드도 멈춤(여기서는 하지 않음)
        // 선택지 2: PoisonError::into_inner로 가드를 꺼내 상태를 확인하고 바로잡는다
        let mut guard = match ledger.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let sum: i64 = guard.entries.iter().sum();
        println!("    꺼내 본 상태: {:?} (항목 합 {sum})", *guard);
        guard.total = sum; // 불변식을 복구합니다.
        drop(guard);
        ledger.clear_poison(); // 복구했다고 표시합니다(1.77.0).
        println!("    복구 후 is_poisoned: {}", ledger.is_poisoned());
        // 출력: 작업 스레드의 join: Err(패닉)
        // 출력: is_poisoned: true
        // 출력: try_lock: Err(Poisoned)
        // 출력: 꺼내 본 상태: Ledger { entries: [100, 200, 50], total: 300 } (항목 합 350)
        // 출력: 복구 후 is_poisoned: false
    }

    println!("5. 혼자 가지고 있으면 잠글 필요가 없습니다: get_mut, into_inner");
    {
        let mut local = Mutex::new(vec![1, 2]);
        // &mut Mutex는 다른 누구도 이 Mutex를 볼 수 없다는 뜻이므로 실제로 잠그지 않습니다.
        local.get_mut().unwrap().push(3);
        let v = local.into_inner().unwrap(); // Mutex를 소비하고 안의 값을 꺼냅니다.
        println!("    get_mut 후 into_inner: {v:?}");

        // Arc로 나눠 준 Mutex도 스레드가 모두 끝나면 마지막 소유자가 꺼낼 수 있습니다.
        let shared = Arc::new(Mutex::new(Vec::new()));
        let handles: Vec<_> = (1..=3)
            .map(|id| {
                let shared = Arc::clone(&shared);
                thread::spawn(move || shared.lock().unwrap().push(id * 10))
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        // Arc::into_inner는 마지막 강한 참조일 때만 Some입니다(Rc::into_inner와 같은 이름, 1.70.0).
        let mut collected = Arc::into_inner(shared).unwrap().into_inner().unwrap();
        collected.sort(); // 넣은 순서는 실행마다 다를 수 있어 정렬합니다.
        println!("    Arc::into_inner로 꺼낸 값: {collected:?}");
        // 출력: get_mut 후 into_inner: [1, 2, 3]
        // 출력: Arc::into_inner로 꺼낸 값: [10, 20, 30]
    }
}
