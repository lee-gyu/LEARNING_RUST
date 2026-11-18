// RwLock: 읽기 잠금은 여러 스레드가 동시에, 쓰기 잠금은 한 스레드만 가질 수 있습니다.
// 읽기가 대부분인 설정 값은 RwLock<Arc<T>>로 두고, 읽는 쪽은 Arc 복제본(스냅숏)만 가져가는
// 모양이 자주 쓰입니다. Arc::make_mut, Arc::ptr_eq도 여기서 봅니다.
//
// 실행: cargo run -p p1-ch11-threads-sync --example 04_rwlock
//
// Barrier로 스레드의 진행 시점을 맞추므로 출력은 실행할 때마다 같습니다.

use std::sync::{Arc, Barrier, RwLock, TryLockError};
use std::thread;

#[derive(Clone, Debug)]
struct Config {
    version: u32,
    greeting: String,
}

fn describe<T>(result: Result<T, TryLockError<T>>) -> &'static str {
    match result {
        Ok(_) => "Ok",
        Err(TryLockError::WouldBlock) => "Err(WouldBlock)",
        Err(TryLockError::Poisoned(_)) => "Err(Poisoned)",
    }
}

fn main() {
    println!("1. 읽기 잠금은 여러 스레드가 동시에 쥘 수 있습니다");
    {
        let menu = RwLock::new(vec!["김밥", "라면", "떡볶이"]);
        // 읽는 스레드 3개 + main = 4. 네 스레드가 모두 도착해야 wait()에서 풀려납니다.
        let barrier = Barrier::new(4);

        let lengths = thread::scope(|s| {
            let readers: Vec<_> = (0..3)
                .map(|i| {
                    let (menu, barrier) = (&menu, &barrier);
                    s.spawn(move || {
                        let items = menu.read().unwrap();
                        barrier.wait(); // 읽기 잠금을 쥔 채 만납니다.
                        barrier.wait(); // main이 확인을 마칠 때까지 계속 쥐고 있습니다.
                        items[i].chars().count()
                    })
                })
                .collect();

            barrier.wait();
            // 세 스레드가 읽기 잠금을 쥔 채 Barrier를 통과했다는 것 자체가
            // "동시에 읽고 있었다"는 증거입니다. Mutex였다면 한 스레드만 잠금을 얻으므로
            // 나머지가 Barrier에 도착하지 못해 영원히 멈췄을 것입니다.
            println!(
                "    세 스레드가 읽는 중에 try_read:  {}",
                describe(menu.try_read())
            );
            println!(
                "    세 스레드가 읽는 중에 try_write: {}",
                describe(menu.try_write())
            );
            barrier.wait();
            readers
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        println!("    읽은 메뉴의 글자 수: {lengths:?}");
        println!("    모두 끝난 뒤 try_write: {}", describe(menu.try_write()));
        // 출력: 세 스레드가 읽는 중에 try_read:  Ok
        // 출력: 세 스레드가 읽는 중에 try_write: Err(WouldBlock)
        // 출력: 읽은 메뉴의 글자 수: [2, 2, 3]
        // 출력: 모두 끝난 뒤 try_write: Ok
    }

    println!("2. 쓰기 잠금은 혼자만 가집니다");
    {
        let score = RwLock::new(0);
        let mut writer = score.write().unwrap();
        *writer += 10;
        let (r, w) = thread::scope(|s| {
            s.spawn(|| (describe(score.try_read()), describe(score.try_write())))
                .join()
                .unwrap()
        });
        drop(writer);
        println!("    쓰는 중에 다른 스레드의 try_read: {r}, try_write: {w}");
        println!("    쓰기를 마친 뒤 읽은 값: {}", *score.read().unwrap());
        // 출력: 쓰는 중에 다른 스레드의 try_read: Err(WouldBlock), try_write: Err(WouldBlock)
        // 출력: 쓰기를 마친 뒤 읽은 값: 10
    }

    println!("3. RwLock<Arc<T>>: 읽는 쪽은 스냅숏을 들고 가고 잠금은 바로 놓습니다");
    {
        let config = RwLock::new(Arc::new(Config {
            version: 1,
            greeting: "안녕".to_string(),
        }));
        let barrier = Barrier::new(3);

        let seen = thread::scope(|s| {
            let readers: Vec<_> = (0..2)
                .map(|_| {
                    s.spawn(|| {
                        // 읽기 잠금은 Arc를 복제하는 동안만 쥡니다(이 문장 끝에서 놓음).
                        let snapshot = Arc::clone(&config.read().unwrap());
                        barrier.wait(); // 스냅숏을 들고 있는 동안 main이 설정을 바꿉니다.
                        barrier.wait();
                        format!("v{} {}", snapshot.version, snapshot.greeting)
                    })
                })
                .collect();

            barrier.wait();
            {
                let mut current = config.write().unwrap(); // 읽기 잠금이 없으니 바로 얻음
                let before = Arc::clone(&current);
                println!(
                    "    바꾸기 전 같은 Arc를 가진 소유자 수: {}",
                    Arc::strong_count(&before)
                );
                // make_mut: 다른 소유자(스냅숏)가 있으면 안의 값을 복제해 갈라선 뒤 고칩니다.
                let cfg = Arc::make_mut(&mut current);
                cfg.version = 2;
                cfg.greeting = "반가워요".to_string();
                println!(
                    "    make_mut 뒤 바꾸기 전과 같은 할당인가: {}",
                    Arc::ptr_eq(&before, &current)
                );
            }
            barrier.wait();
            readers
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        let now = config.read().unwrap();
        println!("    읽는 스레드가 본 값: {seen:?}");
        println!("    지금 설정: v{} {}", now.version, now.greeting);
        // 출력: 바꾸기 전 같은 Arc를 가진 소유자 수: 4
        // 출력: make_mut 뒤 바꾸기 전과 같은 할당인가: false
        // 출력: 읽는 스레드가 본 값: ["v1 안녕", "v1 안녕"]
        // 출력: 지금 설정: v2 반가워요
    }
}
