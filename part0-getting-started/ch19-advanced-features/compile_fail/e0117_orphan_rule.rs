// 기대하는 에러: E0117 — only traits defined in the current crate can be implemented for types defined outside of the crate
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0117_orphan_rule.rs
//
// Display(표준 라이브러리의 트레이트)도 Vec<String>(표준 라이브러리의 타입)도 이 크레이트의 것이 아닙니다.
// 고아 규칙(ch10)은 트레이트와 타입 중 적어도 하나가 내 것일 때만 구현을 허용합니다.
// 해결: `struct Wrapper(Vec<String>);`처럼 감싼 내 타입(뉴타입)을 만들어 그 타입에 구현합니다.
//       examples/05_newtype_and_alias.rs를 참고하세요.

use std::fmt;

impl fmt::Display for Vec<String> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.join(", "))
    }
}

fn main() {
    let words = vec![String::from("안녕"), String::from("세상")];
    println!("{words}");
}
