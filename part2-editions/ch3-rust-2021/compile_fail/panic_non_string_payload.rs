// 기대하는 에러: (E 코드 없음)
// 핵심 메시지: format argument must be a string literal
// 에디션: 2021, 2024에서 실패 / 2018에서는 경고(non_fmt_panics)와 함께 통과
//          (2018에서 실행하면 페이로드가 i32인 패닉이 일어나 종료 코드 101로 끝납니다.)
//
// 2018 에디션의 panic!은 인자가 하나뿐이면 그 값을 서식 처리하지 않고 페이로드로 그대로 넘겼습니다.
// 2021 에디션부터 panic!은 println!처럼 첫 인자를 언제나 서식 문자열로 처리하므로
// 문자열 리터럴이 아닌 값을 첫 인자로 받지 않습니다.
// 메시지가 필요하면 panic!("{}", code), 값 자체를 페이로드로 넘기려면 std::panic::panic_any(code)를 씁니다.
//
// 확인: rustc --edition 2024 compile_fail/panic_non_string_payload.rs --out-dir /tmp/rust-p2-ch3
//       rustc --edition 2018 compile_fail/panic_non_string_payload.rs --out-dir /tmp/rust-p2-ch3

fn check_status(code: u16) {
    if code >= 400 {
        panic!(code); // → format argument must be a string literal
    }
}

fn main() {
    check_status(404);
}
