// 지침 2-②: 함수가 볼 수 없는 "부른 곳의 정보"가 필요할 때는 매크로를 쓰되, 매크로는 얇게 둡니다.
//
// 함수는 인자의 값만 받습니다. 인자를 만든 식의 글자(`stock >= ordered`)나 부른 파일과 줄은 알 수 없습니다.
// 매크로는 `stringify!`로 식의 글자를, `file!()`/`line!()`으로 부른 위치를 컴파일할 때 넣을 수 있습니다.
// 다만 위치만 필요하다면 `#[track_caller]` 함수로 충분합니다(1.46부터).
//
// 실행: cargo run -p er-ch28-macros-judiciously --example 03_call_site

use std::panic::Location;

// 1. 보통 함수: 실패했다는 것만 압니다.
fn check_v1(ok: bool) -> bool {
    if !ok {
        println!("  검사 실패");
    }
    ok
}

// 2. #[track_caller] 함수: 이 함수를 부른 위치를 Location::caller()로 압니다. 식의 글자는 여전히 모릅니다.
#[track_caller]
fn check_v2(ok: bool) -> bool {
    if !ok {
        let at = Location::caller();
        println!("  검사 실패 ({}:{})", at.file(), at.line());
    }
    ok
}

// 3. 매크로: 식의 글자와 위치를 모두 넣습니다.
// 일은 보통 함수 `check_at`이 하고, 매크로는 컴파일할 때만 알 수 있는 것을 덧붙여 한 번 부르기만 합니다.
// 그래서 `$cond`는 함수 인자로 정확히 한 번 평가되고(지침 3), 숨은 return도 없습니다(지침 4).
fn check_at(ok: bool, expr: &str, file: &str, line: u32) -> bool {
    if !ok {
        println!("  검사 실패: `{expr}` ({file}:{line})");
    }
    ok
}

macro_rules! check {
    ($cond:expr) => {
        check_at($cond, stringify!($cond), file!(), line!())
    };
}

fn main() {
    let stock: u32 = 3;
    let ordered: u32 = 5;
    let price: u32 = 1_200;

    println!("1. 보통 함수");
    check_v1(price > 0);
    check_v1(stock >= ordered); // 출력:   검사 실패

    println!("2. #[track_caller] 함수");
    check_v2(price > 0);
    check_v2(stock >= ordered); // 출력:   검사 실패 (appd-a/ch28-macros-judiciously/examples/03_call_site.rs:56)

    println!("3. 매크로");
    check!(price > 0);
    let passed = check!(stock >= ordered); // 출력:   검사 실패: `stock >= ordered` (appd-a/ch28-macros-judiciously/examples/03_call_site.rs:60)
    println!("  돌려받은 값: {passed}"); // 출력:   돌려받은 값: false
}
