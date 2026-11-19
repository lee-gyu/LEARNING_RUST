// 에디션별 결과: 2024에서 실패, 2021에서 통과(경고도 없음)
// 기대하는 에러(2024): E 코드 없음 — 두 가지
//   extern blocks must be unsafe
//   unsafe attribute used without unsafe
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/unsafe_missing_on_ffi.rs --out-dir /tmp/rust-p2-ch4   # 실패
//   rustc --edition 2021 compile_fail/unsafe_missing_on_ffi.rs --out-dir /tmp/rust-p2-ch4   # 통과
//
// 2021 에디션까지의 FFI 코드 모양입니다. 2024 에디션에서는
// - `extern` 블록 앞에 `unsafe`를 붙여야 합니다(선언이 실제 C 함수와 맞다는 보증).
// - `no_mangle`, `export_name`, `link_section` 속성은 `unsafe(...)`로 감싸야 합니다
//   (심볼 이름이 다른 라이브러리와 겹치지 않는다는 보증).
// 해결: `unsafe extern "C" { ... }`, `#[unsafe(no_mangle)]`. 두 모양 모두 2021 에디션에서도
// 컴파일되므로, 에디션을 올리기 전에 미리 고쳐 둘 수 있습니다.

use std::ffi::c_int;

extern "C" {
    fn abs(x: c_int) -> c_int;
} // → 2024에서 "extern blocks must be unsafe"

#[no_mangle] // → 2024에서 "unsafe attribute used without unsafe"
pub extern "C" fn p2ch4_add_one(x: c_int) -> c_int {
    x + 1
}

fn main() {
    // SAFETY: -3은 INT_MIN이 아니므로 abs의 결과가 정의되어 있습니다.
    let value = unsafe { abs(-3) };
    println!("{}", p2ch4_add_one(value));
}
