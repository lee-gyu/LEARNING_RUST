// 부록 B ch17 · Q17.1·Q17.2·Q17.6 — 같은 일을 하는 C 함수와 Rust 함수도 동작이 다르다
//
// C 표준 라이브러리의 strtol(문자열을 long으로)을 Rust에서 직접 부르고, 표준 라이브러리의 str::parse::<i64>와
// 같은 입력에서 결과를 비교합니다. strtol은 앞의 공백을 건너뛰고, 숫자가 아닌 글자를 만나면 거기까지만 읽고,
// 범위를 넘으면 값을 최댓값이나 최솟값으로 잘라(clamp) 돌려줍니다. parse는 문자열 전체가 숫자여야 하고 범위를 넘으면
// 에러입니다.
// C 코드를 Rust로 옮길 때 "같은 일을 하는 함수"로 바꾸면 이런 차이가 그대로 동작의 차이가 됩니다.
//
// 2번 구역은 strtol을 안전한 Rust로 다시 쓴 함수 두 판을 C의 결과와 입력마다 대조합니다(차등 비교).
// 첫 판은 공백을 u8::is_ascii_whitespace로 판단하는데, 이 함수는 C의 isspace와 달리 세로 탭(\v, U+000B)을
// 공백으로 보지 않습니다(표준 라이브러리 문서의 경고). 대조가 그 차이를 찾아냅니다.
//
// c_long은 이 환경(macOS aarch64)에서 i64입니다. Windows처럼 c_long이 32비트인 플랫폼에서는 범위를 넘는 입력의 결과가
// 달라집니다.
//
// 실행: cargo run -p faq-ch17-interop-migration --example 01_strtol_vs_parse

use std::ffi::{CStr, CString, c_char, c_int, c_long};

unsafe extern "C" {
    // C 표준 <stdlib.h>: long strtol(const char *str, char **endptr, int base);
    // 원시 포인터를 받아 그 너머를 읽고 쓰므로 safe fn으로 선언할 수 없습니다.
    fn strtol(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_long;
}

/// strtol을 10진수로 부르고 (값, 읽은 바이트 수)를 돌려주는 안전한 래퍼
fn c_strtol(s: &CStr) -> (i64, usize) {
    let mut end: *mut c_char = std::ptr::null_mut();
    // SAFETY: s.as_ptr()는 NUL로 끝나는 유효한 문자열이고, end는 strtol이 결과 위치를 써 넣을 지역 변수입니다.
    let value = unsafe { strtol(s.as_ptr(), &mut end, 10) };
    // SAFETY: strtol은 end에 s의 시작이나 s 안의 위치를 써 넣으므로, 두 포인터는 같은 할당 안에 있습니다.
    let used = unsafe { end.offset_from(s.as_ptr()) };
    // 이 린트는 c_long이 i64인 플랫폼에서만 나므로 #[expect] 대신 #[allow]를 씁니다.
    #[allow(
        clippy::useless_conversion,
        reason = "c_long은 이 환경에서 i64지만, 32비트인 플랫폼도 있어 i64로 넓히는 변환을 둡니다"
    )]
    let value = i64::from(value);
    (value, used as usize)
}

/// 공백인지 판단하는 함수
type IsSpace = fn(u8) -> bool;

/// strtol(10진수)을 안전한 Rust로 옮긴 함수. is_space가 공백 판단을 맡습니다.
fn strtol_like(s: &[u8], is_space: IsSpace) -> (i64, usize) {
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let negative = i < s.len() && s[i] == b'-';
    if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
        i += 1;
    }
    let digits_start = i;
    let mut value: i64 = 0;
    let mut overflow = false;
    while i < s.len() && s[i].is_ascii_digit() {
        let d = i64::from(s[i] - b'0');
        // 음수 쪽이 하나 더 넓으므로(i64::MIN) 음수로 쌓아 가며 오버플로를 검사합니다.
        match value.checked_mul(10).and_then(|v| v.checked_sub(d)) {
            Some(v) => value = v,
            None => overflow = true,
        }
        i += 1;
    }
    if i == digits_start {
        // 숫자가 하나도 없으면 strtol은 0을 돌려주고 endptr을 문자열의 시작에 둡니다.
        return (0, 0);
    }
    let value = match (overflow, negative) {
        (true, true) => i64::MIN,
        (true, false) => i64::MAX,
        (false, true) => value,
        (false, false) => value.checked_neg().unwrap_or(i64::MAX),
    };
    (value, i)
}

