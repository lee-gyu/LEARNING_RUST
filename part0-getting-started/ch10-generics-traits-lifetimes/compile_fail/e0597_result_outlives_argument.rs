// 기대하는 에러: E0597 — `string2` does not live long enough
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0597_result_outlives_argument.rs
//
// longest의 시그니처는 "반환값은 x와 y가 둘 다 유효한 동안만 유효하다"고 말합니다.
// result를 string2의 스코프 밖에서 쓰면, 그 약속을 지킬 수 없으므로 거부됩니다.
// 사람 눈에는 result가 더 긴 string1을 가리킨다는 것이 보이지만, 컴파일러는 본문이 아니라
// 시그니처만 보고 판단합니다. 라이프타임 표기는 string2를 더 오래 살게 해 주지 않습니다.
// 해결: result를 안쪽 블록 안에서 쓰거나, string2를 바깥 스코프에 선언합니다.

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("long string is long");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(string1.as_str(), string2.as_str());
    } // string2가 여기서 drop됩니다.
    println!("더 긴 문자열: {result}");
}
