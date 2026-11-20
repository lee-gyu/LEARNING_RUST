// 지침 4: 메모리의 할당과 해제는 FFI 경계의 같은 쪽에서 합니다.
//
// 실행: cargo run -p er-ch34-ffi-boundaries --example 03_alloc_free_same_side
//
// Rust 라이브러리가 C에 핸들을 넘기는 상황입니다. C 코드 대신 c_caller 모듈이 C 호출자처럼
// 원시 포인터만 들고 extern "C" 함수를 부릅니다. Session이 버려질 때마다 메시지를 출력하고
// 살아 있는 개수를 세어, 메모리가 언제 해제되는지 관찰합니다.
// 해제된 포인터를 실제로 쓰는 코드(미정의 동작)는 실행하지 않습니다.

use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE: AtomicUsize = AtomicUsize::new(0);

fn live() -> usize {
    LIVE.load(Ordering::SeqCst)
}

/// C에는 불투명 포인터로만 보이는 Rust 타입
pub struct Session {
    user: String,
    hits: u32,
}

impl Session {
    fn new(user: &str) -> Session {
        LIVE.fetch_add(1, Ordering::SeqCst);
        Session {
            user: user.to_string(),
            hits: 0,
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
        println!("      (Session '{}' 해제, 조회 {}번)", self.user, self.hits);
    }
}

mod v1 {
    use super::*;

    /// 비권장: Box를 만들고 그 주소만 돌려줍니다.
    pub extern "C" fn session_new(user_id: u32) -> *mut Session {
        let session = Box::new(Session::new(&format!("v1-{user_id}")));
        ptr::from_ref(&*session).cast_mut()
    } // ← session(Box)이 여기서 버려집니다. 돌려준 주소는 이미 해제된 메모리입니다.
}

mod v2 {
    use super::*;

    /// 권장: Box::into_raw로 소유권을 원시 포인터에 옮겨 C에 넘깁니다.
    /// 이 포인터는 v2::session_free로만 돌려받습니다.
    pub extern "C" fn session_new(user_id: u32) -> *mut Session {
        Box::into_raw(Box::new(Session::new(&format!("v2-{user_id}"))))
    }

    /// # Safety
    /// `s`는 NULL이거나 session_new가 돌려주고 아직 해제하지 않은 포인터여야 합니다.
    pub unsafe extern "C" fn session_touch(s: *mut Session) -> u32 {
        // SAFETY: s는 NULL이거나 살아 있는 Session이라는 것이 이 함수의 전제 조건입니다.
        // NULL은 as_mut()이 None으로 바꿉니다.
        match unsafe { s.as_mut() } {
            Some(session) => {
                session.hits += 1;
                session.hits
            }
            None => 0,
        }
    }

    /// # Safety
    /// `s`는 NULL이거나 session_new가 돌려주고 아직 해제하지 않은 포인터여야 합니다.
    pub unsafe extern "C" fn session_free(s: *mut Session) {
        if s.is_null() {
            return;
        }
        // SAFETY: s는 session_new가 Box::into_raw로 만들었고 아직 해제되지 않았다는 것이
        // 전제 조건입니다. Box로 되돌려 버리면 할당한 쪽(Rust의 할당기)이 해제합니다.
        drop(unsafe { Box::from_raw(s) });
    }
}

/// C 호출자 역할: 원시 포인터만 들고 다니고, 해제는 라이브러리의 해제 함수에 맡깁니다.
mod c_caller {
    use super::*;

    pub fn use_v1() {
        let p = v1::session_new(7);
        // 포인터는 NULL이 아니지만 가리키는 Session은 이미 없습니다. 여기서 v1의 포인터로
        // 무엇을 읽거나 해제하면 해제 후 사용·이중 해제가 되므로, NULL인지만 봅니다.
        println!("   돌려받은 포인터가 NULL인가: {}", p.is_null());
    }

    pub fn use_v2() {
        let p = v2::session_new(8);
        println!("   만든 직후 살아 있는 Session: {}", live());
        // SAFETY: p는 방금 session_new가 돌려준 포인터이고 아직 해제하지 않았습니다.
        unsafe { v2::session_touch(p) };
        // SAFETY: 위와 같습니다.
        let hits = unsafe { v2::session_touch(p) };
        println!("   조회 {hits}번 뒤 살아 있는 Session: {}", live());
        // SAFETY: p는 session_new가 돌려준 포인터이고, 여기서 한 번만 해제합니다.
        unsafe { v2::session_free(p) };
        println!("   session_free 뒤 살아 있는 Session: {}", live());
        // SAFETY: NULL은 허용된 값입니다.
        unsafe { v2::session_free(ptr::null_mut()) };
    }

    pub fn forget_v2() {
        let p = v2::session_new(9);
        println!("   해제 함수를 부르지 않으면 살아 있는 Session: {}", live());
        // 실제 C 프로그램이 해제를 잊었다면 여기서 누수입니다. 예제를 정리하려고 늦게라도 해제합니다.
        // SAFETY: p는 session_new가 돌려준 포인터이고, 여기서 한 번만 해제합니다.
        unsafe { v2::session_free(p) };
    }
}

fn main() {
    println!("1. 비권장: Box를 버리고 주소만 돌려줌");
    c_caller::use_v1();
    println!("   살아 있는 Session: {}", live());
    // 출력:       (Session 'v1-7' 해제, 조회 0번)
    // 출력:    돌려받은 포인터가 NULL인가: false
    // 출력:    살아 있는 Session: 0
    // 해제 메시지가 C 쪽이 포인터를 받기도 전에 나옵니다.

    println!("2. 권장: Box::into_raw로 넘기고, 같은 라이브러리의 해제 함수로 되찾음");
    c_caller::use_v2();
    // 출력:    만든 직후 살아 있는 Session: 1
    // 출력:    조회 2번 뒤 살아 있는 Session: 1
    // 출력:       (Session 'v2-8' 해제, 조회 2번)
    // 출력:    session_free 뒤 살아 있는 Session: 0

    println!("3. 해제 함수를 잊으면 Rust는 대신 해제해 주지 않음");
    c_caller::forget_v2();
    // 출력:    해제 함수를 부르지 않으면 살아 있는 Session: 1
    // 출력:       (Session 'v2-9' 해제, 조회 0번)
}
