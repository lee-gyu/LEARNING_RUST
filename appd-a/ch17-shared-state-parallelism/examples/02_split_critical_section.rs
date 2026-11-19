// 지침 1(어긋난 상태): 교착 상태를 피하려고 잠금 범위를 좁혀 한 번에 잠금 하나만 쥐면, 교착 상태는
// 사라지지만 함께 맞아야 하는 두 표가 어긋난 순간이 다른 스레드에 보이고, 어긋난 채 남기도 합니다.
//
// 01과 같은 호텔 프런트입니다. 장부(투숙객 → 객실)와 배정표(객실 → 투숙객)는 서로 맞아야 합니다.
//
//   1. check_in("서아", 302)의 두 단계 사이에 현황판(다른 스레드)이 두 표를 읽음
//   2. close_room(301)의 두 단계 사이에 check_in("서아", 301)이 끼어듦
//
// 각 구역에서 [비권장] 잠금 둘(범위를 좁혀 한 번에 하나만 잠금)과 [권장] 잠금 하나를 비교합니다.
// 비권장 판의 close_room은 01처럼 배정표부터 고치고 check_in은 장부부터 고치지만, 잠금을 동시에 둘
// 쥐는 일이 없으므로 교착 상태는 생기지 않습니다.
//
// 끼어드는 시점은 Gap으로 맞춥니다. 지정한 작업이 두 단계 사이에 이르면 다른 작업을 출발시키고,
// 그 작업이 끝나기를 최대 100ms 기다립니다. 잠금 하나 판에서는 다른 작업이 잠금을 기다리느라 끝날
// 수 없으므로 100ms 뒤에 지정한 작업이 마저 진행합니다. 출력은 실행마다 같습니다.
//
// 실행: cargo run -p er-ch17-shared-state-parallelism --example 02_split_critical_section

use std::collections::BTreeMap;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Duration;

type Guests = BTreeMap<String, u32>; // 장부: 투숙객 → 객실
type Rooms = BTreeMap<u32, String>; // 배정표: 객실 → 투숙객

const PAUSE: Duration = Duration::from_millis(100);

// ---------------------------------------------------------------------------
// 관찰용 장치: 지정한 작업의 "두 단계 사이"에 다른 작업이 실행되게 합니다.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Op {
    CheckIn,
    CloseRoom,
}

#[derive(Default)]
struct GapState {
    paused: bool,     // 지정한 작업이 두 단계 사이에 들어섰음
    other_done: bool, // 끼어든 작업이 끝났음
}

struct Gap {
    target: Op,
    state: Mutex<GapState>,
    changed: Condvar,
}

impl Gap {
    fn new(target: Op) -> Gap {
        Gap {
            target,
            state: Mutex::new(GapState::default()),
            changed: Condvar::new(),
        }
    }

    /// 작업의 두 단계 사이에서 부릅니다. 지정한 작업이면 끼어든 작업이 끝나기를 최대 PAUSE 기다립니다.
    fn between_steps(&self, op: Op) {
        if op != self.target {
            return;
        }
        let mut st = self.state.lock().unwrap();
        st.paused = true;
        self.changed.notify_all();
        let _st = self
            .changed
            .wait_timeout_while(st, PAUSE, |s| !s.other_done)
            .unwrap();
    }

    /// 끼어들 작업: 지정한 작업이 두 단계 사이에 들어설 때까지 기다립니다.
    fn wait_for_gap(&self) {
        let st = self.state.lock().unwrap();
        let _st = self.changed.wait_while(st, |s| !s.paused).unwrap();
    }

    fn other_finished(&self) {
        self.state.lock().unwrap().other_done = true;
        self.changed.notify_all();
    }
}

// ---------------------------------------------------------------------------
// 두 판
// ---------------------------------------------------------------------------

trait FrontDesk: Sync {
    fn check_in(&self, guest: &str, room: u32);
    fn close_room(&self, room: u32);
    /// 현황판: 두 표를 읽어 사본을 돌려줍니다.
    fn read_both(&self) -> (Guests, Rooms);
    fn gap(&self) -> &Gap;
}

/// 비권장: 잠금 둘. 교착 상태를 피하려고 한 번에 하나만, 짧게 잠급니다.
struct TwoNarrowLocks {
    guests: Mutex<Guests>,
    rooms: Mutex<Rooms>,
    gap: Gap,
}

impl FrontDesk for TwoNarrowLocks {
    fn check_in(&self, guest: &str, room: u32) {
        self.guests.lock().unwrap().insert(guest.to_string(), room); // 장부만 잠깐
        self.gap.between_steps(Op::CheckIn); // 아무 잠금도 쥐지 않은 틈
        self.rooms.lock().unwrap().insert(room, guest.to_string()); // 배정표만 잠깐
    }

    fn close_room(&self, room: u32) {
        self.rooms.lock().unwrap().remove(&room);
        self.gap.between_steps(Op::CloseRoom);
        self.guests.lock().unwrap().retain(|_, r| *r != room);
    }

    fn read_both(&self) -> (Guests, Rooms) {
        let guests = self.guests.lock().unwrap().clone();
        let rooms = self.rooms.lock().unwrap().clone();
        (guests, rooms)
    }

    fn gap(&self) -> &Gap {
        &self.gap
    }
}

/// 권장: 두 표를 한 구조체에 담고 잠금 하나로 지킵니다. 한 작업의 두 단계는 한 임계 구역 안에 있습니다.
struct Ledger {
    guests: Guests,
    rooms: Rooms,
}

