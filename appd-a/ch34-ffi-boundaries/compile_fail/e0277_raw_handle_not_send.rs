// 기대하는 에러: E0277
//   `NonNull<tally>` cannot be sent between threads safely
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0277_raw_handle_not_send.rs
//
// C 핸들을 감싼 래퍼를 다른 스레드로 옮기려 합니다. 필드가 원시 포인터(NonNull)이면
// Send와 Sync가 자동으로 구현되지 않습니다. C 코드는 Rust의 Send/Sync 검사를 받지 않으므로
// 컴파일러가 "모른다"는 쪽, 즉 막는 쪽을 기본값으로 고른 것입니다.
// 해결: C 라이브러리의 문서가 "핸들을 다른 스레드로 넘겨 써도 된다"고 약속할 때만
//   unsafe impl Send for Tally {}를 // SAFETY: 주석과 함께 적습니다(projects/tally_ffi/src/lib.rs).
//   "같은 핸들을 여러 스레드에서 동시에 써도 된다"는 약속까지 있어야 Sync도 구현합니다.

use std::ptr::NonNull;
use std::thread;

// C 헤더의 `typedef struct tally tally;`를 옮긴 불투명 타입
#[repr(C)]
pub struct tally {
    _opaque: [u8; 0],
}

pub struct Tally {
    raw: NonNull<tally>,
}

impl Tally {
    // 실제로는 C의 tally_new가 돌려준 포인터를 담습니다. 이 파일은 컴파일만 하므로 흉내만 냅니다.
    fn new() -> Tally {
        Tally {
            raw: NonNull::dangling(),
        }
    }

    fn is_open(&self) -> bool {
        self.raw != NonNull::dangling()
    }
}

fn main() {
    let t = Tally::new();
    let worker = thread::spawn(move || t.is_open());
    println!("{:?}", worker.join());
}
