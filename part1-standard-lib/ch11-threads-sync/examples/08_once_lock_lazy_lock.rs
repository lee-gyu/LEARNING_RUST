// 한 번만 초기화: OnceLock과 LazyLock은 OnceCell/LazyCell(Part 1 ch7)의 스레드 안전한 짝입니다.
// static에 둘 수 있고, 여러 스레드가 동시에 처음 접근해도 초기화는 정확히 한 번만 일어납니다.
//
// 실행: cargo run -p p1-ch11-threads-sync --example 08_once_lock_lazy_lock
//
// 스레드 안에서 출력하지 않고, 순서와 무관한 개수·비교만 출력하므로 실행할 때마다 같습니다.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::thread;

#[derive(Debug)]
struct Config {
    name: String,
    workers: usize,
}

// (1) 실행 중에 정해지는 값을 한 번만 넣어 두는 전역: OnceLock
static CONFIG: OnceLock<Config> = OnceLock::new();

// (2) 만드는 방법이 정해진 전역 표: LazyLock
//     HashMap::new()는 const fn이 아니라서 static의 초기값으로 바로 쓸 수 없습니다(E0015).
//     LazyLock은 처음 접근할 때 이 클로저를 불러 값을 만듭니다.
static TABLE_INIT_CALLS: AtomicUsize = AtomicUsize::new(0);
static UNIT_PRICES: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(|| {
    TABLE_INIT_CALLS.fetch_add(1, Ordering::SeqCst);
    HashMap::from([("커피", 4500), ("차", 4000), ("빵", 3000)])
});

// (3) 초기값이 const로 만들어지면 LazyLock 없이 바로 static에 둡니다.
static ORDER_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// 요약을 처음 요청받을 때 한 번만 만들어 두는 글. OnceCell의 자리에 OnceLock을 씁니다.
struct Article {
    body: String,
    summary: OnceLock<String>,
    summary_calls: AtomicUsize,
}

impl Article {
    fn summary(&self) -> &str {
        self.summary.get_or_init(|| {
            self.summary_calls.fetch_add(1, Ordering::SeqCst);
            self.body.chars().take(6).collect::<String>() + "…"
        })
    }
}

fn main() {
    println!("1. OnceLock: 실행 중에 한 번 정하고 그 뒤로는 읽기만 하는 전역 설정");
    {
        println!("    set 전 get: {:?}", CONFIG.get().map(|c| &c.name));
        let first = CONFIG.set(Config {
            name: "주문 서버".to_string(),
            workers: 4,
        });
        let second = CONFIG.set(Config {
            name: "다른 서버".to_string(),
            workers: 8,
        });
        println!("    첫 set: {first:?}");
        // 두 번째 값은 들어가지 않고 Err에 담겨 그대로 돌아옵니다.
        println!("    두 번째 set: {:?}", second.map_err(|c| c.name));

        let seen: Vec<String> = thread::scope(|s| {
            let handles: Vec<_> = (0..3)
                .map(|_| {
                    s.spawn(|| {
                        let c = CONFIG.get().unwrap(); // &'static Config
                        format!("{}/{}", c.name, c.workers)
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        println!("    스레드들이 읽은 값: {seen:?}");
        // 출력: set 전 get: None
        // 출력: 첫 set: Ok(())
        // 출력: 두 번째 set: Err("다른 서버")
        // 출력: 스레드들이 읽은 값: ["주문 서버/4", "주문 서버/4", "주문 서버/4"]

        // wait(1.86.0): 값이 채워질 때까지 기다립니다. 다른 스레드가 set하면 깨어납니다.
        let ready = OnceLock::new();
        let got = thread::scope(|s| {
            let waiter = s.spawn(|| *ready.wait());
            ready.set(42).unwrap();
            waiter.join().unwrap()
        });
        println!("    wait로 받은 값: {got}");
        // 출력: wait로 받은 값: 42
    }

    println!("2. 필드에 둔 OnceLock: 여러 스레드가 동시에 처음 요청해도 계산은 한 번");
    {
        let article = Article {
            body: "러스트 표준 라이브러리의 동기화 도구".to_string(),
            summary: OnceLock::new(),
            summary_calls: AtomicUsize::new(0),
        };

        let summaries: Vec<&str> = thread::scope(|s| {
            let handles: Vec<_> = (0..8).map(|_| s.spawn(|| article.summary())).collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        // 모든 스레드가 받은 &str이 같은 String을 가리키는지 주소로 비교합니다.
        let same = summaries.iter().all(|s| std::ptr::eq(*s, summaries[0]));
        println!(
            "    요약: {}, 계산 횟수: {}",
            summaries[0],
            article.summary_calls.load(Ordering::SeqCst)
        );
        println!("    8개 스레드가 같은 String을 받았는가: {same}");
        // 출력: 요약: 러스트 표준…, 계산 횟수: 1
        // 출력: 8개 스레드가 같은 String을 받았는가: true
    }

    println!("3. LazyLock: 처음 접근할 때 초기화되는 static");
    {
        // LazyLock::get(1.94.0)은 초기화를 일으키지 않고 들여다보기만 합니다.
        println!(
            "    접근 전: 초기화됨? {}",
            LazyLock::get(&UNIT_PRICES).is_some()
        );

        let totals: Vec<u32> = thread::scope(|s| {
            let orders = [("커피", 2), ("빵", 3), ("차", 1), ("커피", 1)];
            let handles: Vec<_> = orders
                .into_iter()
                .map(|(item, qty)| {
                    s.spawn(move || {
                        // 역참조(인덱싱, 메서드 호출)하는 순간 필요하면 초기화가 일어납니다.
                        let price = UNIT_PRICES[item] * qty;
                        ORDER_LOG.lock().unwrap().push(format!("{item}x{qty}"));
                        price
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        println!(
            "    접근 후: 초기화됨? {}",
            LazyLock::get(&UNIT_PRICES).is_some()
        );
        println!(
            "    주문별 금액: {totals:?}, 합계: {}",
            totals.iter().sum::<u32>()
        );
        println!(
            "    초기화 클로저가 불린 횟수: {}",
            TABLE_INIT_CALLS.load(Ordering::SeqCst)
        );
        println!(
            "    ORDER_LOG에 쌓인 주문 수: {}",
            ORDER_LOG.lock().unwrap().len()
        );
        // 출력: 접근 전: 초기화됨? false
        // 출력: 접근 후: 초기화됨? true
        // 출력: 주문별 금액: [9000, 9000, 4000, 4500], 합계: 26500
        // 출력: 초기화 클로저가 불린 횟수: 1
        // 출력: ORDER_LOG에 쌓인 주문 수: 4
    }
}