/// Rust 표준 라이브러리의 ASCII 공백: 공백, \t, \n, \f, \r (\v는 빠짐)
fn rust_ascii_space(b: u8) -> bool {
    b.is_ascii_whitespace()
}

/// C 로캘의 isspace: 공백, \t, \n, \v, \f, \r
fn c_isspace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

fn main() {
    let inputs = [
        "42",
        " 42",
        "+42",
        "42abc",
        "abc",
        "",
        "9223372036854775808",
        "-9223372036854775809",
        "\u{b}42",
    ];

    println!("1. strtol과 parse::<i64>"); // 출력: 1. strtol과 parse::<i64>
    for text in inputs {
        let c_text = CString::new(text).expect("입력에 NUL이 없습니다");
        let (value, used) = c_strtol(&c_text);
        let parsed = text.parse::<i64>().map_err(|e| *e.kind());
        println!(
            "  {:<24} strtol: {value} ({used}바이트 읽음) / parse: {parsed:?}",
            format!("{text:?}")
        );
    }
    // 출력:   "42"                     strtol: 42 (2바이트 읽음) / parse: Ok(42)
    // 출력:   " 42"                    strtol: 42 (3바이트 읽음) / parse: Err(InvalidDigit)
    // 출력:   "+42"                    strtol: 42 (3바이트 읽음) / parse: Ok(42)
    // 출력:   "42abc"                  strtol: 42 (2바이트 읽음) / parse: Err(InvalidDigit)
    // 출력:   "abc"                    strtol: 0 (0바이트 읽음) / parse: Err(InvalidDigit)
    // 출력:   ""                       strtol: 0 (0바이트 읽음) / parse: Err(Empty)
    // 출력:   "9223372036854775808"    strtol: 9223372036854775807 (19바이트 읽음) / parse: Err(PosOverflow)
    // 출력:   "-9223372036854775809"   strtol: -9223372036854775808 (20바이트 읽음) / parse: Err(NegOverflow)
    // 출력:   "\u{b}42"                strtol: 42 (3바이트 읽음) / parse: Err(InvalidDigit)

    println!("2. strtol을 옮긴 Rust 함수를 C의 결과와 대조"); // 출력: 2. strtol을 옮긴 Rust 함수를 C의 결과와 대조
    let versions: [(&str, IsSpace); 2] = [
        ("첫 판(is_ascii_whitespace)", rust_ascii_space),
        ("고친 판(C의 isspace 목록)", c_isspace),
    ];
    for (name, is_space) in versions {
        let mut differ = Vec::new();
        for text in inputs {
            let c_text = CString::new(text).expect("입력에 NUL이 없습니다");
            if c_strtol(&c_text) != strtol_like(text.as_bytes(), is_space) {
                differ.push(format!("{text:?}"));
            }
        }
        let differ_text = if differ.is_empty() {
            "없음".to_string()
        } else {
            differ.join(", ")
        };
        println!(
            "  {name}: {}개 중 {}개 같음, 다른 입력: {differ_text}",
            inputs.len(),
            inputs.len() - differ.len()
        );
    }
    // 출력:   첫 판(is_ascii_whitespace): 9개 중 8개 같음, 다른 입력: "\u{b}42"
    // 출력:   고친 판(C의 isspace 목록): 9개 중 9개 같음, 다른 입력: 없음
}
