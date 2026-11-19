// 컴파일러가 잡지 못하는 호환되지 않는 변경: derive(Ord)의 필드 순서를 바꾸면 정렬 결과가 바뀝니다.
//
// 실행: cargo run -p er-ch21-semver --example 02_silent_behavior_change
//
// 가상의 택배 라이브러리 `parcel`의 배송 예약 칸 `Slot`을 세 판으로 흉내 냅니다.
//   v1_0_0      : 1.0.0. day, window 순서로 선언하고 PartialOrd/Ord를 derive합니다.
//   v1_0_1_bad  : 1.0.1(비권장). 관리자가 필드를 "보기 좋게" window, day 순서로 바꿨습니다.
//                 공개 API의 이름과 타입은 그대로라 패치로 냈습니다.
//   v1_0_1_good : 1.0.1(권장). 같은 재배치를 하되 Ord를 손으로 써서 비교 순서를 고정했습니다.
//
// derive(PartialOrd, Ord)는 필드를 선언한 순서대로 사전식으로 비교합니다(ch10 2.2절).
// 그래서 선언 순서가 곧 "정렬 순서"라는 공개 동작이고, 바꾸면 사용자의 정렬 결과가 바뀝니다.
// 사용자 코드는 필드 이름을 적은 구조체 리터럴만 쓰므로 세 판 모두에서 경고 없이 컴파일됩니다.

/// parcel 1.0.0
mod v1_0_0 {
    /// 배송 시간대. 이 열거형도 derive(Ord)이므로 배리언트 선언 순서가 곧 비교 순서입니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Window {
        Morning,
        Afternoon,
        Evening,
    }

    impl Window {
        pub fn label(self) -> &'static str {
            match self {
                Window::Morning => "오전",
                Window::Afternoon => "오후",
                Window::Evening => "저녁",
            }
        }
    }

    /// 배송 예약 칸. 날짜가 빠른 칸이 먼저, 같은 날이면 이른 시간대가 먼저입니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Slot {
        pub day: u8,
        pub window: Window,
    }

    /// 라이브러리의 단위 테스트에 해당하는 검사: "날짜가 먼저"라는 약속이 지켜지는가
    pub fn order_contract_holds() -> bool {
        let late_on_day1 = Slot {
            day: 1,
            window: Window::Evening,
        };
        let early_on_day2 = Slot {
            day: 2,
            window: Window::Morning,
        };
        late_on_day1 < early_on_day2
    }
}

/// parcel 1.0.1(비권장): 필드 선언 순서만 바꿨습니다.
mod v1_0_1_bad {
    pub use super::v1_0_0::Window;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Slot {
        pub window: Window, // 앞으로 옮김: 이제 시간대가 먼저 비교됩니다
        pub day: u8,
    }

    pub fn order_contract_holds() -> bool {
        let late_on_day1 = Slot {
            day: 1,
            window: Window::Evening,
        };
        let early_on_day2 = Slot {
            day: 2,
            window: Window::Morning,
        };
        late_on_day1 < early_on_day2
    }
}

/// parcel 1.0.1(권장): 같은 재배치를 하되 비교 순서는 손으로 고정했습니다.
mod v1_0_1_good {
    use std::cmp::Ordering;

    pub use super::v1_0_0::Window;

    // PartialEq/Eq는 필드 순서와 상관없으므로 derive를 그대로 둡니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Slot {
        pub window: Window,
        pub day: u8,
    }

    // 비교 순서를 선언 순서에서 떼어 내 "날짜, 시간대"로 적었습니다.
    // 이제 필드를 어떻게 재배치해도 정렬 결과는 1.0.0과 같습니다.
    impl Ord for Slot {
        fn cmp(&self, other: &Self) -> Ordering {
            (self.day, self.window).cmp(&(other.day, other.window))
        }
    }

    impl PartialOrd for Slot {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    pub fn order_contract_holds() -> bool {
        let late_on_day1 = Slot {
            day: 1,
            window: Window::Evening,
        };
        let early_on_day2 = Slot {
            day: 2,
            window: Window::Morning,
        };
        late_on_day1 < early_on_day2
    }
}

/// 1.0.0 시절에 쓴 사용자 코드: 예약 칸을 정렬해 배송 순서를 정합니다.
macro_rules! user_code {
    ($parcel:ident) => {{
        use $parcel::{Slot, Window};

        let mut slots = [
            Slot {
                day: 2,
                window: Window::Morning,
            },
            Slot {
                day: 1,
                window: Window::Evening,
            },
            Slot {
                day: 2,
                window: Window::Afternoon,
            },
            Slot {
                day: 1,
                window: Window::Morning,
            },
        ];
        slots.sort();
        let order: Vec<String> = slots
            .iter()
            .map(|s| format!("{}일 {}", s.day, s.window.label()))
            .collect();
        // 정렬된 배열의 마지막 칸을 "마지막 배송"으로 안내합니다.
        let last = slots
            .last()
            .map(|s| format!("{}일 {}", s.day, s.window.label()));
        (order.join(", "), last.unwrap_or_default())
    }};
}

fn report(version: &str, result: (String, String), contract: bool) {
    println!("[{version}]");
    println!("  배송 순서: {}", result.0);
    println!("  마지막 배송 안내: {}", result.1);
    println!(
        "  라이브러리의 순서 검사: {}",
        if contract { "통과" } else { "실패" }
    );
}

fn main() {
    report("1.0.0", user_code!(v1_0_0), v1_0_0::order_contract_holds());
    report(
        "1.0.1 비권장: 필드만 재배치",
        user_code!(v1_0_1_bad),
        v1_0_1_bad::order_contract_holds(),
    );
    report(
        "1.0.1 권장: 재배치 + 손으로 쓴 Ord",
        user_code!(v1_0_1_good),
        v1_0_1_good::order_contract_holds(),
    );
    // 출력:
    // [1.0.0]
    //   배송 순서: 1일 오전, 1일 저녁, 2일 오전, 2일 오후
    //   마지막 배송 안내: 2일 오후
    //   라이브러리의 순서 검사: 통과
    // [1.0.1 비권장: 필드만 재배치]
    //   배송 순서: 1일 오전, 2일 오전, 2일 오후, 1일 저녁
    //   마지막 배송 안내: 1일 저녁
    //   라이브러리의 순서 검사: 실패
    // [1.0.1 권장: 재배치 + 손으로 쓴 Ord]
    //   배송 순서: 1일 오전, 1일 저녁, 2일 오전, 2일 오후
    //   마지막 배송 안내: 2일 오후
    //   라이브러리의 순서 검사: 통과
}
