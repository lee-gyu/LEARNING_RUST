// 지침 4: 잠금을 합칠 수 없어 잠금 순서를 정해야 한다면(마지막 수단), 순서를 주석에만 적지 말고
// 잠글 때마다 검사합니다. 교착 상태는 운 나쁜 실행 순서에서만 나타나므로 보통의 테스트로는 잡히지
// 않지만, 순서 위반은 한 스레드로 한 번만 실행해도 드러납니다.
//
// 01의 비권장 판과 같은 호텔 프런트입니다. 정한 순서는 "장부(1) → 배정표(2)"이고, close_room은
// 배정표부터 잡아서 이 순서를 어깁니다.
//
//   1. 비권장: 순서를 주석으로만 정함 → 한 스레드에서 차례로 실행하는 테스트는 통과
//   2. 권장: 잠금마다 순위를 붙이고(RankedMutex) 잠글 때 검사 → 같은 테스트가 close_room의 위반을 찾음
//   3. 고친 close_room(장부부터 잠금)은 검사를 통과
//
// 스레드는 쓰지 않습니다. 출력은 실행마다 같습니다.
//
// 실행: cargo run -p er-ch17-shared-state-parallelism --example 05_checked_lock_order

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

type Guests = BTreeMap<String, u32>; // 장부: 투숙객 → 객실
type Rooms = BTreeMap<u32, String>; // 배정표: 객실 → 투숙객

// ---------------------------------------------------------------------------
// 1. 비권장: 순서를 주석으로만 정한 프런트
// ---------------------------------------------------------------------------

/// 잠금 순서: 장부 → 배정표. (이 주석을 어겨도 컴파일러도 테스트도 알려 주지 않습니다.)
struct CommentOnlyFront {
    guests: Mutex<Guests>,
    rooms: Mutex<Rooms>,
}

impl CommentOnlyFront {
    fn check_in(&self, guest: &str, room: u32) {
        let mut guests = self.guests.lock().unwrap();
        let mut rooms = self.rooms.lock().unwrap();
        guests.insert(guest.to_string(), room);
        rooms.insert(room, guest.to_string());
    }

    fn close_room(&self, room: u32) {
        let mut rooms = self.rooms.lock().unwrap(); // 순서 위반: 배정표를 먼저 잡음
        let mut guests = self.guests.lock().unwrap();
        rooms.remove(&room);
        guests.retain(|_, r| *r != room);
    }
}

// ---------------------------------------------------------------------------
// 2. 권장: 순위를 붙인 잠금. 이 스레드가 쥔 잠금보다 순위가 낮거나 같은 잠금을 잡으려 하면 Err.
// ---------------------------------------------------------------------------

thread_local! {
    /// 이 스레드가 지금 쥐고 있는 잠금들의 (순위, 이름)
    static HELD: RefCell<Vec<(u32, &'static str)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug)]
struct OrderError {
    holding: (u32, &'static str),
    wanted: (u32, &'static str),
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ((hr, hn), (wr, wn)) = (self.holding, self.wanted);
        write!(f, "{hn}({hr})를 쥔 채 {wn}({wr})를 잠그려 함")
    }
}

struct RankedMutex<T> {
    rank: u32,
    name: &'static str,
    inner: Mutex<T>,
}

struct RankedGuard<'a, T> {
    rank: u32,
    guard: MutexGuard<'a, T>,
}

impl<T> RankedMutex<T> {
    fn new(rank: u32, name: &'static str, value: T) -> RankedMutex<T> {
        RankedMutex {
            rank,
            name,
            inner: Mutex::new(value),
        }
    }

    /// 순위를 검사한 뒤 잠급니다. 검사는 이 스레드가 쥔 잠금만 보므로 다른 스레드와 상관없이 결정적입니다.
    fn lock(&self) -> Result<RankedGuard<'_, T>, OrderError> {
        let highest = HELD.with_borrow(|held| held.iter().copied().max());
        if let Some(holding) = highest
            && holding.0 >= self.rank
        {
            return Err(OrderError {
                holding,
                wanted: (self.rank, self.name),
            });
        }
        let guard = self.inner.lock().unwrap();
        HELD.with_borrow_mut(|held| held.push((self.rank, self.name)));
        Ok(RankedGuard {
            rank: self.rank,
            guard,
        })
    }
}

impl<T> Drop for RankedGuard<'_, T> {
    fn drop(&mut self) {
        // 가드를 잡은 순서와 다르게 버려도 되도록, 같은 순위의 기록을 뒤에서부터 찾아 지웁니다.
        // 이 drop이 끝난 뒤에 guard 필드가 버려지며 실제 잠금이 풀립니다.
        HELD.with_borrow_mut(|held| {
            if let Some(i) = held.iter().rposition(|(r, _)| *r == self.rank) {
                held.remove(i);
            }
        });
    }
}

