// 지침 1(교착 상태): 잠금 둘을 두 함수가 반대 순서로 잡으면(잠금 순서 뒤집힘) 교착 상태가 됩니다.
// 함께 맞아야 하는 데이터를 잠금 하나로 묶으면 같은 실행 순서에서도 끝까지 끝납니다.
//
// 호텔 프런트에는 두 표가 있습니다. 장부(투숙객 → 객실)와 배정표(객실 → 투숙객)는 언제나 서로
// 맞아야 합니다. 두 작업을 두 스레드가 동시에 실행합니다.
//   - check_in("서아", 302): 손님이 오면 장부부터 고치고 배정표를 고칩니다.
//   - close_room(301): 점검할 객실을 비울 때는 배정표부터 고치고 장부를 고칩니다.
//
//   1. 비권장: 잠금 둘, 두 작업이 반대 순서로 잡음 → 교착 상태
//   2. 권장: 장부와 배정표를 잠금 하나로 묶음 → 끝남
//   3. 대안: 잠금 둘을 두되 모든 작업이 같은 순서(장부 → 배정표)로 잡음 → 끝남
//
// 세 판 모두 "두 작업이 첫 잠금을 쥔 순간"에 서로를 최대 100ms 기다리게 해서, 실제로는 운이 나쁠 때만
// 생기는 실행 순서를 매번 만듭니다(Meeting). 교착 상태 판의 두 스레드는 끝나지 않으므로 main은
// 1초만 기다린 뒤 "끝났다는 신호가 오지 않음"을 확인하고 넘어갑니다. 멈춘 스레드는 main이 끝날 때
// 프로세스와 함께 사라집니다. 출력은 실행마다 같고, 실행에는 1.5초쯤 걸립니다.
//
// 실행: cargo run -p er-ch17-shared-state-parallelism --example 01_lock_inversion

use std::collections::BTreeMap;
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

type Guests = BTreeMap<String, u32>; // 장부: 투숙객 → 객실
type Rooms = BTreeMap<u32, String>; // 배정표: 객실 → 투숙객

/// 상대가 첫 잠금을 쥘 때까지 기다리는 최대 시간
const PAUSE: Duration = Duration::from_millis(100);
/// main이 두 작업이 끝나기를 기다리는 최대 시간
const LIMIT: Duration = Duration::from_secs(1);

// ---------------------------------------------------------------------------
// 관찰용 장치: 두 작업이 첫 잠금을 쥔 순간에 서로를 기다리게 합니다.
// 상대가 PAUSE 안에 오지 않으면(상대가 잠금을 기다리느라 올 수 없으면) 혼자 계속 진행합니다.
// 실제 코드에는 이런 장치가 없고, 이 틈은 운영체제가 스레드를 바꾸는 순간에 우연히 생깁니다.
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MeetState {
    waiting: u32,
    both: bool,
}

#[derive(Default)]
struct Meeting {
    state: Mutex<MeetState>,
    changed: Condvar,
}

impl Meeting {
    fn pause(&self) {
        let mut st = self.state.lock().unwrap();
        st.waiting += 1;
        if st.waiting == 2 {
            st.both = true; // 두 작업이 동시에 첫 잠금을 쥐었음
            self.changed.notify_all();
        }
        let (mut st, _timeout) = self
            .changed
            .wait_timeout_while(st, PAUSE, |s| !s.both)
            .unwrap();
        st.waiting -= 1;
    }

    fn both_met(&self) -> bool {
        self.state.lock().unwrap().both
    }
}

// ---------------------------------------------------------------------------
// 같은 두 작업의 세 판
// ---------------------------------------------------------------------------

trait FrontDesk: Send + Sync + 'static {
    fn check_in(&self, guest: &str, room: u32);
    fn close_room(&self, room: u32);
    fn status(&self) -> String;
    fn meeting(&self) -> &Meeting;
}

fn initial() -> (Guests, Rooms) {
    let guests = Guests::from([("하준".to_string(), 301)]);
    let rooms = Rooms::from([(301, "하준".to_string())]);
    (guests, rooms)
}

fn describe(guests: &Guests, rooms: &Rooms) -> String {
    format!("장부 {guests:?}, 배정표 {rooms:?}")
}

