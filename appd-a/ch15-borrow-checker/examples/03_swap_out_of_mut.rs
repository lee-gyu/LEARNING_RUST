//! 지침 3: `&mut` 뒤에 있는 값은 옮기지 말고 바꿔 끼워 꺼낸다.
//!
//! 식당의 대기 호출판입니다. 호출 중인 손님(`Option`), 오늘 앉은 손님 목록(`Vec`),
//! 기본값이 없는 근무 기록(`Shift`)을 `&mut self` 메서드에서 꺼냅니다.
//! `self.calling`을 그냥 옮기면 E0507이라(compile_fail/e0507_move_out_then_replace.rs),
//! 컴파일러가 제안하는 대로 복제해서 꺼내는 판(비권장)과 자리 표시 값을 넣으며 꺼내는 판(권장)을
//! 같은 순서로 실행해 손님(`Party`)이 몇 번 복제됐는지 비교합니다.
//!
//! 실행: cargo run -p er-ch15-borrow-checker --example 03_swap_out_of_mut

use std::collections::VecDeque;
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};

static PARTY_CLONES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct Party {
    name: String,
    size: u8,
}

// 복제 횟수를 세려고 손으로 구현했습니다. 하는 일은 derive와 같습니다.
impl Clone for Party {
    fn clone(&self) -> Self {
        PARTY_CLONES.fetch_add(1, Ordering::Relaxed);
        Party {
            name: self.name.clone(),
            size: self.size,
        }
    }
}

/// 근무 기록. 근무 이름이 꼭 있어야 해서 `Default`를 구현하지 않습니다.
#[derive(Debug, Clone)]
struct Shift {
    name: &'static str,
    served: u32,
}

impl Shift {
    fn new(name: &'static str) -> Shift {
        Shift { name, served: 0 }
    }
}

struct Waitlist {
    calling: Option<Party>,   // 지금 호출 중인 손님
    waiting: VecDeque<Party>, // 차례를 기다리는 손님
    seated: Vec<Party>,       // 이번 근무에 앉은 손님
    shift: Shift,
}

impl Waitlist {
    fn open(names: &[(&str, u8)]) -> Waitlist {
        Waitlist {
            calling: None,
            waiting: names
                .iter()
                .map(|&(name, size)| Party {
                    name: name.to_string(),
                    size,
                })
                .collect(),
            seated: Vec::new(),
            shift: Shift::new("점심"),
        }
    }

    // ---- 비권장: 복제해서 꺼내고, 원래 자리는 따로 비웁니다 ----

    /// 다음 손님을 부르고, 앞서 불렀지만 오지 않은 손님을 돌려줍니다.
    fn call_next_cloned(&mut self) -> Option<Party> {
        let previous = self.calling.clone(); // E0507의 help가 제안하는 길
        self.calling = self.waiting.pop_front();
        previous
    }

    fn seat_cloned(&mut self) {
        if let Some(party) = &self.calling {
            self.seated.push(party.clone());
            self.shift.served += 1;
        }
        self.calling = None;
    }

    fn end_shift_cloned(&mut self, next: &'static str) -> (Shift, Vec<Party>) {
        let done = self.shift.clone();
        self.shift = Shift::new(next);
        let seated = self.seated.clone(); // 목록 전체를 복제
        self.seated.clear();
        (done, seated)
    }

    // ---- 권장: 꺼내는 순간 그 자리에 다른 값을 채웁니다 ----

    fn call_next(&mut self) -> Option<Party> {
        let next = self.waiting.pop_front();
        mem::replace(&mut self.calling, next) // 새 값을 넣고 예전 값을 받음
    }

    fn seat(&mut self) {
        if let Some(party) = self.calling.take() {
            // None을 남기고 꺼냄
            self.seated.push(party);
            self.shift.served += 1;
        }
    }

    fn end_shift(&mut self, next: &'static str) -> (Shift, Vec<Party>) {
        let done = mem::replace(&mut self.shift, Shift::new(next)); // Default가 없으니 replace
        let seated = mem::take(&mut self.seated); // 빈 Vec이 뜻이 맞는 빈 상태이므로 take
        (done, seated)
    }
}

fn names(parties: &[Party]) -> Vec<&str> {
    parties.iter().map(|p| p.name.as_str()).collect()
}

fn main() {
    let guests = [("김", 2), ("이", 4), ("박", 3), ("최", 2)];

    // 두 판 모두 같은 순서로 부릅니다: 김 호출 → 이 호출(김은 오지 않음) → 이 착석 → 박 호출 → 박 착석
    let mut list = Waitlist::open(&guests);
    list.call_next_cloned();
    let no_show = list.call_next_cloned();
    list.seat_cloned();
    list.call_next_cloned();
    list.seat_cloned();
    let (done, seated) = list.end_shift_cloned("저녁");
    println!("[비권장] clone()으로 꺼내기");
    println!(
        "  오지 않은 손님: {:?}, {} 근무 착석 {}팀 {:?}",
        no_show.map(|p| p.name),
        done.name,
        done.served,
        names(&seated)
    );
    let total: u32 = seated.iter().map(|p| u32::from(p.size)).sum();
    println!("  착석 인원 {total}명, 대기 {}팀", list.waiting.len());
    println!(
        "  Party 복제: {}번",
        PARTY_CLONES.swap(0, Ordering::Relaxed)
    );
    // 출력:   오지 않은 손님: Some("김"), 점심 근무 착석 2팀 ["이", "박"]
    // 출력:   착석 인원 7명, 대기 1팀
    // 출력:   Party 복제: 5번      ← 김 1 + 이 1 + 박 1 + 근무를 마치며 목록 2

    let mut list = Waitlist::open(&guests);
    list.call_next();
    let no_show = list.call_next();
    list.seat();
    list.call_next();
    list.seat();
    let (done, seated) = list.end_shift("저녁");
    println!("[권장] take/replace로 바꿔 끼우기");
    println!(
        "  오지 않은 손님: {:?}, {} 근무 착석 {}팀 {:?}",
        no_show.map(|p| p.name),
        done.name,
        done.served,
        names(&seated)
    );
    let total: u32 = seated.iter().map(|p| u32::from(p.size)).sum();
    println!("  착석 인원 {total}명, 대기 {}팀", list.waiting.len());
    println!(
        "  Party 복제: {}번",
        PARTY_CLONES.swap(0, Ordering::Relaxed)
    );
    println!(
        "  새 근무: {} {}팀, 착석 목록 {}팀",
        list.shift.name,
        list.shift.served,
        list.seated.len()
    );
    // 출력:   오지 않은 손님: Some("김"), 점심 근무 착석 2팀 ["이", "박"]
    // 출력:   착석 인원 7명, 대기 1팀
    // 출력:   Party 복제: 0번
    // 출력:   새 근무: 저녁 0팀, 착석 목록 0팀
}
