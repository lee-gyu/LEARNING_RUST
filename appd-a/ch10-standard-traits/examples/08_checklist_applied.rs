// 지침: 공개 타입을 내놓을 때는 표준 트레이트를 체크리스트로 하나씩 따지고, 약속을 지킬 수
//       있는 것은 넉넉하게 구현합니다. 빠뜨린 트레이트는 사용자가 대신 구현할 수 없습니다
//       (트레이트도 타입도 사용자의 것이 아니므로 고아 규칙에 걸림).
// 비권장: 가계부 라이브러리의 YearMonth(연월)에 아무 트레이트도 없음
// 권장:   체크리스트를 적용한 YearMonth
//
// 실행: cargo run -p er-ch10-standard-traits --example 08_checklist_applied

// 기본으로 꺼진 두 린트를 켜면 공개 타입의 빠진 Debug와 Copy를 알려 줍니다.
#![warn(missing_debug_implementations, missing_copy_implementations)]

use std::collections::{BTreeMap, HashMap};

const SPENDING: [(u16, u8, u32); 4] = [
    (2026, 11, 12_000),
    (2025, 12, 8_000),
    (2026, 10, 41_000),
    (2026, 11, 51_500),
];

// ---- 라이브러리 v0: 생성자와 접근자만 -------------------------------------------

pub mod v0 {
    #[expect(
        missing_debug_implementations,
        missing_copy_implementations,
        reason = "트레이트가 빠진 판을 보이려고 일부러 뺌"
    )]
    pub struct YearMonth {
        year: u16,
        month: u8,
    }

    impl YearMonth {
        pub fn new(year: u16, month: u8) -> Option<YearMonth> {
            (1..=12)
                .contains(&month)
                .then_some(YearMonth { year, month })
        }
        pub fn year(&self) -> u16 {
            self.year
        }
        pub fn month(&self) -> u8 {
            self.month
        }
    }
}

// ---- 라이브러리 v1: 체크리스트 적용 -----------------------------------------------

pub mod v1 {
    use std::fmt;

    /// 연월. month는 언제나 1~12입니다.
    // Debug          예: 개발자가 볼 수 있어야 함. 숨길 비밀 없음 → derive
    // Clone, Copy    예: 정수 두 개짜리 값이고, 연월이라는 개념에 필드가 늘 일이 없음 → derive
    // PartialEq, Eq  예: 필드가 모두 같으면 같은 달. NaN 같은 예외 없음 → derive
    // Hash           예: Eq와 같은 필드를 넣음 → derive (맵의 키로 쓸 수 있음)
    // PartialOrd, Ord 예: 시간 순서가 곧 자연스러운 순서이고, 필드를 year, month 순서로
    //                선언했으므로 derive가 맞음. 필드 순서를 바꾸면 순서가 바뀌므로 바꾸지 않음
    // Default        아니오: "기본 달"이 없고, derive하면 생성자가 거부하는 0년 0월이 생김
    // Display        예: 표준 표기(YYYY-MM) 하나가 분명함 → 손으로
    // 연산자          아니오: 연월 + 연월은 뜻이 없음. 다음 달은 이름 붙은 메서드로
    // Send, Sync     자동 트레이트라 구현하지 않음. 필드가 모두 Send + Sync라 저절로 따라옴
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct YearMonth {
        year: u16,
        month: u8,
    }

    impl YearMonth {
        pub fn new(year: u16, month: u8) -> Option<YearMonth> {
            (1..=12)
                .contains(&month)
                .then_some(YearMonth { year, month })
        }
        pub fn year(self) -> u16 {
            self.year
        }
        pub fn month(self) -> u8 {
            self.month
        }
        /// 다음 달. 연도가 u16을 넘으면 None입니다.
        pub fn next(self) -> Option<YearMonth> {
            if self.month == 12 {
                Some(YearMonth {
                    year: self.year.checked_add(1)?,
                    month: 1,
                })
            } else {
                Some(YearMonth {
                    month: self.month + 1,
                    ..self
                })
            }
        }
    }

