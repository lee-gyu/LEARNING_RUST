//! `csrc/v1/tally.h`의 선언을 손으로 옮긴 모듈입니다(ch34의 `src/ffi.rs`와 같은 내용).
//!
//! 이 파일이 C 쪽과 어긋나도 컴파일러와 링커는 알려 주지 않습니다. 링커는 함수 이름만
//! 맞춰 보기 때문입니다. 그래서 C 선언을 옮긴 코드는 이 한 파일에 모으고, 헤더의 각
//! 선언을 바로 위 주석에 적어 나란히 대조할 수 있게 했습니다. 이 파일은 v1 헤더를 보고
//! 썼으므로, `TALLY_C_VERSION=2`나 `3`으로 빌드하면 C와 어긋난 채로 조용히 컴파일됩니다.
//! 같은 헤더에서 bindgen으로 생성한 선언은 `generated/tally-sys`에 있습니다.
//!
//! 이름은 헤더의 C 이름을 그대로 씁니다(대조하기 쉽게). rustc의 `non_camel_case_types`
//! 린트는 `#[repr(C)]`를 붙인 타입에는 경고하지 않으므로 `tally_config` 같은 이름도
//! 경고 없이 컴파일됩니다. 이 모듈은 비공개이고, 바깥에는 `lib.rs`의 안전한 래퍼만 보입니다.

use core::ffi::c_char;

// #define TALLY_MAX_BUCKETS 32
pub const TALLY_MAX_BUCKETS: u32 = 32;
// #define TALLY_NAME_MAX 31
pub const TALLY_NAME_MAX: usize = 31;

// enum tally_status { TALLY_OK = 0, TALLY_ERR_NULL = -1, TALLY_ERR_NAME_TOO_LONG = -2 };
// C가 돌려주는 int32_t를 Rust 열거형으로 바로 받지 않고 정수 상수로 둡니다.
// 목록에 없는 값이 오면 Rust 열거형으로는 미정의 동작이지만, 정수는 무엇이 와도 안전합니다.
pub const TALLY_OK: i32 = 0;
pub const TALLY_ERR_NULL: i32 = -1;
pub const TALLY_ERR_NAME_TOO_LONG: i32 = -2;

// typedef struct tally_config { int64_t low; uint32_t bucket_width; uint32_t bucket_count; } tally_config;
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct tally_config {
    pub low: i64,
    pub bucket_width: u32,
    pub bucket_count: u32,
}

// typedef struct tally_summary {
//     uint64_t count; uint64_t below; uint64_t above; int64_t min; int64_t max;
// } tally_summary;
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct tally_summary {
    pub count: u64,
    pub below: u64,
    pub above: u64,
    pub min: i64,
    pub max: i64,
}

// typedef struct tally tally;  (불투명 타입)
// Rust에서는 크기가 0인 #[repr(C)] 구조체로 "이름만 있는 타입"을 흉내 냅니다.
// 이 타입의 값은 만들 수 없고 *mut tally / *const tally로만 다룹니다.
// 불투명 타입을 선언하는 더 엄밀한 방법은 Part 10 ch12-ffi-beneath-std에서 다룹니다.
#[repr(C)]
pub struct tally {
    _opaque: [u8; 0],
}

unsafe extern "C" {
    // tally_config tally_default_config(void);
    // 인자가 없고 늘 같은 값을 돌려주므로 safe fn으로 선언합니다.
    pub safe fn tally_default_config() -> tally_config;

    // tally *tally_new(tally_config config);
    // C가 설정을 검사해 잘못되면 NULL을 돌려주므로, 어떤 값을 넘겨도 미정의 동작이 없습니다.
    // 그래서 safe fn입니다. 돌려받은 포인터를 쓰는 일은 여전히 unsafe입니다.
    pub safe fn tally_new(config: tally_config) -> *mut tally;

    // void tally_free(tally *t);
    // 아래는 모두 포인터가 살아 있어야 하므로 unsafe fn입니다(표시가 없어도 unsafe).
    pub unsafe fn tally_free(t: *mut tally);

    // int32_t tally_record(tally *t, int64_t sample);
    pub unsafe fn tally_record(t: *mut tally, sample: i64) -> i32;

    // tally_summary tally_summarize(const tally *t);
    // C의 const 포인터는 *const로, const가 없는 포인터는 *mut로 옮깁니다.
    pub unsafe fn tally_summarize(t: *const tally) -> tally_summary;

    // int32_t tally_set_name(tally *t, const char *name);
    pub unsafe fn tally_set_name(t: *mut tally, name: *const c_char) -> i32;

    // const char *tally_name(const tally *t);
    pub unsafe fn tally_name(t: *const tally) -> *const c_char;

    // char *tally_render(const tally *t);
    pub unsafe fn tally_render(t: *const tally) -> *mut c_char;

    // void tally_string_free(char *s);
    pub unsafe fn tally_string_free(s: *mut c_char);
}
