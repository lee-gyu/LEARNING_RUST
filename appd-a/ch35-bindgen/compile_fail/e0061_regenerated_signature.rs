// 기대하는 에러: E0061 this function takes 3 arguments but 2 arguments were supplied
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0061_regenerated_signature.rs
//
// C 라이브러리 v3은 `tally_record`에 `uint32_t weight` 인자를 더했습니다. 헤더에서 선언을 다시
// 생성하면 선언이 인자 3개로 바뀌고, 인자 2개로 부르던 래퍼는 컴파일되지 않습니다.
// 같은 상황에서 손으로 쓴 선언은 인자 2개 그대로 남아 경고 없이 빌드되고, 실행하면 미정의
// 동작입니다(projects/tally_bindings, TALLY_C_VERSION=3). 한 파일로 줄이려고 생성된 선언을
// 모듈로 흉내 냈고, 컴파일이 링크 전에 실패하므로 C 라이브러리는 필요 없습니다.

mod ffi {
    // bindgen 0.73.2가 v3 헤더에서 생성한 줄 그대로입니다.
    #[repr(C)]
    #[derive(Debug)]
    pub struct tally {
        _unused: [u8; 0],
    }
    unsafe extern "C" {
        pub fn tally_record(t: *mut tally, sample: i64, weight: u32) -> i32;
    }
}

/// v1·v2 헤더에 맞춰 쓴 래퍼의 메서드를 그대로 옮겼습니다.
pub fn record(raw: *mut ffi::tally, sample: i64) -> i32 {
    // SAFETY: 이 파일은 컴파일되지 않는 것이 목적이라 이 호출은 실행되지 않습니다.
    unsafe { ffi::tally_record(raw, sample) }
}

fn main() {
    println!("{}", record(std::ptr::null_mut(), 1));
}