    impl fmt::Display for YearMonth {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:04}-{:02}", self.year, self.month)
        }
    }
}

// ---- 사용자 코드(다른 크레이트라고 생각합니다) -----------------------------------

// v1의 YearMonth는 Debug, Clone, PartialEq를 구현하므로 사용자도 derive할 수 있습니다.
// 필드를 v0::YearMonth로 바꾸면 derive(Debug)부터 E0277입니다
// (compile_fail/e0277_user_derive_debug.rs). 다른 크레이트의 타입이라면
// impl Debug for v0::YearMonth도 고아 규칙(E0117) 때문에 쓸 수 없습니다.
#[derive(Debug, Clone, PartialEq)]
struct MonthlyReport {
    month: v1::YearMonth,
    total: u32,
}

fn assert_send_sync<T: Send + Sync>() {}

fn main() {
    println!("[비권장] 트레이트가 없는 YearMonth: (연, 월) 튜플로 우회");
    // Eq + Hash가 없어 키로 쓸 수 없으므로, 접근자로 꺼낸 튜플을 키로 씁니다.
    let mut totals: HashMap<(u16, u8), u32> = HashMap::new();
    for (year, month, amount) in SPENDING {
        if let Some(ym) = v0::YearMonth::new(year, month) {
            *totals.entry((ym.year(), ym.month())).or_default() += amount;
        }
    }
    // Ord가 없으니 정렬도 튜플로, Display가 없으니 출력 형식도 사용자가 직접 맞춥니다.
    let mut keys: Vec<_> = totals.keys().copied().collect();
    keys.sort();
    for (year, month) in keys {
        println!("  {year:04}-{month:02}: {}원", totals[&(year, month)]);
    }
    // 출력:   2025-12: 8000원
    // 출력:   2026-10: 41000원
    // 출력:   2026-11: 63500원

    println!("[권장] 체크리스트를 적용한 YearMonth");
    // Ord가 있으니 BTreeMap의 키로 바로 쓰고, 순회하면 시간 순서입니다.
    let mut totals: BTreeMap<v1::YearMonth, u32> = BTreeMap::new();
    for (year, month, amount) in SPENDING {
        if let Some(ym) = v1::YearMonth::new(year, month) {
            *totals.entry(ym).or_default() += amount;
        }
    }
    for (month, total) in &totals {
        println!("  {month}: {total}원");
    }
    // 출력:   2025-12: 8000원
    // 출력:   2026-10: 41000원
    // 출력:   2026-11: 63500원

    // Eq + Hash가 있으니 HashMap의 키로도 쓸 수 있습니다.
    let by_hash: HashMap<v1::YearMonth, u32> = totals.iter().map(|(m, t)| (*m, *t)).collect();
    let november = v1::YearMonth::new(2026, 11);
    println!(
        "  HashMap에서 2026-11 찾기: {:?}",
        november.and_then(|m| by_hash.get(&m))
    );
    // 출력:   HashMap에서 2026-11 찾기: Some(63500)

    if let Some((&month, &total)) = totals.last_key_value() {
        let report = MonthlyReport { month, total };
        let copy = report.clone();
        println!("  최근 보고서: {report:?}");
        // 출력:   최근 보고서: MonthlyReport { month: YearMonth { year: 2026, month: 11 }, total: 63500 }
        println!(
            "  복제본과 같은가? {}, 다음 달: {}",
            copy == report,
            month.next().map_or(String::from("없음"), |m| m.to_string())
        );
        // 출력:   복제본과 같은가? true, 다음 달: 2026-12
        println!("  연 {} / 월 {}", month.year(), month.month());
        // 출력:   연 2026 / 월 11
    }

    // Send + Sync는 구현하지 않았지만 자동으로 따라왔다는 것을 컴파일 단계에서 확인합니다.
    // 필드에 Rc 같은 타입이 끼면 이 줄이 E0277로 바뀝니다.
    assert_send_sync::<v1::YearMonth>();
}
