// 2024 에디션에서 unsafe가 드러나야 하는 곳 네 가지를 보여 줍니다.
// `unsafe extern` 블록, unsafe 속성, unsafe 함수 안의 unsafe 블록, `static mut` 참조 금지.
//
// 실행: cargo run -p p2-ch4-rust-2024-language --example 05_unsafe_changes
//
// 이 파일의 문법은 2021 에디션에서도 모두 컴파일됩니다
// (projects/edition_compare의 new_syntax_2021로 확인할 수 있습니다).

use std::ffi::{c_char, c_double, c_int};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

// 1. unsafe extern 블록
//    블록 앞의 `unsafe`는 "아래 선언이 실제 C 함수와 맞다는 것은 내가 보증한다"는 표시입니다.
//    항목마다 `safe`(부를 때 unsafe 블록이 필요 없음)나 `unsafe`를 붙일 수 있고,
//    아무것도 붙이지 않으면 `unsafe fn`으로 취급합니다.
unsafe extern "C" {
    /// <math.h>의 `sqrt`. 어떤 값을 넘겨도 결과가 정의되어 있으므로 `safe`입니다.
    safe fn sqrt(x: c_double) -> c_double;
    /// <string.h>의 `strlen`. 0 바이트로 끝나는 올바른 포인터가 필요합니다.
    unsafe fn strlen(s: *const c_char) -> usize;
    /// <stdlib.h>의 `abs`. 표시가 없으므로 `unsafe fn`입니다(`abs(INT_MIN)`은 C에서 미정의 동작).
    fn abs(x: c_int) -> c_int;
}

// 2. unsafe 속성
//    `no_mangle`과 `export_name`은 링커가 보는 심볼 이름을 정합니다. 링크되는 모든 라이브러리가
//    이름 공간 하나를 함께 쓰므로, 이름이 겹치면 안전한 코드만으로도 프로그램이 망가질 수 있습니다.
// SAFETY: 이 프로그램에 `p2ch4_triple`이라는 이름의 다른 심볼은 없습니다.
#[unsafe(no_mangle)]
pub extern "C" fn p2ch4_triple(x: c_int) -> c_int {
    x * 3
}

// SAFETY: 이 프로그램에 `p2ch4_renamed`라는 이름의 다른 심볼은 없습니다.
#[unsafe(export_name = "p2ch4_renamed")]
pub extern "C" fn answer() -> c_int {
    42
}

/// C 프로그램이 위 두 함수를 쓸 때처럼, 심볼 이름만 알고 선언해서 부릅니다.
mod c_side {
    use std::ffi::c_int;

    unsafe extern "C" {
        pub safe fn p2ch4_triple(x: c_int) -> c_int;
        pub safe fn p2ch4_renamed() -> c_int;
    }
}

// 3. unsafe 함수의 본문도 unsafe 연산은 `unsafe { }`로 감쌉니다.
/// `ptr`이 가리키는 `i32`를 읽습니다.
///
/// # Safety
///
/// `ptr`은 살아 있는 `i32`를 가리키는 올바른 포인터여야 합니다.
unsafe fn read_i32(ptr: *const i32) -> i32 {
    // SAFETY: 부르는 쪽이 `ptr`이 올바르다고 보장합니다(함수의 Safety 절).
    unsafe { *ptr }
}

/// 감싸지 않은 옛 모양입니다. 2024 에디션에서는 `unsafe_op_in_unsafe_fn` 경고가 납니다.
/// 경고가 난다는 것을 확인하려고 `#[expect]`를 붙였습니다(경고가 나지 않으면 오히려 경고).
///
/// # Safety
///
/// `ptr`은 살아 있는 `i32`를 가리키는 올바른 포인터여야 합니다.
#[expect(
    unsafe_op_in_unsafe_fn,
    reason = "2024 에디션에서 이 린트가 기본으로 경고한다는 것을 보이려는 옛 모양입니다"
)]
unsafe fn read_i32_old_style(ptr: *const i32) -> i32 {
    *ptr
}

// 4. static mut 대신 쓰는 방법들
static mut RAW_COUNTER: u32 = 0; // 원시 포인터로만 다룹니다.
static ATOMIC_COUNTER: AtomicU32 = AtomicU32::new(0); // `mut`이 없는 static
static EVENTS: Mutex<Vec<&str>> = Mutex::new(Vec::new()); // `mut`이 없는 static

fn main() {
    println!("1. unsafe extern 블록");
    println!(
        "    sqrt(2.25) = {} (safe fn이라 unsafe 블록 없이 부름)",
        sqrt(2.25)
    );
    // SAFETY: c"..." 리터럴은 0 바이트로 끝나는 올바른 C 문자열이고, strlen은 읽기만 합니다.
    let len = unsafe { strlen(c"Rust 2024".as_ptr()) };
    // SAFETY: -7은 INT_MIN이 아니므로 abs의 결과가 정의되어 있습니다.
    let absolute = unsafe { abs(-7) };
    println!("    strlen(c\"Rust 2024\") = {len}, abs(-7) = {absolute}");
    // 출력:     sqrt(2.25) = 1.5 (safe fn이라 unsafe 블록 없이 부름)
    // 출력:     strlen(c"Rust 2024") = 9, abs(-7) = 7

    println!("2. unsafe 속성으로 정한 심볼 이름으로 부르기");
    println!(
        "    p2ch4_triple(5) = {}, p2ch4_renamed() = {}",
        c_side::p2ch4_triple(5),
        c_side::p2ch4_renamed()
    );
    // 출력:     p2ch4_triple(5) = 15, p2ch4_renamed() = 42

    println!("3. unsafe 함수 안의 unsafe 블록");
    let values = [10, 20, 30];
    // SAFETY: 두 포인터 모두 살아 있는 배열 values의 원소(0번, 2번)를 가리킵니다.
    let (first, last) = unsafe { (read_i32(values.as_ptr()), read_i32_old_style(&values[2])) };
    println!("    first = {first}, last = {last}");
    // 출력:     first = 10, last = 30

    println!("4. static mut 대신 쓰는 방법들");
    // (가) &raw mut: 참조를 만들지 않고 원시 포인터로만 다룹니다. 포인터를 만드는 것은 안전합니다.
    let counter = &raw mut RAW_COUNTER;
    // SAFETY: 이 프로그램은 스레드를 만들지 않고, RAW_COUNTER를 가리키는 다른 참조나
    // 포인터를 동시에 쓰지 않습니다.
    let raw_value = unsafe {
        *counter += 1;
        *counter += 1;
        *counter
    };
    // (나) 원자 타입: 정수 하나라면 unsafe 없이 고칠 수 있습니다.
    ATOMIC_COUNTER.fetch_add(2, Ordering::Relaxed);
    // (다) Mutex: 더 복잡한 값은 잠금으로 감쌉니다.
    EVENTS.lock().unwrap().push("시작");
    EVENTS.lock().unwrap().push("끝");
    println!(
        "    &raw mut: {raw_value}, AtomicU32: {}, Mutex: {:?}",
        ATOMIC_COUNTER.load(Ordering::Relaxed),
        EVENTS.lock().unwrap()
    );
    // 출력:     &raw mut: 2, AtomicU32: 2, Mutex: ["시작", "끝"]
}
