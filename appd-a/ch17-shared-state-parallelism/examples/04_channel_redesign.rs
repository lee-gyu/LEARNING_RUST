// 지침 3: 가능하면 공유 상태를 두지 않습니다. 장부와 배정표를 프런트 스레드 하나가 소유하고, 다른
// 스레드는 채널로 요청을 보내 답을 받습니다. 잠금이 없으니 잠금 순서도, 두 단계 사이의 틈도 없습니다.
//
//   1. 01의 두 작업(check_in, close_room)을 두 스레드가 같은 순간에 요청 → 둘 다 끝남
//   2. 02의 현황판: 두 스레드가 체크인과 점검을 500번씩 되풀이하는 동안 현황판이 500번 읽음
//      → 어긋난 상태를 본 횟수 0
//   3. 송신자를 모두 버리면 프런트 스레드의 수신 반복이 끝나고, 스레드가 장부를 돌려줌
//
// 프런트 스레드는 요청 하나를 끝까지 처리한 뒤에 다음 요청을 꺼냅니다. 요청 하나가 곧 임계 구역
// 하나입니다. 손님 스레드들은 스코프 스레드이고, 결과는 main에서 출력하므로 출력은 실행마다 같습니다.
//
// 실행: cargo run -p er-ch17-shared-state-parallelism --example 04_channel_redesign

use std::collections::BTreeMap;
use std::sync::Barrier;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

type Guests = BTreeMap<String, u32>; // 장부: 투숙객 → 객실
type Rooms = BTreeMap<u32, String>; // 배정표: 객실 → 투숙객

/// 장부와 배정표. 프런트 스레드만 소유하므로 잠금이 없고, 메서드는 평범한 &mut self입니다.
struct Ledger {
    guests: Guests,
    rooms: Rooms,
}

impl Ledger {
    fn check_in(&mut self, guest: String, room: u32) {
        self.guests.insert(guest.clone(), room);
        self.rooms.insert(room, guest);
    }

    /// 객실을 비우고 내보낸 손님의 이름을 돌려줍니다.
    fn close_room(&mut self, room: u32) -> Vec<String> {
        self.rooms.remove(&room);
        let leaving: Vec<String> = self
            .guests
            .iter()
            .filter(|(_, r)| **r == room)
            .map(|(name, _)| name.clone())
            .collect();
        self.guests.retain(|_, r| *r != room);
        leaving
    }
}

fn agree(guests: &Guests, rooms: &Rooms) -> bool {
    guests.len() == rooms.len()
        && guests
            .iter()
            .all(|(name, room)| rooms.get(room) == Some(name))
}

/// 프런트 스레드에 보내는 요청. 답이 필요한 요청은 답을 받을 송신자를 함께 보냅니다.
enum Request {
    CheckIn {
        guest: String,
        room: u32,
        reply: Sender<()>,
    },
    CloseRoom {
        room: u32,
        reply: Sender<Vec<String>>,
    },
    Snapshot {
        reply: Sender<(Guests, Rooms)>,
    },
}

/// 프런트 스레드의 본문. 송신자가 모두 사라지면 반복이 끝나고 장부와 처리한 요청 수를 돌려줍니다.
fn run_front(inbox: Receiver<Request>, mut ledger: Ledger) -> (Ledger, usize) {
    let mut handled = 0;
    for request in inbox {
        match request {
            Request::CheckIn { guest, room, reply } => {
                ledger.check_in(guest, room);
                let _ = reply.send(()); // 요청한 쪽이 답을 기다리지 않고 떠났어도 무시
            }
            Request::CloseRoom { room, reply } => {
                let _ = reply.send(ledger.close_room(room));
            }
            Request::Snapshot { reply } => {
                let _ = reply.send((ledger.guests.clone(), ledger.rooms.clone()));
            }
        }
        handled += 1;
    }
    (ledger, handled)
}

/// 손님 쪽 손잡이. 요청을 보내고 답을 기다리는 일을 메서드 안에 감춰, 부르는 쪽은 채널을 보지 않습니다.
struct FrontHandle {
    tx: Sender<Request>,
}

impl FrontHandle {
    fn check_in(&self, guest: &str, room: u32) {
        let (reply, answer) = mpsc::channel();
        let guest = guest.to_string();
        self.tx
            .send(Request::CheckIn { guest, room, reply })
            .expect("프런트 스레드가 살아 있음");
        answer.recv().expect("프런트가 답함");
    }