struct OneLock {
    ledger: Mutex<Ledger>,
    gap: Gap,
}

impl FrontDesk for OneLock {
    fn check_in(&self, guest: &str, room: u32) {
        let mut ledger = self.ledger.lock().unwrap();
        ledger.guests.insert(guest.to_string(), room);
        self.gap.between_steps(Op::CheckIn); // 잠금을 쥔 채: 다른 작업은 들어올 수 없음
        ledger.rooms.insert(room, guest.to_string());
    }

    fn close_room(&self, room: u32) {
        let mut ledger = self.ledger.lock().unwrap();
        ledger.rooms.remove(&room);
        self.gap.between_steps(Op::CloseRoom);
        ledger.guests.retain(|_, r| *r != room);
    }

    fn read_both(&self) -> (Guests, Rooms) {
        let ledger = self.ledger.lock().unwrap();
        (ledger.guests.clone(), ledger.rooms.clone())
    }

    fn gap(&self) -> &Gap {
        &self.gap
    }
}

fn initial() -> (Guests, Rooms) {
    let guests = Guests::from([("하준".to_string(), 301)]);
    let rooms = Rooms::from([(301, "하준".to_string())]);
    (guests, rooms)
}

fn two_locks(target: Op) -> TwoNarrowLocks {
    let (guests, rooms) = initial();
    TwoNarrowLocks {
        guests: Mutex::new(guests),
        rooms: Mutex::new(rooms),
        gap: Gap::new(target),
    }
}

fn one_lock(target: Op) -> OneLock {
    let (guests, rooms) = initial();
    OneLock {
        ledger: Mutex::new(Ledger { guests, rooms }),
        gap: Gap::new(target),
    }
}

/// 장부의 모든 투숙객이 배정표의 그 객실에 있고, 배정표에 장부에 없는 사람이 없으면 "서로 맞음"입니다.
fn agree(guests: &Guests, rooms: &Rooms) -> bool {
    guests.len() == rooms.len()
        && guests
            .iter()
            .all(|(name, room)| rooms.get(room) == Some(name))
}

fn show(label: &str, (guests, rooms): &(Guests, Rooms)) {
    let verdict = if agree(guests, rooms) {
        "서로 맞음"
    } else {
        "서로 맞지 않음"
    };
    println!("    {label}: 장부 {guests:?}, 배정표 {rooms:?} → {verdict}");
}

/// 구역 1: check_in의 두 단계 사이에 현황판이 두 표를 읽습니다.
fn reader_in_gap<D: FrontDesk>(title: &str, desk: D) {
    println!("  {title}");
    let seen = thread::scope(|s| {
        s.spawn(|| desk.check_in("서아", 302));
        let board = s.spawn(|| {
            desk.gap().wait_for_gap();
            let seen = desk.read_both();
            desk.gap().other_finished();
            seen
        });
        board.join().unwrap()
    });
    show("현황판이 본 것", &seen);
    show("끝난 뒤", &desk.read_both());
}

/// 구역 2: close_room의 두 단계 사이에 check_in이 끼어듭니다.
fn writer_in_gap<D: FrontDesk>(title: &str, desk: D) {
    println!("  {title}");
    thread::scope(|s| {
        s.spawn(|| desk.close_room(301));
        s.spawn(|| {
            desk.gap().wait_for_gap();
            desk.check_in("서아", 301);
            desk.gap().other_finished();
        });
    });
    show("끝난 뒤", &desk.read_both());
}

fn main() {
    let (guests, rooms) = initial();
    println!("처음: 장부 {guests:?}, 배정표 {rooms:?}");
    // 출력: 처음: 장부 {"하준": 301}, 배정표 {301: "하준"}

    println!("1. check_in(\"서아\", 302)의 두 단계 사이에 현황판이 두 표를 읽음");
    reader_in_gap(
        "[비권장] 잠금 둘, 범위를 좁혀 한 번에 하나만 잠금",
        two_locks(Op::CheckIn),
    );
    reader_in_gap("[권장] 잠금 하나", one_lock(Op::CheckIn));
    // 출력: 1. check_in("서아", 302)의 두 단계 사이에 현황판이 두 표를 읽음
    //         [비권장] 잠금 둘, 범위를 좁혀 한 번에 하나만 잠금
    //           현황판이 본 것: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준"} → 서로 맞지 않음
    //           끝난 뒤: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아"} → 서로 맞음
    //         [권장] 잠금 하나
    //           현황판이 본 것: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아"} → 서로 맞음
    //           끝난 뒤: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아"} → 서로 맞음

    println!("2. close_room(301)의 두 단계 사이에 check_in(\"서아\", 301)이 끼어듦");
    writer_in_gap(
        "[비권장] 잠금 둘, 범위를 좁혀 한 번에 하나만 잠금",
        two_locks(Op::CloseRoom),
    );
    writer_in_gap("[권장] 잠금 하나", one_lock(Op::CloseRoom));
    // 출력: 2. close_room(301)의 두 단계 사이에 check_in("서아", 301)이 끼어듦
    //         [비권장] 잠금 둘, 범위를 좁혀 한 번에 하나만 잠금
    //           끝난 뒤: 장부 {}, 배정표 {301: "서아"} → 서로 맞지 않음
    //         [권장] 잠금 하나
    //           끝난 뒤: 장부 {"서아": 301}, 배정표 {301: "서아"} → 서로 맞음
}