fn locked_word(free: bool) -> &'static str {
    if free { "풀림" } else { "잠김" }
}

/// 비권장: 장부와 배정표를 따로 잠그고, 두 작업이 반대 순서로 잡습니다.
struct TwoLocksInverted {
    guests: Mutex<Guests>,
    rooms: Mutex<Rooms>,
    meeting: Meeting,
}

impl FrontDesk for TwoLocksInverted {
    fn check_in(&self, guest: &str, room: u32) {
        let mut guests = self.guests.lock().unwrap(); // ① 장부
        guests.insert(guest.to_string(), room);
        self.meeting.pause();
        let mut rooms = self.rooms.lock().unwrap(); // ② 배정표: 스레드 2가 쥐고 있으면 영원히 기다림
        rooms.insert(room, guest.to_string());
    }

    fn close_room(&self, room: u32) {
        let mut rooms = self.rooms.lock().unwrap(); // ① 배정표 ← check_in과 순서가 뒤집힘
        rooms.remove(&room);
        self.meeting.pause();
        let mut guests = self.guests.lock().unwrap(); // ② 장부: 스레드 1이 쥐고 있으면 영원히 기다림
        guests.retain(|_, r| *r != room);
    }

    fn status(&self) -> String {
        match (self.guests.try_lock(), self.rooms.try_lock()) {
            (Ok(g), Ok(r)) => describe(&g, &r),
            (g, r) => format!(
                "장부 {}, 배정표 {}",
                locked_word(g.is_ok()),
                locked_word(r.is_ok())
            ),
        }
    }

    fn meeting(&self) -> &Meeting {
        &self.meeting
    }
}

/// 권장: 함께 맞아야 하는 두 표를 한 구조체에 담고 잠금 하나로 지킵니다.
struct Ledger {
    guests: Guests,
    rooms: Rooms,
}

struct OneLock {
    ledger: Mutex<Ledger>,
    meeting: Meeting,
}

impl FrontDesk for OneLock {
    fn check_in(&self, guest: &str, room: u32) {
        let mut ledger = self.ledger.lock().unwrap(); // 잠금은 하나뿐이라 순서가 없음
        ledger.guests.insert(guest.to_string(), room);
        self.meeting.pause(); // 상대는 잠금을 기다리는 중이라 여기에 올 수 없음
        ledger.rooms.insert(room, guest.to_string());
    }

    fn close_room(&self, room: u32) {
        let mut ledger = self.ledger.lock().unwrap();
        ledger.rooms.remove(&room);
        self.meeting.pause();
        ledger.guests.retain(|_, r| *r != room);
    }

    fn status(&self) -> String {
        match self.ledger.try_lock() {
            Ok(l) => describe(&l.guests, &l.rooms),
            Err(_) => "잠금 잠김".to_string(),
        }
    }

    fn meeting(&self) -> &Meeting {
        &self.meeting
    }
}

/// 대안: 잠금 둘을 두되, 모든 작업이 문서로 정한 같은 순서(장부 → 배정표)로 잡습니다.
struct TwoLocksOrdered {
    guests: Mutex<Guests>, // 잠금 순서 1
    rooms: Mutex<Rooms>,   // 잠금 순서 2: 장부를 쥐지 않은 채 이것만 먼저 잡지 않는다
    meeting: Meeting,
}

impl FrontDesk for TwoLocksOrdered {
    fn check_in(&self, guest: &str, room: u32) {
        let mut guests = self.guests.lock().unwrap();
        guests.insert(guest.to_string(), room);
        self.meeting.pause();
        let mut rooms = self.rooms.lock().unwrap();
        rooms.insert(room, guest.to_string());
    }

    fn close_room(&self, room: u32) {
        let mut guests = self.guests.lock().unwrap(); // 배정표만 필요해 보여도 장부부터
        self.meeting.pause();
        let mut rooms = self.rooms.lock().unwrap();
        rooms.remove(&room);
        guests.retain(|_, r| *r != room);
    }

    fn status(&self) -> String {
        match (self.guests.try_lock(), self.rooms.try_lock()) {
            (Ok(g), Ok(r)) => describe(&g, &r),
            (g, r) => format!(
                "장부 {}, 배정표 {}",
                locked_word(g.is_ok()),
                locked_word(r.is_ok())
            ),
        }
    }

