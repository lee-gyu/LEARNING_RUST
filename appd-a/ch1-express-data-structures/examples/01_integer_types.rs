// 지침: 값이 가질 수 있는 범위에 맞는 정수 타입을 고르고, 타입을 바꾸는 일은
//       입력을 받는 경계 한 곳에서 명시적으로 합니다.
// 비권장: "넉넉하게 i64"로 읽어 들고 다니다가, u16을 요구하는 함수 앞에서 `as`로 맞춥니다.
// 권장:   처음부터 u16으로 읽어, 범위를 벗어난 값을 입력 단계에서 거부합니다.
//
// 실행: cargo run -p er-ch1-express-data-structures --example 01_integer_types

use std::num::ParseIntError;

// 설정 파일에서 읽은 포트 값이라고 생각합니다.
// 포트 번호는 0~65535이므로 둘째와 셋째 값은 포트가 될 수 없습니다.
const INPUTS: [&str; 3] = ["8080", "70000", "-1"];

// 주소를 만드는 쪽은 포트를 u16으로 받습니다.
// 표준 라이브러리의 소켓 주소 타입도 포트를 u16으로 다룹니다.
fn address(port: u16) -> String {
    format!("127.0.0.1:{port}")
}

// ---- 비권장 ----------------------------------------------------------------

// 어떤 정수든 담을 수 있게 i64로 읽습니다. 70000도 -1도 i64로는 "올바른" 값입니다.
fn load_port_wide(text: &str) -> Result<i64, ParseIntError> {
    text.parse()
}

fn connect_discouraged(text: &str) -> String {
    match load_port_wide(text) {
        // address(port)라고 쓰면 E0308(expected `u16`, found `i64`)입니다.
        // Rust는 i64를 u16으로 몰래 바꿔 주지 않습니다. 에러를 없애려고 `as`를 붙이면
        // 컴파일은 되지만, `as`는 u16에 들어가지 않는 위쪽 비트를 버리고 아무 말도 하지 않습니다.
        Ok(port) => address(port as u16),
        Err(e) => format!("거부: {e}"),
    }
}

// ---- 권장 ------------------------------------------------------------------

// 포트의 타입을 u16으로 정해 두면, 파싱이 곧 범위 검사가 됩니다.
// 이 함수가 Ok를 돌려줬다면 그 값은 언제나 올바른 포트이고, 이후 코드는 다시 확인할 필요가 없습니다.
fn load_port(text: &str) -> Result<u16, ParseIntError> {
    text.parse()
}

fn connect(text: &str) -> String {
    match load_port(text) {
        Ok(port) => address(port), // 변환이 필요 없습니다.
        Err(e) => format!("거부: {e}"),
    }
}

// ---- 넓히는 변환도 명시합니다 ---------------------------------------------

// 포트 범위에 든 포트의 개수입니다. 0~65535이면 65536개로, u16에 담을 수 없습니다.
// 그래서 반환 타입은 u32이고, 계산도 u32로 넓힌 다음에 해야 합니다.
fn ports_in_range(first: u16, last: u16) -> u32 {
    u32::from(last) - u32::from(first) + 1
}

fn main() {
    println!("[비권장] i64로 읽고 `as u16`으로 맞춤");
    for text in INPUTS {
        let shown = format!("{text:?}");
        println!("  {shown:<8} -> {}", connect_discouraged(text));
    }
    // 출력:
    //   "8080"   -> 127.0.0.1:8080
    //   "70000"  -> 127.0.0.1:4464
    //   "-1"     -> 127.0.0.1:65535

    println!("[권장] 처음부터 u16으로 읽음");
    for text in INPUTS {
        let shown = format!("{text:?}");
        println!("  {shown:<8} -> {}", connect(text));
    }
    // 출력:
    //   "8080"   -> 127.0.0.1:8080
    //   "70000"  -> 거부: number too large to fit in target type
    //   "-1"     -> 거부: invalid digit found in string

    println!("[넓히기] 0~65535 범위의 포트 개수");
    let (first, last) = (0_u16, u16::MAX);
    // u16 그대로 계산하면 마지막 `+ 1`에서 65536이 되어 u16을 넘칩니다.
    // checked_add는 넘칠 때 None을 돌려주므로 패닉 없이 그 사실을 볼 수 있습니다.
    let in_u16 = (last - first).checked_add(1);
    println!("  u16으로 계산: {in_u16:?}");
    println!("  u32로 넓힌 뒤 계산: {}", ports_in_range(first, last));
    println!("  1024~2047: {}", ports_in_range(1024, 2047));
    // 출력:
    //   u16으로 계산: None
    //   u32로 넓힌 뒤 계산: 65536
    //   1024~2047: 1024
}
