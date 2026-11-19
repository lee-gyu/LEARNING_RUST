// 지침: "비어 있음"이나 "아무것도 고르지 않음"처럼 뜻이 분명한 기본값이 있는 공개 타입에는
//       Default를 구현합니다. 인자 없는 new()가 있다면 Default도 함께 둡니다.
//       반대로 불변식이 있어 아무 값이나 만들면 안 되는 타입에는 Default를 두지 않습니다.
// 비권장: 인쇄 대기열 JobQueue에 new()만 있고 Default가 없음 (사용자가 우회 코드를 씀)
//         1~99만 허용하는 인쇄 부수 Copies에 derive(Default) (생성자가 거부하는 0이 만들어짐)
// 권장:   JobQueue에 derive(Default), Copies에는 Default 대신 이름 붙은 상수
//
// 실행: cargo run -p er-ch10-standard-traits --example 05_default_meaningful

use std::collections::HashMap;
use std::mem;

// ---- 라이브러리 v0: Default가 없는 대기열 ---------------------------------------

pub mod v0 {
    #[derive(Debug)]
    pub struct JobQueue {
        jobs: Vec<&'static str>,
    }

    // 인자 없는 new()가 있는데 Default가 없으면 clippy의 new_without_default(기본 warn)가 알려 줍니다.
    #[expect(
        clippy::new_without_default,
        reason = "Default가 없을 때 사용자가 겪는 일을 보이려고 일부러 뺌"
    )]
    impl JobQueue {
        pub fn new() -> Self {
            JobQueue { jobs: Vec::new() }
        }
    }

    impl JobQueue {
        pub fn push(&mut self, doc: &'static str) {
            self.jobs.push(doc);
        }
        pub fn len(&self) -> usize {
            self.jobs.len()
        }
        pub fn is_empty(&self) -> bool {
            self.jobs.is_empty()
        }
    }
}

// ---- 라이브러리 v1: 빈 대기열이 곧 기본값 -------------------------------------

pub mod v1 {
    // 빈 대기열은 누가 봐도 같은 뜻이므로 derive로 충분합니다.
    #[derive(Debug, Default)]
    pub struct JobQueue {
        jobs: Vec<&'static str>,
    }

    impl JobQueue {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn push(&mut self, doc: &'static str) {
            self.jobs.push(doc);
        }
        pub fn len(&self) -> usize {
            self.jobs.len()
        }
        pub fn is_empty(&self) -> bool {
            self.jobs.is_empty()
        }
    }

    // 열거형은 #[default]로 기본 배리언트를 고릅니다. 빠뜨리면 E0665입니다
    // (compile_fail/e0665_default_enum_without_attr.rs).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Orientation {
        #[default]
        Portrait,
        Landscape,
    }
}

// ---- 불변식이 있는 타입 ---------------------------------------------------------

pub mod bad_copies {
    /// 인쇄 부수. 1~99만 허용합니다.
    // 비권장: derive(Default)는 생성자를 거치지 않고 Copies(0)을 만듭니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Copies(u8);

    impl Copies {
        pub fn new(n: u8) -> Option<Copies> {
            (1..=99).contains(&n).then_some(Copies(n))
        }
        pub fn get(self) -> u8 {
            self.0
        }
    }
}

pub mod copies {
    /// 인쇄 부수. 1~99만 허용합니다.
    // 권장: Default를 두지 않습니다. "흔히 쓰는 값"은 이름이 그 뜻을 말하는 상수로 줍니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Copies(u8);

    impl Copies {
        pub const ONE: Copies = Copies(1);

        pub fn new(n: u8) -> Option<Copies> {
            (1..=99).contains(&n).then_some(Copies(n))
        }
        pub fn get(self) -> u8 {
            self.0
        }
    }
}

// ---- 사용자 코드 -------------------------------------------------------------

// v1의 JobQueue와 Orientation이 Default이므로, 사용자는 자기 타입에 Default를 derive할 수 있습니다.
// v0의 JobQueue를 필드로 두고 derive(Default)를 붙이면 E0277입니다.
#[derive(Debug, Default)]
struct Office {
    queue: v1::JobQueue,
    orientation: v1::Orientation,
}

fn main() {
    println!("[비권장] new()만 있는 대기열: 사용자가 우회합니다");
    let mut floors: HashMap<u8, v0::JobQueue> = HashMap::new();
    floors
        .entry(3)
        .or_insert_with(v0::JobQueue::new)
        .push("회의록.pdf");
    floors
        .entry(3)
        .or_insert_with(v0::JobQueue::new)
        .push("일정표.pdf");
    let mut third = floors.remove(&3).unwrap_or_else(v0::JobQueue::new);
    let sent = mem::replace(&mut third, v0::JobQueue::new());
    println!("  3층에서 {}건을 보냄, 남은 {}건", sent.len(), third.len());
    // 출력:   3층에서 2건을 보냄, 남은 0건

    println!("[권장] Default가 있는 대기열: 표준 API가 그대로 열립니다");
    let mut floors: HashMap<u8, v1::JobQueue> = HashMap::new();
    floors.entry(3).or_default().push("회의록.pdf");
    floors.entry(3).or_default().push("일정표.pdf");
    let mut third = floors.remove(&3).unwrap_or_default();
    let sent = mem::take(&mut third);
    println!("  3층에서 {}건을 보냄, 남은 {}건", sent.len(), third.len());
    // 출력:   3층에서 2건을 보냄, 남은 0건
    let office = Office::default();
    println!(
        "  사용자의 Office::default(): 대기열 비었나 {}, 방향 {:?}",
        office.queue.is_empty(),
        office.orientation
    );
    // 출력:   사용자의 Office::default(): 대기열 비었나 true, 방향 Portrait

    println!("[비권장] 불변식이 있는 타입에 derive(Default)");
    let made = bad_copies::Copies::new(0);
    let defaulted = bad_copies::Copies::default();
    println!(
        "  Copies::new(0) = {made:?}, Copies::default() = {}부",
        defaulted.get()
    );
    // 출력:   Copies::new(0) = None, Copies::default() = 0부

    println!("[권장] Default 대신 이름 붙은 상수");
    let one = copies::Copies::ONE;
    let many = copies::Copies::new(30).unwrap_or(one);
    println!(
        "  Copies::ONE = {}부, Copies::new(30) = {}부, Copies::new(0) = {:?}",
        one.get(),
        many.get(),
        copies::Copies::new(0)
    );
    // 출력:   Copies::ONE = 1부, Copies::new(30) = 30부, Copies::new(0) = None
}
