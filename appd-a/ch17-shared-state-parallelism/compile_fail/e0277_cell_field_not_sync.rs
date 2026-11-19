// 기대하는 에러: E0277 — `Cell<u32>` cannot be shared between threads safely
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_cell_field_not_sync.rs --out-dir /tmp/er-ch17
//
// Send와 Sync는 자동 트레이트라서 필드가 모두 Sync이면 구조체도 Sync입니다. 반대로 필드 하나가
// Sync가 아니면 구조체 전체가 Sync를 잃습니다. 여기서는 조회 횟수를 세려고 넣은 Cell<u32> 필드
// 하나 때문에 RoomBoard를 Arc로 여러 스레드에 나눠 줄 수 없게 됐습니다. Cell은 원자적이지 않은
// 읽기·쓰기로 값을 바꾸므로, 두 스레드가 함께 고치면 한쪽의 변경이 사라지는 데이터 경쟁이 됩니다.
// 해결: 스레드를 넘는 타입의 그 필드만 AtomicU32로 바꿉니다(note가 제안하는 것). 여러 값을 함께
//       고쳐야 한다면 Mutex로 감쌉니다. 타입을 만든 쪽에서는 assert_send_sync 같은 컴파일 시간 검사로
//       이 변화를 빌드 단계에서 알아챕니다(ch10 2.9절).

use std::cell::Cell;
use std::sync::Arc;
use std::thread;

struct RoomBoard {
    floors: Vec<u32>,  // 층마다 빈 객실 수
    lookups: Cell<u32>, // 조회 횟수: &self 메서드에서 고치려고 Cell을 씀
}

impl RoomBoard {
    fn vacant(&self) -> u32 {
        self.lookups.set(self.lookups.get() + 1);
        self.floors.iter().sum()
    }
}

fn main() {
    let board = Arc::new(RoomBoard {
        floors: vec![3, 0, 5],
        lookups: Cell::new(0),
    });

    let for_lobby = Arc::clone(&board);
    let lobby = thread::spawn(move || for_lobby.vacant()); // E0277

    println!("빈 객실 {}개", lobby.join().unwrap());
    println!("조회 {}번", board.lookups.get());
}
