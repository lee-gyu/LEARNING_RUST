// 기대하는 에러: E0004 (두 개) — non-exhaustive patterns: `Case::Smart` not covered
//
// 확인: rustc --edition 2024 compile_fail/e0004_new_case_variant.rs
//
// examples/03_bool_args_to_enum.rs의 Case에 세 번째 선택지 Smart를 추가했습니다.
// Smart는 "찾는 말에 대문자가 있으면 구분하고, 없으면 무시"하는 방식입니다.
// case_sensitive: bool이었다면 세 번째 값을 넣을 자리가 없어 매개변수를 통째로 바꿔야 했을 것입니다.
// 열거형에서는 배리언트를 하나 추가하면, 컴파일러가 아직 Smart를 처리하지 않은 match를 모두 찾아 줍니다.

#[derive(Clone, Copy)]
enum Case {
    Sensitive,
    Insensitive,
    Smart, // 새로 추가한 배리언트
}

impl Case {
    fn label(self) -> &'static str {
        match self {
            // E0004
            Case::Sensitive => "대소문자 구분",
            Case::Insensitive => "대소문자 무시",
        }
    }

    fn normalize(self, s: &str) -> String {
        match self {
            // E0004
            Case::Sensitive => s.to_string(),
            Case::Insensitive => s.to_lowercase(),
        }
    }
}

fn main() {
    let case = Case::Smart;
    println!("{}: {}", case.label(), case.normalize("Rust"));
}