impl<T> Deref for RankedGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard
    }
}

impl<T> DerefMut for RankedGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.guard
    }
}

/// 잠금 순서를 타입에 적은 프런트: 장부(1) → 배정표(2)
struct CheckedFront {
    guests: RankedMutex<Guests>,
    rooms: RankedMutex<Rooms>,
}

impl CheckedFront {
    fn check_in(&self, guest: &str, room: u32) -> Result<(), OrderError> {
        let mut guests = self.guests.lock()?;
        let mut rooms = self.rooms.lock()?;
        guests.insert(guest.to_string(), room);
        rooms.insert(room, guest.to_string());
        Ok(())
    }

    /// 01과 같은 실수: 배정표부터 잡습니다.
    fn close_room(&self, room: u32) -> Result<(), OrderError> {
        let mut rooms = self.rooms.lock()?;
        let mut guests = self.guests.lock()?; // 여기서 Err
        rooms.remove(&room);
        guests.retain(|_, r| *r != room);
        Ok(())
    }

    /// 고친 판: 배정표만 필요해 보여도 장부부터 잡습니다.
    fn close_room_fixed(&self, room: u32) -> Result<(), OrderError> {
        let mut guests = self.guests.lock()?;
        let mut rooms = self.rooms.lock()?;
        rooms.remove(&room);
        guests.retain(|_, r| *r != room);
        Ok(())
    }

    fn describe(&self) -> String {
        match (self.guests.lock(), self.rooms.lock()) {
            (Ok(g), Ok(r)) => format!("장부 {:?}, 배정표 {:?}", *g, *r),
            _ => "순서 위반".to_string(),
        }
    }
}

fn initial() -> (Guests, Rooms) {
    let guests = Guests::from([("하준".to_string(), 301)]);
    let rooms = Rooms::from([(301, "하준".to_string())]);
    (guests, rooms)
}

fn shown(result: &Result<(), OrderError>) -> String {
    match result {
        Ok(()) => "Ok".to_string(),
        Err(e) => format!("Err({e})"),
    }
}

fn main() {
    println!("1. 비권장: 잠금 순서를 주석으로만 정함 (장부 → 배정표)");
    let (guests, rooms) = initial();
    let front = CommentOnlyFront {
        guests: Mutex::new(guests),
        rooms: Mutex::new(rooms),
    };
    // 한 스레드에서 차례로 부르는 "단위 테스트". 두 잠금을 동시에 노리는 다른 스레드가 없으므로 언제나 통과합니다.
    front.check_in("서아", 302);
    front.close_room(301);
    let guests = front.guests.lock().unwrap().clone();
    let rooms = front.rooms.lock().unwrap().clone();
    println!("    check_in, close_room 차례로 실행: 모두 끝남 → 테스트 통과");
    println!("    장부 {guests:?}, 배정표 {rooms:?}");
    // 출력: 1. 비권장: 잠금 순서를 주석으로만 정함 (장부 → 배정표)
    //           check_in, close_room 차례로 실행: 모두 끝남 → 테스트 통과
    //           장부 {"서아": 302}, 배정표 {302: "서아"}

    println!("2. 권장: 잠금마다 순위를 붙이고 잠글 때 검사");
    let (guests, rooms) = initial();
    let front = CheckedFront {
        guests: RankedMutex::new(1, "장부", guests),
        rooms: RankedMutex::new(2, "배정표", rooms),
    };
    println!(
        "    check_in(\"서아\", 302): {}",
        shown(&front.check_in("서아", 302))
    );
    println!("    close_room(301): {}", shown(&front.close_room(301)));
    println!("    지금: {}", front.describe());
    // 출력: 2. 권장: 잠금마다 순위를 붙이고 잠글 때 검사
    //           check_in("서아", 302): Ok
    //           close_room(301): Err(배정표(2)를 쥔 채 장부(1)를 잠그려 함)
    //           지금: 장부 {"서아": 302, "하준": 301}, 배정표 {301: "하준", 302: "서아"}

    println!("3. 고친 close_room: 장부부터 잠금");
    println!(
        "    close_room(301): {}",
        shown(&front.close_room_fixed(301))
    );
    println!("    지금: {}", front.describe());
    let left = HELD.with_borrow(|held| held.len());
    println!("    이 스레드가 쥔 잠금 기록: {left}개");
    // 출력: 3. 고친 close_room: 장부부터 잠금
    //           close_room(301): Ok
    //           지금: 장부 {"서아": 302}, 배정표 {302: "서아"}
    //           이 스레드가 쥔 잠금 기록: 0개
}
