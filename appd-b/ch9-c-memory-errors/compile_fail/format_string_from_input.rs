// 기대하는 에러: 에러 코드 없음
// 핵심 메시지: error: format argument must be a string literal
//
// C에서 사용자 입력을 printf의 서식 문자열로 바로 넘기면(printf(user)), 입력에 든 %s나 %n이 서식 지정자로
// 해석되어 메모리를 읽거나 씁니다. clang은 -Wformat-security 경고를 내지만 컴파일은 됩니다. Rust의 println!은
// 서식 문자열을 컴파일 시간에 해석하므로 문자열 리터럴만 받고, 실행 중에 만든 문자열은 서식으로 쓸 수 없습니다.
//
// 고치는 법: 컴파일러의 help대로 println!("{}", user_input); 로 써서 입력을 데이터로 넘깁니다
// (examples/02_buffers_and_strings.rs의 3번).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch9-c-memory-errors compile_fail/format_string_from_input.rs

fn main() {
    let user_input = String::from("%s%s%n");
    println!(user_input); // 서식 문자열 자리에 입력을 넣음
}
