// Q7.6 — 같은 문자열의 "길이"와 "첫 글자"가 언어마다 다른 까닭
//
// 언어마다 문자열을 무엇의 나열로 보는지가 다릅니다.
//
//   Go      len(s), s[0]          바이트(UTF-8)
//   Java    s.length(), charAt(0) UTF-16 코드 단위
//   JS      s.length, s[0]        UTF-16 코드 단위
//   Python  len(s), s[0]          유니코드 코드 포인트
//
// Rust의 `str`은 UTF-8 바이트이고, 무엇으로 셀지를 메서드 이름으로 고릅니다.
//   s.len()                  바이트 수(Go와 같음)
//   s.encode_utf16().count() UTF-16 코드 단위 수(Java·JS와 같음)
//   s.chars().count()        `char`(유니코드 스칼라 값) 수(Python과 같음)
//
// `len()`이 바이트 수이고 `chars().count()`가 글자 수라는 것은 Part 0 ch8 예제 07에서 봤습니다. 여기서는
// 다른 언어가 세는 방식(UTF-16)과 나란히 놓고, 어느 쪽도 "사람이 보는 글자 수"는 아닐 수 있다는 것을 봅니다.
//
// 실행: cargo run -p faq-ch7-syntax-type-surprises --example 01_string_lengths

fn show(label: &str, s: &str) {
    println!(
        "  {label}: 바이트 {} / UTF-16 {} / char {}",
        s.len(),
        s.encode_utf16().count(),
        s.chars().count()
    );
}

fn main() {
    println!("1. 세 가지 길이");
    show("Rust", "Rust");
    show("한글", "한글");
    show("🦀", "🦀");
    // e 뒤에 결합 부호(combining mark) U+0301을 붙이면 화면에는 é 한 글자로 보입니다.
    show("e + U+0301", "e\u{301}");
    // 출력:   Rust: 바이트 4 / UTF-16 4 / char 4
    // 출력:   한글: 바이트 6 / UTF-16 2 / char 2
    // 출력:   🦀: 바이트 4 / UTF-16 2 / char 1
    // 출력:   e + U+0301: 바이트 3 / UTF-16 2 / char 2

    println!("2. \"🦀\"의 첫 단위를 꺼내면");
    let crab = "🦀";
    println!("  첫 바이트(Go의 s[0]): {:#04x}", crab.as_bytes()[0]);
    let first_unit = crab.encode_utf16().next().unwrap();
    println!("  첫 UTF-16 단위(Java의 charAt(0), JS의 s[0]): {first_unit:#06x}");
    println!(
        "  첫 char(Python의 s[0]): {:?}",
        crab.chars().next().unwrap()
    );
    // 출력:   첫 바이트(Go의 s[0]): 0xf0
    // 출력:   첫 UTF-16 단위(Java의 charAt(0), JS의 s[0]): 0xd83e
    // 출력:   첫 char(Python의 s[0]): '🦀'
    // 0xd83e는 두 단위가 짝을 이뤄야 뜻이 있는 서로게이트(surrogate)의 앞쪽 절반입니다.
}
