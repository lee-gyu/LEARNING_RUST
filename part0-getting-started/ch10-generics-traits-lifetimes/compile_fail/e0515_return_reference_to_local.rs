// 기대하는 에러: E0515 — cannot return value referencing local variable `result`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0515_return_reference_to_local.rs
//
// 라이프타임 'a를 적어 E0106은 피했지만, 반환하는 참조가 매개변수가 아니라
// 함수 안에서 만든 String을 가리킵니다. 그 String은 함수가 끝날 때 drop되므로
// 이 참조는 댕글링 참조가 됩니다. 라이프타임 표기로는 값의 수명을 늘릴 수 없습니다.
// 해결: 참조 대신 소유한 값(String)을 반환해 소유권을 호출한 쪽으로 넘깁니다.

fn longest<'a>(_x: &str, _y: &str) -> &'a str {
    let result = String::from("really long string");
    result.as_str()
}

fn main() {
    let string1 = String::from("abcd");
    let string2 = "xyz";

    let result = longest(string1.as_str(), string2);
    println!("더 긴 문자열: {result}");
}
