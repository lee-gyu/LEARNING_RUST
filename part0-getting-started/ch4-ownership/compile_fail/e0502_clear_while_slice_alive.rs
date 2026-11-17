// 기대하는 에러: E0502 — cannot borrow `s` as mutable because it is also borrowed as immutable
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0502_clear_while_slice_alive.rs
//
// word는 s의 일부를 가리키는 슬라이스(불변 참조)이고, 마지막 println!까지 쓰입니다.
// 그 사이에 clear()가 s를 비우려면 가변 참조가 필요하므로 대여 규칙에 걸립니다.
// 이 규칙이 없다면 word는 사라진 글자를 가리키게 됩니다.

fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[..i];
        }
    }

    s
}

fn main() {
    let mut s = String::from("hello world");

    let word = first_word(&s);

    s.clear();

    println!("첫 단어: {word}");
}
