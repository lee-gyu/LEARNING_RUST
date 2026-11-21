//! C 라이브러리 libfletcher의 함수 선언만 둡니다(`-sys` 패키지의 관례).
//! 안전한 API는 이 패키지에 의존하는 `fletcher` 패키지가 만듭니다.

use std::ffi::c_int;

unsafe extern "C" {
    /// `data`부터 `len`바이트의 Fletcher-16 체크섬을 돌려줍니다.
    pub fn fletcher16(data: *const u8, len: usize) -> u16;

    /// 체크섬이 `expected`와 같으면 1, 다르면 0. C 라이브러리 버전 2부터 있습니다.
    pub fn fletcher16_verify(data: *const u8, len: usize, expected: u16) -> c_int;
}
