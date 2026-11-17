// unsafe의 나머지 능력: C 함수 호출(extern "C"), Rust 함수를 C에 내보내기(#[unsafe(no_mangle)]),
// 정적 변수와 가변 정적 변수(static mut), unsafe 트레이트를 보여 줍니다.
//
// 실행: cargo run -p p0-ch19-advanced-features --example 02_ffi_static_unsafe_trait

use std::ffi::{c_char, c_double, c_int};
use std::sync::Mutex;

// 1. C 표준 라이브러리 함수 선언
//    Edition 2024에서는 extern 블록 앞에 unsafe를 붙여야 합니다.
//    "이 선언(이름, 매개변수 타입)이 실제 C 함수와 맞다"는 것을 컴파일러가 확인할 수 없기 때문입니다.
unsafe extern "C" {
    // <stdlib.h>의 int abs(int). abs(INT_MIN)은 C에서 미정의 동작이라 safe로 선언하지 않았습니다.
    fn abs(input: c_int) -> c_int;

    // <string.h>의 size_t strlen(const char *s). 끝에 0 바이트가 있는 문자열의 포인터를 받아야 합니다.
    fn strlen(s: *const c_char) -> usize;

    // <math.h>의 double sqrt(double). 어떤 입력에도 결과가 정의되어 있으므로
    // safe로 선언했습니다. 이 선언 덕분에 unsafe 블록 없이 부를 수 있습니다.
    safe fn sqrt(x: c_double) -> c_double;
}

// 2. Rust 함수를 C 쪽에 내보내기
//    extern "C"는 C의 호출 규약을 쓰라는 뜻이고, no_mangle은 이름을 바꾸지 말라는 뜻입니다.
//    이름이 바뀌지 않으면 다른 라이브러리의 같은 이름과 충돌할 수 있어 Edition 2024에서는 unsafe(...)로 감쌉니다.
#[unsafe(no_mangle)]
pub extern "C" fn ch19_add(a: c_int, b: c_int) -> c_int {
    a + b
}

// C 프로그램이 ch19_add를 쓸 때 하는 일을 흉내 냅니다: 이름만 알고 선언해서 부릅니다.
// 링커가 위에서 내보낸 ch19_add를 이 이름으로 찾아 연결합니다.
mod c_side {
    use std::ffi::c_int;

    unsafe extern "C" {
        pub fn ch19_add(a: c_int, b: c_int) -> c_int;
    }
}

// 3. 정적 변수
static GREETING: &str = "안녕하세요"; // 불변 정적 변수: 읽기는 안전합니다.
static mut COUNTER: u32 = 0; // 가변 정적 변수: 읽기도 쓰기도 unsafe입니다.

/// `COUNTER`에 `inc`를 더합니다.
///
/// # Safety
///
/// 여러 스레드에서 동시에 부르면 데이터 경쟁이 일어납니다. 한 번에 한 스레드에서만 불러야 합니다.
unsafe fn add_to_count(inc: u32) {
    // SAFETY: 한 스레드에서만 부른다는 것은 부르는 쪽이 보장합니다.
    unsafe {
        COUNTER += inc;
    }
}

// static mut 대신 쓸 수 있는 안전한 방법: 동기화 타입(ch15의 Mutex)을 불변 static에 담습니다.
static LOG: Mutex<Vec<&str>> = Mutex::new(Vec::new());

// 4. unsafe 트레이트: 컴파일러가 확인할 수 없는 약속을 구현하는 쪽이 지켜야 하는 트레이트입니다.
/// 모든 비트가 0인 값이 올바른 값인 타입입니다.
///
/// # Safety
///
/// 구현하는 타입은 메모리를 0으로 채운 값이 그 타입의 올바른 값이어야 합니다.
/// 예를 들어 참조(`&T`)는 0(null)일 수 없으므로 구현하면 안 됩니다.
unsafe trait Zeroable {}

// SAFETY: 정수는 모든 비트가 0이면 0입니다.
unsafe impl Zeroable for u32 {}
// SAFETY: u8 배열은 모든 원소가 0인 배열입니다.
unsafe impl Zeroable for [u8; 4] {}

