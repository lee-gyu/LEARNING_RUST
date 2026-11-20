// 기대하는 에러: 에러 코드 없음(기본으로 deny인 린트), open과 close에서 하나씩
//   invalid definition of the runtime `open` symbol used by the standard library
//   `#[deny(invalid_runtime_symbol_definitions)]` on by default
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/runtime_symbol_open.rs
//
// C에 내보내는 로그 라이브러리가 핸들을 여닫는 함수를 접두사 없이 open/close로 지었습니다.
// #[unsafe(no_mangle)]은 이 이름을 프로그램 전체가 함께 쓰는 심볼 이름 공간에 그대로 올리므로,
// C 라이브러리의 open(2)·close(2)와 부딪힙니다. 표준 라이브러리도 파일을 열 때 이 심볼을 부릅니다.
// rustc 1.98부터 표준 라이브러리가 쓰는 일부 심볼(open, close, read, write, free, memcpy 등)을
// 정의하면 알려 줍니다. 인자 수나 반환값 유무가 다르면 이 에러이고, 시그니처가 맞아 보여도
// suspicious_runtime_symbol_definitions 경고가 납니다. 목록에 없는 흔한 이름(init, new,
// destroy, printf 등)은 아무 경고 없이 겹칩니다.
// 해결: 내보내는 이름에 라이브러리 접두사를 붙입니다(applog_open, applog_close).

use std::ffi::c_char;

pub struct Logger {
    pub lines: u32,
}

/// # Safety
/// `path`는 NULL이거나 0 바이트로 끝나는 문자열이어야 합니다.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn open(path: *const c_char) -> *mut Logger {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(Logger { lines: 0 }))
}

/// # Safety
/// `logger`는 NULL이거나 open이 돌려주고 아직 닫지 않은 포인터여야 합니다.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn close(logger: *mut Logger) {
    if !logger.is_null() {
        // SAFETY: logger는 open이 Box::into_raw로 만들었다는 것이 전제 조건입니다.
        drop(unsafe { Box::from_raw(logger) });
    }
}

fn main() {}
