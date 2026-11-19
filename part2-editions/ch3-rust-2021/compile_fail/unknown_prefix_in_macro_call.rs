// 기대하는 에러: (E 코드 없음)
// 핵심 메시지: prefix `z` is unknown
// 에디션: 2021, 2024에서 실패 / 2018에서 통과(실행하면 2)
//
// 2018 에디션에서는 z"hey"가 식별자 z와 문자열 "hey", 두 토큰이었습니다.
// 2021 에디션부터 "식별자 바로 뒤에 붙은 따옴표"는 접두사가 붙은 리터럴로 예약되었고,
// z는 정해진 접두사(b, r, br, c, cr)가 아니므로 토큰으로 나누는 단계에서 에러가 납니다.
// 고치려면 z "hey"처럼 사이를 띄웁니다.
//
// 확인: rustc --edition 2024 compile_fail/unknown_prefix_in_macro_call.rs --out-dir /tmp/rust-p2-ch3
//       rustc --edition 2018 compile_fail/unknown_prefix_in_macro_call.rs --out-dir /tmp/rust-p2-ch3

/// 받은 토큰 트리의 개수를 셉니다.
macro_rules! count_tt {
    () => { 0usize };
    ($first:tt $($rest:tt)*) => { 1usize + count_tt!($($rest)*) };
}

fn main() {
    println!("{}", count_tt!(z"hey")); // → prefix `z` is unknown
}
