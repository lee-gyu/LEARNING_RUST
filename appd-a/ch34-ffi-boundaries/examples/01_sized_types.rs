// 지침 1: FFI 선언에는 크기가 정해진 정수 타입을 쓰고, C에 없는 타입은 경계에 두지 않습니다.
//
// 실행: cargo run -p er-ch34-ffi-boundaries --example 01_sized_types
//
// C 타입 별칭(c_int, c_long 등)의 크기와 부호는 플랫폼이 정합니다.
// 아래 출력 주석은 macOS aarch64(64비트) 기준이고, 다른 플랫폼에서는 1번 구역이 달라질 수 있습니다.

use std::any::type_name;
use std::ffi::{c_char, c_int, c_long, c_longlong, c_short};
use std::mem::size_of;

fn alias<T>(c_name: &str) {
    println!(
        "   {c_name:<10} → {:<4} {}바이트",
        type_name::<T>(),
        size_of::<T>()
    );
}

// 3번 구역: C에 대응하는 타입이 없는 Rust 타입을 선언에 쓰면 improper_ctypes 경고가 납니다.
// 이 선언들은 부르지 않으므로(심볼을 요청하지 않으므로) 링크에는 영향이 없습니다.
// #[expect]를 지우면 rustc의 경고 6개와 dead_code 경고가 보입니다(README 2.2절).
#[expect(
    improper_ctypes,
    dead_code,
    reason = "C에 없는 타입을 선언에 쓰면 rustc가 경고한다는 것을 보이려는 선언입니다"
)]
mod rejected {
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    unsafe extern "C" {
        pub fn takes_str(s: &str); // str: 길이를 함께 가진 넓은 포인터
        pub fn takes_string(s: String); // String: 배치가 정해지지 않은 구조체
        pub fn takes_tuple(t: (i32, i32)); // 튜플: 배치가 정해지지 않음
        pub fn takes_point(p: Point); // #[repr(C)]가 없는 구조체
        pub fn takes_char(c: char); // Rust의 char는 C의 char와 다름
        pub fn takes_slice(s: &[u8]); // 슬라이스: 넓은 포인터
    }
}

fn main() {
    println!("1. C 타입 별칭이 이 환경에서 가리키는 Rust 타입");
    alias::<c_char>("c_char");
    alias::<c_short>("c_short");
    alias::<c_int>("c_int");
    alias::<c_long>("c_long");
    alias::<c_longlong>("c_longlong");
    alias::<usize>("size_t");
    // 출력:    c_char     → i8   1바이트
    // 출력:    c_short    → i16  2바이트
    // 출력:    c_int      → i32  4바이트
    // 출력:    c_long     → i64  8바이트
    // 출력:    c_longlong → i64  8바이트
    // 출력:    size_t     → usize 8바이트
    // c_long은 별칭일 뿐이라 이 환경에서는 i64와 같은 타입입니다. 문서에 따르면 Windows에서는
    // i32입니다. c_long으로 적은 선언과 그 값을 i64로 다루는 코드는 여기서만 우연히 맞습니다.

    println!("2. 5 GiB 파일 크기를 경계 너머로 돌려받는다면");
    let five_gib: i64 = 5 * 1024 * 1024 * 1024;
    // 비권장: C 헤더가 `long file_size(...)`이고 Rust 선언이 c_long이라면, long이 32비트인
    // 플랫폼에서는 이 값을 담을 수 없습니다. 그 플랫폼의 c_long(= i32)으로 바꿔 흉내 냅니다.
    let as_32bit_long = i32::try_from(five_gib);
    println!("   long이 32비트인 플랫폼: {as_32bit_long:?}");
    // 출력:    long이 32비트인 플랫폼: Err(TryFromIntError(PosOverflow))
    // 권장: 헤더가 `int64_t file_size(...)`이면 Rust 선언도 i64이고 어느 플랫폼에서나 같습니다.
    println!("   int64_t / i64: {five_gib}");
    // 출력:    int64_t / i64: 5368709120

    println!("3. C에 없는 타입 대신 쓰는 것 (rejected 모듈의 선언은 컴파일러가 경고함)");
    let pairs = [
        (
            "&str",
            "*const c_char(0 바이트로 끝남) 또는 *const u8 + 길이",
        ),
        ("String", "*mut c_char + 짝지은 해제 함수, 또는 호출자 버퍼"),
        ("(i32, i32)", "#[repr(C)] 구조체"),
        ("Point", "#[repr(C)] Point"),
        ("char", "u32"),
        ("&[u8]", "*const u8 + 길이"),
    ];
    for (rust_type, instead) in pairs {
        println!("   {rust_type:<10} → {instead}");
    }
    // 출력:    &str       → *const c_char(0 바이트로 끝남) 또는 *const u8 + 길이
    // 출력:    String     → *mut c_char + 짝지은 해제 함수, 또는 호출자 버퍼
    // 출력:    (i32, i32) → #[repr(C)] 구조체
    // 출력:    Point      → #[repr(C)] Point
    // 출력:    char       → u32
    // 출력:    &[u8]      → *const u8 + 길이
}