    fn close_room(&self, room: u32) -> Vec<String> {
        let (reply, answer) = mpsc::channel();
        self.tx
            .send(Request::CloseRoom { room, reply })
            .expect("프런트 스레드가 살아 있음");
        answer.recv().expect("프런트가 답함")
    }

    fn snapshot(&self) -> (Guests, Rooms) {
        let (reply, answer) = mpsc::channel();
        self.tx
            .send(Request::Snapshot { reply })
            .expect("프런트 스레드가 살아 있음");
        answer.recv().expect("프런트가 답함")
    }
}

fn show(label: &str, (guests, rooms): &(Guests, Rooms)) {
    let verdict = if agree(guests, rooms) {
        "서로 맞음"
    } else {
        "서로 맞지 않음"
    };
    println!("    {label}: 장부 {guests:?}, 배정표 {rooms:?} → {verdict}");
}

fn main() {
    let (tx, inbox) = mpsc::channel();
    let ledger = Ledger {
        guests: Guests::from([("하준".to_string(), 301)]),
        rooms: Rooms::from([(301, "하준".to_string())]),
    };
    // 프런트 스레드는 장부의 유일한 소유자입니다. 장부를 옮겨 주므로 Arc도 Mutex도 필요 없습니다.
    let front = thread::spawn(move || run_front(inbox, ledger));
    let handle = FrontHandle { tx };

    println!("1. 01의 두 작업을 두 스레드가 같은 순간에 요청");
    show("처음", &handle.snapshot());
    let start = Barrier::new(2);
    let sent_out = thread::scope(|s| {
        s.spawn(|| {
            start.wait();
            handle.check_in("서아", 302);
        });
        let b = s.spawn(|| {
            start.wait();
            handle.close_room(301)
        });
        b.join().unwrap()
    }); // 스코프가 끝나면 두 요청이 모두 끝났음
    println!("    close_room(301)이 내보낸 손님: {sent_out:?}");
    show("지금", &handle.snapshot());
    // 출력: 1. 01의 두 작업을 두 스레드가 같은 순간에 요청
    //           처음: 장부 {"하준": 301}, 배정표 {301: "하준"} → 서로 맞음
    //           close_room(301)이 내보낸 손님: ["하준"]
    //           지금: 장부 {"서아": 302}, 배정표 {302: "서아"} → 서로 맞음

    println!("2. 체크인과 점검을 500번씩 되풀이하는 동안 현황판이 500번 읽음");
    let mismatched = thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..500 {
                handle.check_in("도윤", 401);
                handle.close_room(401);
            }
        });
        s.spawn(|| {
            for _ in 0..500 {
                handle.check_in("지아", 402);
                handle.close_room(402);
            }
        });
        let board = s.spawn(|| {
            (0..500)
                .filter(|_| {
                    let (guests, rooms) = handle.snapshot();
                    !agree(&guests, &rooms)
                })
                .count()
        });
        board.join().unwrap()
    });
    println!("    현황판이 어긋난 상태를 본 횟수: {mismatched}/500");
    show("지금", &handle.snapshot());
    // 출력: 2. 체크인과 점검을 500번씩 되풀이하는 동안 현황판이 500번 읽음
    //           현황판이 어긋난 상태를 본 횟수: 0/500
    //           지금: 장부 {"서아": 302}, 배정표 {302: "서아"} → 서로 맞음

    println!("3. 송신자를 모두 버리면 프런트 스레드가 끝나며 장부를 돌려줌");
    drop(handle); // 마지막 송신자. 이것이 없으면 run_front의 for 반복이 끝나지 않음
    let (ledger, handled) = front.join().unwrap();
    println!("    처리한 요청: {handled}개");
    show("돌려받은 장부", &(ledger.guests, ledger.rooms));
    // 출력: 3. 송신자를 모두 버리면 프런트 스레드가 끝나며 장부를 돌려줌
    //           처리한 요청: 2505개
    //           돌려받은 장부: 장부 {"서아": 302}, 배정표 {302: "서아"} → 서로 맞음
    // 2505개 = 구역 1의 4개(조회 2, 체크인 1, 점검 1) + 구역 2의 2501개(체크인·점검 2000, 현황판 500, 조회 1)
}
