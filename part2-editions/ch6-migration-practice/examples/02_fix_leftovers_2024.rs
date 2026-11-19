//! 2021 → 2024 `cargo fix --edition`이 남긴 모양과, 사람이 다듬은 모양을 나란히 실행합니다.
//!
//! 실행: cargo run -p p2-ch6-migration-practice --example 02_fix_leftovers_2024
//!
//! 두 모양은 대부분 같은 값을 내지만, 1번 구역처럼 "언제 잠금이 풀리는가" 같은 숨은 동작이
//! 다를 수 있습니다. 자동 수정은 2021 에디션의 동작을 지키는 쪽으로만 고칩니다.

use std::ops::RangeInclusive;
use std::sync::Mutex;

/// 칸마다 합계를 캐시하는 장부
struct Totals {
    cache: Mutex<[Option<i64>; 4]>,
    amounts: Vec<(usize, i64)>,
}

impl Totals {
    fn sum_of(&self, slot: usize) -> i64 {
        self.amounts
            .iter()
            .filter(|(s, _)| *s == slot)
            .map(|(_, a)| a)
            .sum()
    }

    /// cargo fix 모양: `if let`을 `match`로 바꿔 2021 에디션의 임시값 범위를 지킵니다.
    /// 조건식의 잠금(`MutexGuard`)이 `match`가 끝날 때까지 살아 있습니다.
    /// 돌려주는 bool은 "캐시가 비어 있을 때 다시 잠글 수 있었는가"입니다.
    fn total_fixed(&self, slot: usize) -> (i64, bool) {
        let mut relocked = false;
        let total = match self.cache.lock().unwrap()[slot] {
            Some(cached) => cached,
            _ => {
                relocked = self.cache.try_lock().is_ok();
                self.sum_of(slot)
            }
        };
        self.cache.lock().unwrap()[slot] = Some(total);
        (total, relocked)
    }

    /// 다듬은 모양: 2024 에디션의 `if let`은 else 블록에 들어가기 전에 잠금을 풉니다.
    /// 그래서 else 안에서 바로 캐시를 채울 수 있습니다. 2021 에디션이었다면 잠금을 쥔 채로
    /// 같은 `Mutex`를 다시 잠그게 되어 교착 상태나 패닉이 됩니다.
    fn total_cleaned(&self, slot: usize) -> (i64, bool) {
        if let Some(cached) = self.cache.lock().unwrap()[slot] {
            (cached, false)
        } else {
            let relocked = self.cache.try_lock().is_ok();
            let total = self.sum_of(slot);
            self.cache.lock().unwrap()[slot] = Some(total);
            (total, relocked)
        }
    }
}

struct Ids {
    last: u32,
}

impl Ids {
    /// cargo fix 모양: `+ use<>`로 "self를 캡처하지 않는다"고 적어 2021의 캡처 규칙을 지킵니다.
    fn ids_fixed(&self) -> impl Iterator<Item = u32> + use<> {
        1..=self.last
    }

    /// 다른 길: 숨길 필요가 없는 타입이면 구체 타입을 그대로 돌려줍니다.
    fn ids_concrete(&self) -> RangeInclusive<u32> {
        1..=self.last
    }

    fn bump(&mut self) -> u32 {
        self.last += 1;
        self.last
    }
}

enum Budget {
    Limit(i64),
    Unlimited,
}

impl Budget {
    fn describe(&self) -> String {
        match self {
            Budget::Limit(limit) => format!("{limit}원까지"),
            Budget::Unlimited => String::from("한도 없음"),
        }
    }
}

/// cargo fix 모양: `expr`를 `expr_2021`로 바꿔 `_`가 첫 규칙에 걸리지 않게 합니다.
macro_rules! budget_fixed {
    ($limit:expr_2021) => {
        Budget::Limit($limit)
    };
    (_) => {
        Budget::Unlimited
    };
}

/// 다듬은 모양: `_` 규칙을 먼저 두고 2024의 `expr`를 그대로 씁니다. `const { .. }`도 받습니다.
macro_rules! budget_cleaned {
    (_) => {
        Budget::Unlimited
    };
    ($limit:expr) => {
        Budget::Limit($limit)
    };
}

struct Config {
    unit: String,
}

impl Config {
    fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Config {
        Config {
            unit: lookup("P2_CH6_UNIT").unwrap_or_else(|| String::from("원")),
        }
    }
}

fn main() {
    println!("1. if let → match");
    let totals = Totals {
        cache: Mutex::new([None; 4]),
        amounts: vec![(0, 1000), (0, 2000), (1, 500)],
    };
    let (sum, relocked) = totals.total_fixed(0);
    println!("    cargo fix 모양: 합계 {sum}, 빈 캐시일 때 다시 잠금 {relocked}");
    let (sum, relocked) = totals.total_cleaned(1);
    println!("    다듬은 모양:    합계 {sum}, 빈 캐시일 때 다시 잠금 {relocked}");
    println!(
        "    두 번째 호출(캐시): {:?}, {:?}",
        totals.total_fixed(0),
        totals.total_cleaned(1)
    );
    // 출력:     cargo fix 모양: 합계 3000, 빈 캐시일 때 다시 잠금 false
    // 출력:     다듬은 모양:    합계 500, 빈 캐시일 때 다시 잠금 true
    // 출력:     두 번째 호출(캐시): (3000, false), (500, false)

    println!("2. + use<>");
    let mut ids = Ids { last: 2 };
    let held = ids.ids_fixed(); // 반복자를 쥔 채로
    let new_id = ids.bump(); // &mut 대여가 가능합니다
    let concrete = ids.ids_concrete();
    println!(
        "    쥐고 있던 번호 {:?}, 새 번호 {new_id}, 구체 타입 {concrete:?}",
        held.collect::<Vec<_>>()
    );
    // 출력:     쥐고 있던 번호 [1, 2], 새 번호 3, 구체 타입 1..=3

    println!("3. expr_2021");
    println!(
        "    cargo fix 모양: {}, {}",
        budget_fixed!(30_000).describe(),
        budget_fixed!(_).describe()
    );
    println!(
        "    다듬은 모양:    {}, {}, {}",
        budget_cleaned!(30_000).describe(),
        budget_cleaned!(_).describe(),
        budget_cleaned!(const { 3 * 10_000 }).describe()
    );
    // 출력:     cargo fix 모양: 30000원까지, 한도 없음
    // 출력:     다듬은 모양:    30000원까지, 한도 없음, 30000원까지

    println!("4. unsafe {{ set_var }}");
    // cargo fix 모양: 자동 수정은 unsafe 블록과 FIXME 주석을 넣을 뿐입니다.
    // FIXME: Audit that the environment access only happens in single-threaded code.
    // SAFETY: (FIXME를 확인한 결과) 이 예제는 여기까지 스레드를 하나도 만들지 않았습니다.
    unsafe { std::env::set_var("P2_CH6_UNIT", "엔") };
    let fixed = Config::from_lookup(|key| std::env::var(key).ok());
    // SAFETY: 위와 같습니다.
    unsafe { std::env::remove_var("P2_CH6_UNIT") };
    // 다듬은 모양: 환경을 바꾸지 않고 값을 직접 넘깁니다. unsafe가 필요 없습니다.
    let cleaned = Config::from_lookup(|_| Some(String::from("엔")));
    println!(
        "    cargo fix 모양: {}, 다듬은 모양: {}",
        fixed.unit, cleaned.unit
    );
    // 출력:     cargo fix 모양: 엔, 다듬은 모양: 엔
}
