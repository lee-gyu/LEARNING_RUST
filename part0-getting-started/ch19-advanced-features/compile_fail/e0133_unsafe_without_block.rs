// 기대하는 에러: E0133 — dereference of raw pointer is unsafe and requires unsafe block
//                         call to unsafe function `read_first` is unsafe and requires unsafe block
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0133_unsafe_without_block.rs
//
// 원시 포인터를 만드는 것은 안전한 코드에서도 되지만, 따라가서(역참조) 값을 읽는 것과
// unsafe 함수를 부르는 것은 unsafe 블록 안에서만 할 수 있습니다.
// 해결: 그 부분을 `unsafe { ... }`로 감싸고, 왜 안전한지 `// SAFETY:` 주석으로 적습니다.

/// # Safety
///
/// `ptr`은 살아 있는 `i32`를 가리켜야 합니다.
unsafe fn read_first(ptr: *const i32) -> i32 {
    unsafe { *ptr }
}

fn main() {
    let num = 5;
    let r = &raw const num; // 여기까지는 괜찮습니다.

    let value = *r; // 원시 포인터 역참조
    let again = read_first(r); // unsafe 함수 호출

    println!("{value} {again}");
}
