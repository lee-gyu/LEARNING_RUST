// 지침 2: 잠금은 도우미 메서드 안에서만 잡고 풉니다. 잠금을 쥔 채 부르는 쪽의 클로저를 부르지 않고,
// MutexGuard를 부르는 쪽에 돌려주지 않습니다.
//
// 호텔 프런트는 01의 권장 판처럼 장부와 배정표를 잠금 하나(Mutex<Ledger>)로 지킵니다. 프런트 타입을
// 쓰는 쪽이 "오늘 떠나는 손님(하준, 도윤)을 체크아웃시키는" 코드를 씁니다.
//
//   1. 비권장: for_each_guest_locked가 잠금을 쥔 채 클로저를 부름 → 클로저 안의 check_out이 같은 잠금을 다시 잠금
//   2. 비권장: ledger()가 MutexGuard를 돌려줌 → 부르는 쪽이 가드를 쥔 채 check_out을 부름
//   3. 권장: for_each_guest가 사본을 만들어 잠금을 푼 뒤에 클로저를 부름
//   4. 권장: 가드 대신 값(사본)을 돌려주는 도우미 guests()
//
// 같은 스레드가 이미 쥔 Mutex를 다시 lock()하면 어떻게 되는지 표준 라이브러리 문서는 정하지 않았고
// "돌아오지 않는다(교착 상태나 패닉)"고만 적습니다. 이 환경(macOS)에서는 멈췄습니다. 그래서 각 구역은
// thread::spawn으로 띄우고 main은 300ms만 기다립니다. 멈춘 스레드는 main이 끝날 때 프로세스와 함께
// 사라집니다. 출력은 실행마다 같습니다.
//
// 실행: cargo run -p er-ch17-shared-state-parallelism --example 03_keep_locks_inside

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, mpsc};
use std::thread;
use std::time::Duration;

const LEAVING_TODAY: [&str; 2] = ["하준", "도윤"];
const LIMIT: Duration = Duration::from_millis(300);

struct Ledger {
    guests: BTreeMap<String, u32>, // 장부: 투숙객 → 객실
    rooms: BTreeMap<u32, String>,  // 배정표: 객실 → 투숙객
}

struct Front {
    ledger: Mutex<Ledger>,
}

impl Front {
    fn new() -> Front {
        let stays = [("하준", 301), ("서아", 302), ("도윤", 303)];
        let mut ledger = Ledger {
            guests: BTreeMap::new(),
            rooms: BTreeMap::new(),
        };
        for (name, room) in stays {
            ledger.guests.insert(name.to_string(), room);
            ledger.rooms.insert(room, name.to_string());
        }
        Front {
            ledger: Mutex::new(ledger),
        }
    }

    /// 도우미: 잠금은 이 메서드 안에서만 잡고 풉니다. 두 표를 함께 고치는 일이 한 임계 구역입니다.
    fn check_out(&self, guest: &str) -> Option<u32> {
        let mut ledger = self.ledger.lock().unwrap();
        let room = ledger.guests.remove(guest)?;
        ledger.rooms.remove(&room);
        Some(room)
    }

    /// 비권장 1: 잠금을 쥔 채 부르는 쪽의 클로저를 부릅니다. 그 클로저가 무엇을 할지 이 타입은 모릅니다.
    fn for_each_guest_locked(&self, mut f: impl FnMut(&str, u32)) {
        let ledger = self.ledger.lock().unwrap();
        for (name, room) in &ledger.guests {
            f(name, *room);
        }
    }

    /// 비권장 2: 가드를 돌려줍니다. 잠금을 언제까지 쥘지, 쥔 채 무엇을 할지가 부르는 쪽으로 넘어갑니다.
    fn ledger(&self) -> MutexGuard<'_, Ledger> {
        self.ledger.lock().unwrap()
    }

    /// 권장: 잠금 안에서는 사본만 만들고, 잠금을 푼 뒤에 클로저를 부릅니다.
    fn for_each_guest(&self, mut f: impl FnMut(&str, u32)) {
        let snapshot = self.guests(); // 잠금은 이 줄 안에서 잡고 풀림
        for (name, room) in &snapshot {
            f(name, *room);
        }
    }

    /// 권장: 가드 대신 값을 돌려줍니다.
    fn guests(&self) -> Vec<(String, u32)> {
        let ledger = self.ledger.lock().unwrap();
        ledger.guests.iter().map(|(n, r)| (n.clone(), *r)).collect()
    }

    /// 관찰용: 기다리지 않고(try_lock) 지금 상태를 봅니다.
    fn status(&self) -> String {
        match self.ledger.try_lock() {
            Ok(l) => format!("장부 {:?}, 배정표 {:?}", l.guests, l.rooms),
            Err(_) => "잠김 (잠금을 쥔 스레드가 멈춰 있음)".to_string(),
        }
    }
}

