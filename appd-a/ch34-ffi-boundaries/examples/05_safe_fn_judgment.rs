// 지침 5: unsafe를 어디에 둘지 고릅니다. 어떤 인자로 불러도 안전한 C 함수만 safe fn으로
// 선언하고, 조건이 있는 함수는 unsafe fn으로 선언한 뒤 그 조건을 타입으로 보장하는 래퍼를 둡니다.
//
// 실행: cargo run -p er-ch34-ffi-boundaries --example 05_safe_fn_judgment
//
// C 표준 라이브러리의 hypot(<math.h>)과 toupper(<ctype.h>)를 부릅니다. 둘 다 표준
// 라이브러리가 이미 링크하는 시스템 라이브러리에 있어 따로 링크할 것이 없습니다.
// toupper의 결과는 로캘에 따라 다르며, 아래 출력은 Rust 프로그램의 기본값인 "C" 로캘 기준입니다.

use std::ffi::c_int;

mod libc_decls {
    use std::ffi::c_int;

    unsafe extern "C" {
        // double hypot(double x, double y);
        // 포인터도 전역 상태도 없고, 어떤 f64 쌍(NaN과 무한대 포함)에도 결과가 정의돼 있습니다.
        // 그래서 "어떤 인자로 불러도 미정의 동작이 없다"고 보증할 수 있어 safe fn으로 선언합니다.
        pub safe fn hypot(x: f64, y: f64) -> f64;

        // int toupper(int c);
        // C 표준은 c가 EOF이거나 unsigned char로 나타낼 수 있는 값이어야 한다고 정합니다.
        // 그 밖의 값(예: 1000)은 미정의 동작이므로 safe fn으로 선언하면 안 됩니다.
        pub unsafe fn toupper(c: c_int) -> c_int;
    }
}

mod careless {
    use std::ffi::c_int;

    // 비권장: 조건이 있는 함수를 safe fn으로 선언했습니다. 컴파일러는 C 함수의 조건을 모르므로
    // 그대로 받아들이고, 이제 안전한 코드 어디서나 toupper(1000)을 unsafe 없이 쓸 수 있습니다.
    // 같은 크레이트에 같은 심볼을 다르게 선언한 곳(libc_decls)이 있어서 이번에는 rustc가
    // clashing_extern_declarations로 알려 주었지만, 이 선언 하나만 있었다면 아무 경고도 없습니다.
    #[expect(
        clashing_extern_declarations,
        reason = "같은 크레이트 안의 두 선언이 다르면 rustc가 알려 준다는 것을 보입니다"
    )]
    unsafe extern "C" {
        pub safe fn toupper(c: c_int) -> c_int;
    }
}

/// 권장: 매개변수를 u8로 받으면 toupper의 조건("unsigned char로 나타낼 수 있는 값")을
/// 타입이 보장하므로, 이 래퍼는 unsafe fn일 필요가 없습니다.
fn ascii_upper_via_c(b: u8) -> u8 {
    // SAFETY: c_int::from(b)는 0..=255이고, 모두 unsigned char로 나타낼 수 있는 값입니다.
    let r = unsafe { libc_decls::toupper(c_int::from(b)) };
    // toupper는 unsigned char 범위의 값이나 c를 그대로 돌려줍니다. 혹시 범위를 벗어나면 원래 값을 씁니다.
    u8::try_from(r).unwrap_or(b)
}

fn main() {
    println!("1. safe fn: 어떤 인자에도 정의된 결과");
    for (x, y) in [(3.0, 4.0), (f64::INFINITY, f64::NAN), (f64::NAN, 1.0)] {
        println!("   hypot({x}, {y}) = {}", libc_decls::hypot(x, y));
    }
    // 출력:    hypot(3, 4) = 5
    // 출력:    hypot(inf, NaN) = inf
    // 출력:    hypot(NaN, 1) = NaN

    println!("2. unsafe fn + 타입으로 조건을 보장하는 래퍼");
    let word = b"ffi-guard \xE9";
    let upper: Vec<u8> = word.iter().map(|&b| ascii_upper_via_c(b)).collect();
    println!("   {:?}", String::from_utf8_lossy(word));
    println!("   {:?}", String::from_utf8_lossy(&upper));
    // 출력:    "ffi-guard �"
    // 출력:    "FFI-GUARD �"
    // 0xE9는 "C" 로캘에서 글자로 보지 않으므로 그대로 돌아옵니다.

    println!("3. 비권장: 조건이 있는 함수를 safe fn으로 선언");
    // 올바른 인자로 부르는 동안은 아무 문제가 없어 보입니다. careless::toupper(1000)도
    // unsafe 없이 컴파일되지만, 실행하면 미정의 동작이므로 이 예제는 부르지 않습니다.
    let c = careless::toupper(c_int::from(b'q'));
    println!(
        "   careless::toupper('q') = {:?}",
        char::from_u32(c.cast_unsigned())
    );
    // 출력:    careless::toupper('q') = Some('Q')
}