    fn meeting(&self) -> &Meeting {
        &self.meeting
    }
}

// ---------------------------------------------------------------------------
// 두 작업을 동시에 실행하고, 제한 시간 안에 몇 개가 끝났는지 셉니다.
// 스코프 스레드는 스코프가 끝날 때 모든 스레드를 join하므로, 멈춘 스레드가 있으면 main까지 멈춥니다.
// 그래서 멈출 수 있는 작업은 thread::spawn으로 띄우고 Arc로 나눠 줍니다.
// ---------------------------------------------------------------------------

fn run_both<D: FrontDesk>(desk: D) -> (Arc<D>, usize) {
    let desk = Arc::new(desk);
    let (done_tx, done_rx) = mpsc::channel();

    let d1 = Arc::clone(&desk);
    let tx1 = done_tx.clone();
    thread::spawn(move || {
        d1.check_in("서아", 302);
        let _ = tx1.send(()); // 끝났다는 신호
    });

    let d2 = Arc::clone(&desk);
    thread::spawn(move || {
        d2.close_room(301);
        let _ = done_tx.send(());
    });

    let deadline = Instant::now() + LIMIT;
    let mut finished = 0;
    while finished < 2 {
        let left = deadline.saturating_duration_since(Instant::now());
        match done_rx.recv_timeout(left) {
            Ok(()) => finished += 1,
            Err(_) => break, // 제한 시간 안에 신호가 오지 않음
        }
    }
    (desk, finished)
}

fn yes_no(b: bool) -> &'static str {
    if b { "예" } else { "아니오" }
}

fn report<D: FrontDesk>(title: &str, desk: D) {
    println!("{title}");
    let (desk, finished) = run_both(desk);
    let met = desk.meeting().both_met();
    println!("    두 작업이 동시에 첫 잠금을 쥠: {}", yes_no(met));
    let verdict = if finished == 2 {
        "모두 끝남"
    } else {
        "교착 상태"
    };
    println!("    1초 안에 끝난 작업: {finished}/2 → {verdict}");
    println!("    지금: {}", desk.status());
}

fn main() {
    let (guests, rooms) = initial();
    println!("처음: {}", describe(&guests, &rooms));
    println!("동시에 실행: check_in(\"서아\", 302)와 close_room(301)");
    // 출력: 처음: 장부 {"하준": 301}, 배정표 {301: "하준"}
    //       동시에 실행: check_in("서아", 302)와 close_room(301)

    let (guests, rooms) = initial();
    report(
        "1. 비권장: 잠금 둘, check_in은 장부 → 배정표, close_room은 배정표 → 장부",
        TwoLocksInverted {
            guests: Mutex::new(guests),
            rooms: Mutex::new(rooms),
            meeting: Meeting::default(),
        },
    );
    // 출력: 1. 비권장: 잠금 둘, check_in은 장부 → 배정표, close_room은 배정표 → 장부
    //           두 작업이 동시에 첫 잠금을 쥠: 예
    //           1초 안에 끝난 작업: 0/2 → 교착 상태
    //           지금: 장부 잠김, 배정표 잠김

    let (guests, rooms) = initial();
    report(
        "2. 권장: 장부와 배정표를 잠금 하나로 묶음",
        OneLock {
            ledger: Mutex::new(Ledger { guests, rooms }),
            meeting: Meeting::default(),
        },
    );
    // 출력: 2. 권장: 장부와 배정표를 잠금 하나로 묶음
    //           두 작업이 동시에 첫 잠금을 쥠: 아니오
    //           1초 안에 끝난 작업: 2/2 → 모두 끝남
    //           지금: 장부 {"서아": 302}, 배정표 {302: "서아"}

    let (guests, rooms) = initial();
    report(
        "3. 대안: 잠금 둘, 모든 작업이 장부 → 배정표 순서",
        TwoLocksOrdered {
            guests: Mutex::new(guests),
            rooms: Mutex::new(rooms),
            meeting: Meeting::default(),
        },
    );
    // 출력: 3. 대안: 잠금 둘, 모든 작업이 장부 → 배정표 순서
    //           두 작업이 동시에 첫 잠금을 쥠: 아니오
    //           1초 안에 끝난 작업: 2/2 → 모두 끝남
    //           지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
}
