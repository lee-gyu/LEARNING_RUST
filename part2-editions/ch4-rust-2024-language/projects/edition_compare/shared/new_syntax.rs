//! 2024 에디션이 요구하는 문법으로 쓴 코드가 2021 에디션에서도 그대로 컴파일되는지
//! 확인합니다. 두 실행 파일의 출력은 같습니다.
//!
//! 실행(이 프로젝트 디렉터리에서):
//!   cargo run -q --bin new_syntax_2021
//!   cargo run -q --bin new_syntax_2024
//!
//! 이 파일의 문법은 모두 2021 에디션에서도 쓸 수 있습니다. 그래서 2021 에디션 코드를
//! 먼저 이 모양으로 고쳐 두면, 에디션을 올릴 때 바꿀 것이 줄어듭니다.

use std::ffi::{c_char, c_double, c_int};

// 1. `unsafe extern` 블록과 항목별 `safe`/`unsafe` 표시
//    2021 에디션에서는 블록 앞의 `unsafe`를 생략할 수 있지만, 붙여도 됩니다.
//    `safe`/`unsafe` 표시는 블록에 `unsafe`가 붙어 있을 때만 쓸 수 있습니다.
unsafe extern "C" {
    /// C 표준 라이브러리의 `sqrt`. 어떤 `c_double`을 넘겨도 결과가 정의되어 있으므로 `safe`입니다.
    safe fn sqrt(x: c_double) -> c_double;
    /// C 표준 라이브러리의 `strlen`. 0 바이트로 끝나는 올바른 포인터가 필요하므로 `unsafe`입니다.
    unsafe fn strlen(s: *const c_char) -> usize;
    /// C 표준 라이브러리의 `abs`. 표시가 없으면 `unsafe fn`으로 취급합니다.
    /// (`abs(INT_MIN)`은 C에서 미정의 동작이라 `safe`로 선언하지 않았습니다.)
    fn abs(x: c_int) -> c_int;
}

// 2. unsafe 속성: `#[unsafe(no_mangle)]`
//    2021 에디션에서는 `#[no_mangle]`로도 되지만, 이 모양도 받아들입니다.
// SAFETY: 이 프로그램에 `p2ch4_double`이라는 이름의 다른 심볼은 없습니다.
#[unsafe(no_mangle)]
pub extern "C" fn p2ch4_double(x: c_int) -> c_int {
    x * 2
}

// 3. unsafe 함수 안에서도 unsafe 연산은 `unsafe { }`로 감쌉니다.
/// `ptr`이 가리키는 `i32`를 읽습니다.
///
/// # Safety
///
/// `ptr`은 살아 있는 `i32`를 가리키는 올바른 포인터여야 합니다.
unsafe fn read_i32(ptr: *const i32) -> i32 {
    // SAFETY: 부르는 쪽이 `ptr`이 올바르다고 보장합니다(함수의 Safety 절).
    unsafe { *ptr }
}

// 4. `static mut`는 참조 대신 `&raw const`/`&raw mut`로 다룹니다.
static mut TICKS: u32 = 0;

// 5. `gen`이라는 이름은 raw 식별자 `r#gen`으로 씁니다.
fn r#gen() -> u32 {
    4
}

// 6. 정밀 캡처 `use<..>`: 반환 타입이 어떤 매개변수를 쓰는지 직접 적습니다.
//    `use<>`는 "인자의 라이프타임을 캡처하지 않는다"는 뜻입니다.
fn index_range(items: &[i32]) -> impl Iterator<Item = usize> + use<> {
    0..items.len()
}

fn main() {
    println!("1. unsafe extern: sqrt(2.25) = {}", sqrt(2.25));
    // SAFETY: c"..." 리터럴은 0 바이트로 끝나는 올바른 C 문자열이고, strlen은 읽기만 합니다.
    let len = unsafe { strlen(c"edition".as_ptr()) };
    // SAFETY: -3은 INT_MIN이 아니므로 abs의 결과가 정의되어 있습니다.
    let absolute = unsafe { abs(-3) };
    println!("   strlen(c\"edition\") = {len}, abs(-3) = {absolute}");

    println!("2. unsafe 속성: p2ch4_double(21) = {}", p2ch4_double(21));

    let values = [10, 20];
    // SAFETY: `values.as_ptr()`은 살아 있는 배열의 첫 원소를 가리킵니다.
    let first = unsafe { read_i32(values.as_ptr()) };
    println!("3. unsafe fn 안의 unsafe 블록: {first}");

    // 원시 포인터를 만드는 것까지는 안전한 코드입니다. 참조(&mut TICKS)는 만들지 않습니다.
    let ticks = &raw mut TICKS;
    // SAFETY: 이 프로그램은 스레드를 만들지 않고, 이 블록 밖에서 TICKS를 가리키는
    // 참조나 포인터를 쓰지 않으므로 동시에 접근하는 곳이 없습니다.
    let after = unsafe {
        *ticks += 1;
        *ticks
    };
    println!("4. &raw mut으로 static mut 고치기: {after}");

    println!("5. r#gen() = {}", r#gen());

    let mut items = vec![1, 2, 3];
    let range = index_range(&items);
    items.push(4); // use<> 덕분에 range가 items를 빌리고 있지 않습니다.
    println!(
        "6. use<>: {:?}, items = {items:?}",
        range.collect::<Vec<_>>()
    );

    // 7. 참조를 맞추는 패턴에서 `mut`/`ref`가 필요하면 바깥 `&`부터 직접 적습니다.
    let pair = &(String::from("러스트"), 90);
    let &(ref name, mut score) = pair;
    score += 5;
    println!("7. 완전히 명시한 패턴: {name} {score}");
}