/// 일을 새 스레드에서 실행하고 LIMIT 안에 끝나면 결과를 돌려줍니다. 끝나지 않으면 None입니다.
fn finish_within<R: Send + 'static>(job: impl FnOnce() -> R + Send + 'static) -> Option<R> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(job());
    });
    rx.recv_timeout(LIMIT).ok()
}

fn report(title: &str, front: &Front, result: Option<Vec<u32>>) {
    println!("{title}");
    match result {
        Some(rooms) => println!("    300ms 안에 끝남: 예, 비운 객실 {rooms:?}"),
        None => {
            println!("    300ms 안에 끝남: 아니오 → check_out이 같은 스레드가 쥔 잠금을 기다림")
        }
    }
    println!("    지금: {}", front.status());
}

fn main() {
    println!("처음: {}", Front::new().status());
    println!("오늘 떠나는 손님: {LEAVING_TODAY:?}");
    // 출력: 처음: 장부 {"도윤": 303, "서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아", 303: "도윤"}
    //       오늘 떠나는 손님: ["하준", "도윤"]

    // 1. 비권장: 잠금을 쥔 채 클로저를 부름
    let front = Arc::new(Front::new());
    let f = Arc::clone(&front);
    let result = finish_within(move || {
        let mut vacated = Vec::new();
        f.for_each_guest_locked(|name, _room| {
            if LEAVING_TODAY.contains(&name) {
                // for_each_guest_locked가 쥔 잠금을 같은 스레드가 다시 잠금
                vacated.extend(f.check_out(name));
            }
        });
        vacated
    });
    report(
        "1. 비권장: for_each_guest_locked가 잠금을 쥔 채 클로저를 부름",
        &front,
        result,
    );
    // 출력: 1. 비권장: for_each_guest_locked가 잠금을 쥔 채 클로저를 부름
    //           300ms 안에 끝남: 아니오 → check_out이 같은 스레드가 쥔 잠금을 기다림
    //           지금: 잠김 (잠금을 쥔 스레드가 멈춰 있음)

    // 2. 비권장: 가드를 돌려받은 쪽이 가드를 쥔 채 다른 메서드를 부름
    let front = Arc::new(Front::new());
    let f = Arc::clone(&front);
    let result = finish_within(move || {
        let ledger = f.ledger(); // 떠날 손님을 고르려고 가드를 받음
        let leaving: Vec<String> = ledger
            .guests
            .keys()
            .filter(|name| LEAVING_TODAY.contains(&name.as_str()))
            .cloned()
            .collect();
        // ledger(가드)는 Drop이 있어 이 클로저가 끝날 때까지 살아 있음
        leaving
            .iter()
            .filter_map(|name| f.check_out(name))
            .collect()
    });
    report(
        "2. 비권장: ledger()가 돌려준 MutexGuard를 쥔 채 check_out을 부름",
        &front,
        result,
    );
    // 출력: 2. 비권장: ledger()가 돌려준 MutexGuard를 쥔 채 check_out을 부름
    //           300ms 안에 끝남: 아니오 → check_out이 같은 스레드가 쥔 잠금을 기다림
    //           지금: 잠김 (잠금을 쥔 스레드가 멈춰 있음)

    // 3. 권장: 같은 모양의 클로저를, 잠금을 푼 뒤에 부르는 for_each_guest에 넘김
    let front = Arc::new(Front::new());
    let f = Arc::clone(&front);
    let result = finish_within(move || {
        let mut vacated = Vec::new();
        f.for_each_guest(|name, _room| {
            if LEAVING_TODAY.contains(&name) {
                vacated.extend(f.check_out(name));
            }
        });
        vacated
    });
    report(
        "3. 권장: for_each_guest가 사본을 만들고 잠금을 푼 뒤 클로저를 부름",
        &front,
        result,
    );
    // 출력: 3. 권장: for_each_guest가 사본을 만들고 잠금을 푼 뒤 클로저를 부름
    //           300ms 안에 끝남: 예, 비운 객실 [303, 301]
    //           지금: 장부 {"서아": 302}, 배정표 {302: "서아"}

    // 4. 권장: 가드 대신 값을 돌려받고, 고치는 일은 도우미 메서드에 맡김
    let front = Arc::new(Front::new());
    let f = Arc::clone(&front);
    let result = finish_within(move || {
        f.guests()
            .into_iter()
            .filter(|(name, _)| LEAVING_TODAY.contains(&name.as_str()))
            .filter_map(|(name, _)| f.check_out(&name))
            .collect()
    });
    report(
        "4. 권장: guests()가 가드 대신 사본을 돌려줌",
        &front,
        result,
    );
    // 출력: 4. 권장: guests()가 가드 대신 사본을 돌려줌
    //           300ms 안에 끝남: 예, 비운 객실 [303, 301]
    //           지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
}
