// C 함수에 C 문자열을 넘기고, C 함수가 돌려준 포인터를 CStr로 읽습니다.
// C 표준 라이브러리의 strlen과 getenv만 쓰고, 둘 다 안전한 Rust 함수로 감쌉니다.
//
// 실행: cargo run -p p1-ch14-ffi-any-misc --example 04_calling_c
//
// 1번 구역의 C 타입 별칭은 플랫폼마다 다릅니다(아래 출력은 macOS aarch64 기준).
// 4번 구역의 CARGO_PKG_NAME은 cargo run이 넣어 주는 환경 변수라서,
// 실행 파일을 직접 실행하면 None이 나옵니다.

use std::any::type_name;
use std::env;
use std::ffi::{CStr, CString, c_char, c_int, c_long};

// Edition 2024에서는 extern 블록에 unsafe를 붙입니다(Part 0 ch19).
// 아래 선언이 실제 C 함수와 맞는지는 컴파일러가 확인할 수 없기 때문입니다.
unsafe extern "C" {
    // <string.h> size_t strlen(const char *s);
    fn strlen(s: *const c_char) -> usize;
    // <stdlib.h> char *getenv(const char *name); 없으면 NULL을 돌려줍니다.
    fn getenv(name: *const c_char) -> *mut c_char;
}

/// C의 strlen을 안전하게 부릅니다.
/// 매개변수를 &CStr로 받으면 "0 바이트로 끝나는 살아 있는 버퍼"라는 strlen의 조건을
/// 타입이 보장하므로, 이 함수는 unsafe fn일 필요가 없습니다.
fn c_strlen(s: &CStr) -> usize {
    // SAFETY: s.as_ptr()는 0 바이트로 끝나는 버퍼를 가리키고, 그 버퍼는 s를 빌린 동안
    // (이 호출이 끝날 때까지) 살아 있습니다. strlen은 읽기만 하고 포인터를 보관하지 않습니다.
    unsafe { strlen(s.as_ptr()) }
}

/// C의 getenv를 안전하게 부르고, 결과를 소유한 String으로 복사해 돌려줍니다.
fn c_getenv(name: &CStr) -> Option<String> {
    // SAFETY: name은 0 바이트로 끝나는 살아 있는 C 문자열이고, getenv는 읽기만 합니다.
    let ptr = unsafe { getenv(name.as_ptr()) };
    if ptr.is_null() {
        return None; // 환경 변수가 없으면 NULL입니다. NULL을 CStr::from_ptr에 넘기면 미정의 동작입니다.
    }
    // SAFETY: null이 아닌 getenv의 반환값은 0 바이트로 끝나는 환경 변수 값을 가리킵니다.
    // 이 프로그램은 스레드가 하나이고 환경을 바꾸지 않으므로 아래에서 복사하는 동안 그 버퍼가 유지됩니다.
    let value: &CStr = unsafe { CStr::from_ptr(ptr) };
    // 그 버퍼는 환경이 바뀌면(setenv 등) 사라질 수 있습니다. 빌린 채로 돌려주지 않고 바로 복사합니다.
    Some(value.to_string_lossy().into_owned())
}

