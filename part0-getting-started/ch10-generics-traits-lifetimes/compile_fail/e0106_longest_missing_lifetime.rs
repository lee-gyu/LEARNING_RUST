// 기대하는 에러: E0106 — missing lifetime specifier
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0106_longest_missing_lifetime.rs
//
// 반환 타입 &str은 무언가를 빌린 참조입니다. 그런데 입력 참조가 x, y 두 개라서
// 반환값이 x를 빌린 것인지 y를 빌린 것인지 시그니처만 봐서는 알 수 없습니다.
// 라이프타임 생략 규칙으로도 정해지지 않으므로, 컴파일러는 추측하지 않고 표기를 요구합니다.
// 해결: fn longest<'a>(x: &'a str, y: &'a str) -> &'a str

fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("abcd");
    let string2 = "xyz";

    let result = longest(string1.as_str(), string2);
    println!("더 긴 문자열: {result}");
}
