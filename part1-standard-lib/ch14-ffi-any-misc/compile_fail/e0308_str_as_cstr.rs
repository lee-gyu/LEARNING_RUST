// 기대하는 에러: E0308
// 핵심 메시지: mismatched types — expected `&CStr`, found `&str`
//
// &str은 끝에 0 바이트가 없고 중간에 0 바이트가 있을 수도 있으므로 &CStr이 아닙니다.
// 두 타입 사이에는 자동 변환이 없습니다. c"..." 리터럴을 쓰거나 CString::new로 만들어 넘깁니다.
//
// 확인: rustc --edition 2024 compile_fail/e0308_str_as_cstr.rs --out-dir /tmp/rust-p1-ch14

use std::ffi::{CStr, c_char};

unsafe extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

fn c_strlen(s: &CStr) -> usize {
    // SAFETY: &CStr은 0 바이트로 끝나는 살아 있는 버퍼를 가리킵니다.
    unsafe { strlen(s.as_ptr()) }
}

fn main() {
    println!("{}", c_strlen("hello")); // → E0308
}