fn main() {
    println!("1. C 타입 별칭 (이 환경 기준)");
    // c_char는 C의 char입니다. 부호가 있는지는 C 표준이 정하지 않고 플랫폼에 맡기므로,
    // 문서도 "플랫폼에 따라 i8 또는 u8"이라고만 말합니다. 이 환경(macOS aarch64)에서는 i8입니다.
    println!(
        "    c_char = {} (최솟값 {}), c_int = {}, c_long = {}",
        type_name::<c_char>(),
        c_char::MIN,
        type_name::<c_int>(),
        type_name::<c_long>()
    );
    // 출력:     c_char = i8 (최솟값 -128), c_int = i32, c_long = i64

    println!("2. &CStr을 받는 안전한 래퍼");
    let owned = CString::new(String::from("소유한 문자열")).unwrap();
    // &CString은 Deref로 &CStr이 되므로 리터럴과 CString을 같은 함수에 넘길 수 있습니다.
    println!("    c_strlen(c\"hello\") = {}", c_strlen(c"hello")); // 출력:     c_strlen(c"hello") = 5
    println!(
        "    c_strlen(&owned) = {}, owned.count_bytes() = {}",
        c_strlen(&owned),
        owned.count_bytes()
    );
    // 출력:     c_strlen(&owned) = 19, owned.count_bytes() = 19

    println!("3. as_ptr()의 포인터는 CString이 살아 있는 동안만 유효합니다");
    let word = String::from("러스트");
    // (가) CString을 변수에 묶어 두면 그 변수가 살아 있는 동안 포인터를 써도 됩니다.
    let c_word = CString::new(word.clone()).unwrap();
    let ptr = c_word.as_ptr();
    // SAFETY: ptr이 가리키는 c_word가 아직 살아 있고 0 바이트로 끝납니다.
    let len_a = unsafe { strlen(ptr) };
    // (나) 한 표현식 안에서 만들고 바로 넘기는 것도 괜찮습니다. 임시값은 strlen이 끝난 뒤에 버려집니다.
    // SAFETY: 임시 CString은 이 호출이 끝날 때까지 살아 있고 0 바이트로 끝납니다.
    let len_b = unsafe { strlen(CString::new(word).unwrap().as_ptr()) };
    println!("    (가) 변수에 묶음: {len_a}, (나) 한 표현식 안: {len_b}");
    // 출력:     (가) 변수에 묶음: 9, (나) 한 표현식 안: 9
    // 틀린 모양: let p = CString::new(..).unwrap().as_ptr();
    // 이 문장이 끝나면 임시 CString이 버려져 p는 해제된 메모리를 가리킵니다.
    // 컴파일러가 dangling_pointers_from_temporaries 경고를 냅니다(README 4.6절).

    println!("4. C 함수가 돌려준 포인터 읽기: getenv");
    let from_c = c_getenv(c"CARGO_PKG_NAME");
    let from_std = env::var_os("CARGO_PKG_NAME").map(|v| v.to_string_lossy().into_owned());
    println!("    getenv(\"CARGO_PKG_NAME\") = {from_c:?}");
    println!("    env::var_os와 같은가: {}", from_c == from_std);
    println!(
        "    getenv(\"CH14_SURELY_UNSET\") = {:?}",
        c_getenv(c"CH14_SURELY_UNSET")
    );
    // 출력:     getenv("CARGO_PKG_NAME") = Some("p1-ch14-ffi-any-misc")
    // 출력:     env::var_os와 같은가: true
    // 출력:     getenv("CH14_SURELY_UNSET") = None

    println!("5. 직접 가진 버퍼에 CStr::from_ptr 쓰기");
    // C 쪽에서 두 문자열을 0 바이트로 구분해 이어 붙인 버퍼를 받았다고 합시다.
    let packed: [u8; 8] = *b"abc\0xyz\0";
    // SAFETY: packed는 이 함수 끝까지 살아 있고, 첫 0 바이트(3번 위치)가 버퍼 안에 있습니다.
    let first = unsafe { CStr::from_ptr(packed.as_ptr().cast::<c_char>()) };
    println!("    from_ptr: {first:?}"); // 출력:     from_ptr: "abc"
    // 슬라이스를 가지고 있다면 unsafe 없이 같은 일을 하는 from_bytes_until_nul이 낫습니다.
    // 버퍼 안에 0 바이트가 없으면 끝을 넘어 읽는 대신 Err를 돌려줍니다.
    println!(
        "    from_bytes_until_nul: {:?}",
        CStr::from_bytes_until_nul(&packed)
    );
    // 출력:     from_bytes_until_nul: Ok("abc")
}
