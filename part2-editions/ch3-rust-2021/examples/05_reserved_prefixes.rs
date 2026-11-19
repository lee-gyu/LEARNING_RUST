// 2021 에디션은 식별자 바로 뒤에 ", ', #이 붙은 모양을 "접두사"로 예약했습니다.
// 토큰 수를 세는 매크로로 소스가 어떻게 토큰으로 나뉘는지 보고,
// 예약 덕분에 에디션 경계 없이 추가된 C 문자열 리터럴과 raw 라이프타임을 사용합니다.
//
// 실행: cargo run -p p2-ch3-rust-2021 --example 05_reserved_prefixes

use std::ffi::CStr;

/// 받은 토큰 트리(tt)의 개수를 셉니다. 매크로 입력이 몇 개의 토큰으로 나뉘는지 보여 줍니다.
macro_rules! count_tt {
    () => { 0usize };
    ($first:tt $($rest:tt)*) => { 1usize + count_tt!($($rest)*) };
}

/// 입력을 그대로 문자열로 보여 주고 토큰 수를 함께 출력합니다.
macro_rules! show {
    ($($t:tt)*) => {
        println!("    {:<10} → 토큰 {}개", stringify!($($t)*), count_tt!($($t)*))
    };
}

// 'gen은 2024 에디션에서 키워드가 되었으므로 라이프타임 이름으로 쓸 수 없습니다.
// raw 라이프타임 'r#gen은 키워드와 겹치는 이름도 라이프타임으로 쓰게 해 줍니다.
fn longer<'r#gen>(a: &'r#gen str, b: &'r#gen str) -> &'r#gen str {
    if a.len() >= b.len() { a } else { b }
}

// 'r#a는 'a와 같은 라이프타임입니다. 선언은 'a로, 사용은 'r#a로 해도 됩니다.
// (참조 인자가 둘이라 반환값의 라이프타임을 생략할 수 없는 시그니처입니다.)
fn head<'a>(text: &'r#a str, sep: &str) -> &'a str {
    text.split(sep).next().unwrap_or("")
}

fn main() {
    println!("1. 접두사가 붙은 모양은 토큰 하나입니다");
    show!(c"hi");
    // 출력:     c"hi"      → 토큰 1개
    show!(c "hi");
    // 출력:     c "hi"     → 토큰 2개
    show!(b"hi");
    // 출력:     b"hi"      → 토큰 1개
    show!(r#"raw"#);
    // 출력:     r#"raw"#   → 토큰 1개
    show!('r#a);
    // 출력:     'r#a       → 토큰 1개
    show!('r #a);
    // 출력:     'r #a      → 토큰 3개
    show!(z "hey");
    // 출력:     z "hey"    → 토큰 2개
    // 2018에서는: c"hi"는 2개(c와 "hi"), 'r#a는 3개('r, #, a)입니다. b"hi"와 r#"raw"#는
    // 2015부터 있던 리터럴이라 어느 에디션에서나 1개입니다. 띄어 쓴 줄은 에디션과 상관없이 같습니다.
    // z"hey"처럼 붙여 쓰면 2021부터는 "prefix `z` is unknown" 컴파일 에러입니다.

    println!("2. C 문자열 리터럴 c\"...\"와 cr\"...\"");
    let hello: &CStr = c"안녕";
    // 끝에 0 바이트가 붙은 &'static CStr입니다. 한글은 UTF-8로 3바이트씩입니다.
    println!("    {:?} → {:?}", hello, hello.to_bytes_with_nul());
    // 출력:     "안녕" → [236, 149, 136, 235, 133, 149, 0]
    // \xff 같은 바이트 이스케이프도 됩니다(&str 리터럴에서는 안 되는 것).
    let latin1: &CStr = c"caf\xe9";
    println!("    {:?} → {:?}", latin1, latin1.to_bytes());
    // 출력:     "caf\xe9" → [99, 97, 102, 233]
    // cr"..."은 역슬래시를 이스케이프로 처리하지 않는 raw C 문자열입니다.
    let path = cr"C:\temp";
    println!(
        "    cr\"C:\\temp\" == c\"C:\\\\temp\": {}",
        path == c"C:\\temp"
    );
    // 출력:     cr"C:\temp" == c"C:\\temp": true

    println!("3. raw 라이프타임 'r#이름");
    println!("    longer = {}", longer("러스트", "에디션 가이드"));
    // 출력:     longer = 에디션 가이드
    println!("    head = {}", head("2021-에디션", "-"));
    // 출력:     head = 2021
}
