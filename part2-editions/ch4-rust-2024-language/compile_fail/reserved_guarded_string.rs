// 에디션별 결과: 2024에서 실패, 2021에서 통과
// 기대하는 에러(2024): E 코드 없음 — 두 가지
//   invalid string literal (unprefixed guarded string literals are reserved for future use since Rust 2024)
//   reserved multi-hash token is forbidden
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/reserved_guarded_string.rs --out-dir /tmp/rust-p2-ch4   # 실패
//   rustc --edition 2021 compile_fail/reserved_guarded_string.rs --out-dir /tmp/rust-p2-ch4   # 통과(3, 3 출력)
//
// 2021 에디션에서 `#"foo"#`는 토큰 세 개(`#`, `"foo"`, `#`), `###`도 `#` 세 개였습니다.
// 2024 에디션은 앞에 `#`가 붙은 문자열 리터럴과 공백 없이 이어진 `#` 두 개 이상을 앞으로의
// 문법을 위해 예약했으므로, 매크로 입력으로도 쓸 수 없습니다.
// 해결: 컴파일러의 help대로 공백을 넣습니다. `# "foo" #`, `# # #`는 두 에디션에서 토큰 세 개입니다.
// raw 문자열 `r#"foo"#`는 이 변경과 상관없습니다.

/// 받은 토큰 트리의 개수를 셉니다.
macro_rules! token_count {
    () => {
        0
    };
    ($first:tt $($rest:tt)*) => {
        1 + token_count!($($rest)*)
    };
}

fn main() {
    println!("{}", token_count!(#"foo"#)); // → 2024에서 에러
    println!("{}", token_count!(###)); // → 2024에서 에러
}
