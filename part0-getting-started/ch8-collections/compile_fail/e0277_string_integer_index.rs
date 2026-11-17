// 기대하는 에러: E0277 — the type `str` cannot be indexed by `{integer}`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_string_integer_index.rs
//
// String은 UTF-8 바이트를 담고 있어서, s[0]이 돌려줄 수 있는 것은 '안'이 아니라 '안'의 첫 바이트(236)뿐입니다.
// 글자가 아닌 값을 조용히 돌려주는 대신, Rust는 문자열의 정수 인덱싱을 아예 허용하지 않습니다.

fn main() {
    let s = String::from("안녕하세요");

    let first = s[0];

    println!("첫 글자: {first}");
}
