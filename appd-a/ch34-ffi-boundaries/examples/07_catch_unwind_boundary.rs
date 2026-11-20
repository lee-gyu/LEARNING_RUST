// 지침 7: 패닉이 FFI 경계를 넘지 못하게 막습니다. 내보내는 함수의 본문을 catch_unwind로
// 감싸 패닉을 에러 코드로 바꿉니다.
//
// 실행: cargo run -p er-ch34-ffi-boundaries --example 07_catch_unwind_boundary
//
// 일부러 낸 패닉 두 번의 메시지가 표준 에러에 나옵니다(기본 패닉 훅). 메시지 안의 스레드
// 번호는 실행마다 다릅니다. extern "C" 함수 밖으로 패닉이 나가면 프로세스가 중단되므로
// 비권장 판에는 패닉하는 입력을 넣지 않습니다. 중단되는 모습은 ch18의 projects/panic_strategy에 있습니다.

use std::ffi::{CStr, c_char};
use std::panic::{self, UnwindSafe};
use std::ptr;

pub const ER34_OK: i32 = 0;
pub const ER34_ERR_NULL: i32 = -1;
pub const ER34_ERR_PANIC: i32 = -99;

const LABELS: [&CStr; 4] = [c"off", c"low", c"mid", c"high"];

/// 안쪽 Rust 코드. 0..=3만 들어온다고 믿은 버그가 있습니다(4 이상이면 인덱스 패닉).
fn label_of(level: u32) -> &'static CStr {
    LABELS[level as usize]
}

mod v1 {
    use super::*;

    /// 비권장: 본문을 감싸지 않았습니다. label_of가 패닉하면 그 패닉이 extern "C" 경계에
    /// 닿고, 1.81부터는 그 자리에서 프로세스가 중단됩니다(그 전에는 미정의 동작).
    ///
    /// # Safety
    /// `out`은 NULL이거나 `*const c_char` 하나를 쓸 수 있는 정렬된 메모리를 가리켜야 합니다
    /// (출력 매개변수라서 초기화돼 있지 않아도 됩니다).
    pub unsafe extern "C" fn level_label(level: u32, out: *mut *const c_char) -> i32 {
        if out.is_null() {
            return ER34_ERR_NULL;
        }
        let label = label_of(level).as_ptr();
        // SAFETY: out이 NULL이 아님을 위에서 확인했고, 쓸 수 있는 정렬된 메모리라는 것이 전제
        // 조건입니다. 참조를 만들지 않고 write로 쓰므로 초기화돼 있지 않아도 됩니다.
        unsafe { out.write(label) };
        ER34_OK
    }
}

mod v2 {
    use super::*;

    /// 경계 안에서 패닉을 붙잡아 에러 코드로 바꿉니다.
    fn ffi_guard(body: impl FnOnce() -> i32 + UnwindSafe) -> i32 {
        panic::catch_unwind(body).unwrap_or(ER34_ERR_PANIC)
    }

    /// 권장: 접두사를 붙이고, 본문 전체를 ffi_guard 안에서 실행합니다.
    ///
    /// # Safety
    /// `out`은 NULL이거나 `*const c_char` 하나를 쓸 수 있는 정렬된 메모리를 가리켜야 합니다
    /// (출력 매개변수라서 초기화돼 있지 않아도 됩니다).
    pub unsafe extern "C" fn er34_level_label(level: u32, out: *mut *const c_char) -> i32 {
        ffi_guard(|| {
            if out.is_null() {
                return ER34_ERR_NULL;
            }
            // label_of가 패닉하면 쓰기 전에 빠져나가므로, C 쪽의 값은 바뀌지 않습니다.
            let label = label_of(level).as_ptr();
            // SAFETY: out이 NULL이 아님을 위에서 확인했고, 쓸 수 있는 정렬된 메모리라는 것이 전제
            // 조건입니다. 참조를 만들지 않고 write로 쓰므로 초기화돼 있지 않아도 됩니다.
            unsafe { out.write(label) };
            ER34_OK
        })
    }
}

mod v3 {
    use super::*;

    /// 되감기를 허용하는 ABI. 부르는 쪽이 되감기를 처리할 수 있을 때만 씁니다
    /// (예외 처리를 켜고 컴파일한 C++ 코드, 또는 이 예제처럼 Rust).
    pub extern "C-unwind" fn level_label_unwind(level: u32) -> *const c_char {
        label_of(level).as_ptr()
    }
}

fn show(label: *const c_char) -> String {
    // SAFETY: 이 예제의 함수들은 label에 LABELS의 정적 C 문자열 주소만 씁니다.
    let s = unsafe { CStr::from_ptr(label) };
    s.to_string_lossy().into_owned()
}

fn main() {
    println!("1. 비권장: 감싸지 않은 extern \"C\" 함수 (정상 입력만)");
    let mut label: *const c_char = c"(없음)".as_ptr();
    // SAFETY: &mut label은 살아 있는 지역 변수입니다.
    let status = unsafe { v1::level_label(2, &mut label) };
    println!("   level_label(2) = {status}, {:?}", show(label));
    // 출력:    level_label(2) = 0, "mid"
    // level_label(9)를 부르면 패닉이 경계에 닿아 프로세스가 중단됩니다.

    println!("2. 권장: catch_unwind로 감싼 함수");
    for level in [2, 9] {
        let mut label: *const c_char = c"(없음)".as_ptr();
        // SAFETY: &mut label은 살아 있는 지역 변수입니다.
        let status = unsafe { v2::er34_level_label(level, &mut label) };
        println!("   er34_level_label({level}) = {status}, {:?}", show(label));
    }
    // SAFETY: NULL은 허용된 값입니다.
    let status = unsafe { v2::er34_level_label(1, ptr::null_mut()) };
    println!("   er34_level_label(1, NULL) = {status}");
    // 출력:    er34_level_label(2) = 0, "mid"
    // 출력:    er34_level_label(9) = -99, "(없음)"
    // 출력:    er34_level_label(1, NULL) = -1
    // (표준 에러) thread 'main' (스레드 번호) panicked at …/07_catch_unwind_boundary.rs:22:5:
    //            index out of bounds: the len is 4 but the index is 9

    println!("3. extern \"C-unwind\": 패닉이 경계를 넘어 부르는 쪽으로 되감김");
    let ok = panic::catch_unwind(|| show(v3::level_label_unwind(3)));
    let unwound = panic::catch_unwind(|| show(v3::level_label_unwind(9)));
    println!("   level_label_unwind(3): {ok:?}");
    println!(
        "   level_label_unwind(9): 부르는 쪽에서 붙잡음 = {}",
        unwound.is_err()
    );
    // 출력:    level_label_unwind(3): Ok("high")
    // 출력:    level_label_unwind(9): 부르는 쪽에서 붙잡음 = true
}