// 이 함수는 안전한 함수입니다. T: Zeroable이라는 약속에 기대어 안에서 unsafe를 씁니다.
fn zeroed<T: Zeroable>() -> T {
    // SAFETY: T가 Zeroable을 구현했다면 0으로 채운 값이 올바른 값입니다(트레이트의 # Safety).
    unsafe { std::mem::zeroed() }
}

fn main() {
    println!("1. C 함수 호출");
    // SAFETY: abs에는 INT_MIN이 아닌 값을 넘깁니다.
    let a = unsafe { abs(-3) };
    println!("    abs(-3) = {a}"); // 출력:     abs(-3) = 3

    // c"..."는 끝에 0 바이트가 붙은 C 문자열 리터럴입니다. strlen은 0 바이트 앞까지의 바이트 수를 셉니다.
    // SAFETY: C 문자열 리터럴의 포인터는 0 바이트로 끝나는 살아 있는 문자열을 가리킵니다.
    let (len_en, len_ko) = unsafe { (strlen(c"hello".as_ptr()), strlen(c"안녕".as_ptr())) };
    println!("    strlen(\"hello\") = {len_en}, strlen(\"안녕\") = {len_ko}"); // 출력:     strlen("hello") = 5, strlen("안녕") = 6

    // safe로 선언한 함수는 unsafe 블록이 필요 없습니다.
    println!("    sqrt(2.0) = {}", sqrt(2.0)); // 출력:     sqrt(2.0) = 1.4142135623730951

    println!("2. C에 내보낸 함수");
    // Rust 안에서는 보통 함수처럼 부를 수 있습니다.
    println!("    ch19_add(1, 2) = {}", ch19_add(1, 2)); // 출력:     ch19_add(1, 2) = 3
    // 이름만 보고 선언한 쪽(c_side)에서도 찾아 부를 수 있습니다. no_mangle을 지우면 링크 에러가 납니다.
    // SAFETY: c_side의 선언은 위의 ch19_add와 시그니처가 같습니다.
    let sum = unsafe { c_side::ch19_add(2, 3) };
    println!("    c_side::ch19_add(2, 3) = {sum}"); // 출력:     c_side::ch19_add(2, 3) = 5

    println!("3. 정적 변수");
    println!("    GREETING = {GREETING}"); // 출력:     GREETING = 안녕하세요
    // SAFETY: 이 프로그램은 main 스레드 하나에서만 add_to_count를 부릅니다.
    unsafe {
        add_to_count(3);
        add_to_count(4);
    }
    // println!("{}", COUNTER)는 COUNTER의 참조를 몰래 만들어 static_mut_refs 에러(Edition 2024)가 납니다.
    // 값을 복사해 오거나 &raw const로 원시 포인터를 만들어 읽습니다.
    // 원시 포인터를 만드는 것 자체는 안전하므로 unsafe 블록 밖에서 해도 됩니다.
    let ptr = &raw const COUNTER;
    // SAFETY: 지금 COUNTER를 바꾸는 다른 스레드가 없습니다.
    let (copied, through_ptr) = unsafe { (COUNTER, *ptr) };
    println!("    COUNTER = {copied} (복사), {through_ptr} (원시 포인터)"); // 출력:     COUNTER = 7 (복사), 7 (원시 포인터)

    // Mutex를 담은 불변 static은 unsafe 없이 여러 곳에서 고칠 수 있습니다.
    LOG.lock().unwrap().push("첫 번째");
    LOG.lock().unwrap().push("두 번째");
    println!("    LOG = {:?}", LOG.lock().unwrap()); // 출력:     LOG = ["첫 번째", "두 번째"]

    println!("4. unsafe 트레이트");
    let n: u32 = zeroed();
    let bytes: [u8; 4] = zeroed();
    println!("    zeroed::<u32>() = {n}, zeroed::<[u8; 4]>() = {bytes:?}"); // 출력:     zeroed::<u32>() = 0, zeroed::<[u8; 4]>() = [0, 0, 0, 0]
}
