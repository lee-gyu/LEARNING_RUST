// 이 예제가 보여 주는 것: `char`와 `bool`의 메서드
//   char: 분류, 대소문자, 숫자 변환(`to_digit`/`from_digit`), 코드 포인트 변환(`from_u32`), UTF-8 길이
//   bool: `then_some`(즉시 평가)과 `then`(게으른 평가), `ok_or`
// 실행: cargo run -p p1-ch1-std-overview --example 06_char_bool

fn main() {
    // ── 1. 분류 메서드: 유니코드 기준과 ASCII 기준 ─────────────────────────────
    // `is_alphabetic`, `is_numeric`처럼 접두사가 없는 메서드는 유니코드 전체를 기준으로 판단하고,
    // `is_ascii_*` 메서드는 ASCII 범위(영문자, 숫자, 기호)만 봅니다.
    for c in ['a', 'Z', '가', '7', '٣', ' '] {
        println!(
            "{c:?}: alphabetic {:5} ascii_alphabetic {:5} numeric {:5} ascii_digit {:5} whitespace {}",
            c.is_alphabetic(),
            c.is_ascii_alphabetic(),
            c.is_numeric(),
            c.is_ascii_digit(),
            c.is_whitespace()
        );
    }
    // 출력: 'a': alphabetic true  ascii_alphabetic true  numeric false ascii_digit false whitespace false
    // 출력: 'Z': alphabetic true  ascii_alphabetic true  numeric false ascii_digit false whitespace false
    // 출력: '가': alphabetic true  ascii_alphabetic false numeric false ascii_digit false whitespace false
    // 출력: '7': alphabetic false ascii_alphabetic false numeric true  ascii_digit true  whitespace false
    // 출력: '٣': alphabetic false ascii_alphabetic false numeric true  ascii_digit false whitespace false
    // 출력: ' ': alphabetic false ascii_alphabetic false numeric false ascii_digit false whitespace true
    // '٣'은 아라비아-인도 숫자 3입니다. 숫자이지만 ASCII 숫자는 아닙니다.

    // ── 2. 대소문자 ──────────────────────────────────────────────────────
    // 문서: `pub const fn to_ascii_uppercase(&self) -> char`  → char 하나를 돌려줍니다.
    // 문서: `pub fn to_uppercase(self) -> ToUppercase`         → 반복자를 돌려줍니다.
    // 유니코드에서는 대문자로 바꾸면 글자가 여러 개가 되는 경우가 있기 때문입니다('ß' → "SS").
    println!("{} {}", 'q'.to_ascii_uppercase(), 'ß'.to_ascii_uppercase()); // 출력: Q ß  (ASCII가 아니면 그대로)
    // `ToUppercase`는 `Display`를 구현하므로 바로 출력할 수 있습니다.
    println!("{} {}", 'q'.to_uppercase(), 'ß'.to_uppercase()); // 출력: Q SS
    println!(
        "'ß'.to_uppercase()가 내는 글자 수 = {}",
        'ß'.to_uppercase().count()
    ); // 출력: 'ß'.to_uppercase()가 내는 글자 수 = 2
    println!("{}", '가'.is_uppercase() || '가'.is_lowercase()); // 출력: false  (한글에는 대소문자가 없음)

    // ── 3. 숫자 문자 ↔ 숫자 ───────────────────────────────────────────────
    // 문서: `pub const fn to_digit(self, radix: u32) -> Option<u32>`
    // 그 진법의 숫자가 아니면 None입니다.
    println!(
        "{:?} {:?} {:?}",
        '7'.to_digit(10),
        'f'.to_digit(16),
        'f'.to_digit(10)
    ); // 출력: Some(7) Some(15) None
    // 문서: `pub const fn from_digit(num: u32, radix: u32) -> Option<char>`
    println!(
        "{:?} {:?}",
        char::from_digit(11, 16),
        char::from_digit(11, 10)
    ); // 출력: Some('b') None

    // ── 4. 코드 포인트 ↔ char ─────────────────────────────────────────────
    // `char`는 `as u32`로 언제든 정수가 됩니다. 반대 방향은 아무 정수나 문자가 될 수 없으므로 `from_u32`가 Option을 돌려줍니다.
    println!("'가' as u32 = {}", '가' as u32); // 출력: '가' as u32 = 44032
    println!("{:?}", char::from_u32(44032 + 1)); // 출력: Some('각')
    println!("{:?}", char::from_u32(0xD800)); // 출력: None  (서로게이트 영역은 char가 될 수 없음)
    // `u8`만은 `as char`가 허용됩니다. 0~255는 모두 올바른 문자이기 때문입니다(`u32 as char`는 E0604).
    println!("{}", 65_u8 as char); // 출력: A

    // ── 5. UTF-8 길이 ────────────────────────────────────────────────────
    // `char`는 항상 4바이트지만, 문자열(UTF-8) 안에서는 글자에 따라 1~4바이트를 차지합니다.
    for c in ['a', 'é', '가', '🦀'] {
        println!("{c} → {}바이트", c.len_utf8());
    }
    // 출력: a → 1바이트
    // 출력: é → 2바이트
    // 출력: 가 → 3바이트
    // 출력: 🦀 → 4바이트
    println!("char::MAX = {:?}", char::MAX); // 출력: char::MAX = '\u{10ffff}'

    // ── 6. bool: then_some과 then ─────────────────────────────────────────
    // 문서: `pub fn then_some<T>(self, t: T) -> Option<T>`
    // 문서: `pub fn then<T, F>(self, f: F) -> Option<T> where F: FnOnce() -> T`
    // `then_some`의 인자는 `bool` 값과 상관없이 먼저 계산됩니다(즉시 평가).
    // `then`은 `true`일 때만 클로저를 부릅니다(게으른 평가). 값을 만드는 비용이 크면 `then`을 씁니다.
    let mut calls = 0;
    let a = false.then_some(make_value(&mut calls)); // false인데도 make_value가 실행됩니다.
    let b = false.then(|| make_value(&mut calls)); // 클로저가 불리지 않으므로 실행되지 않습니다.
    let c = true.then(|| make_value(&mut calls));
    println!("a = {a:?}, b = {b:?}, c = {c:?}, 호출 횟수 = {calls}"); // 출력: a = None, b = None, c = Some(200), 호출 횟수 = 2

    // "조건이 맞을 때만 값이 있는 Option"을 한 줄로 만들 때 씁니다(아래 `half` 함수).
    // 결과는 `Option`이므로 Part 0에서 배운 `if let`이나 `match`로 꺼냅니다.
    if let Some(h) = half(10) {
        println!("half(10) = {h}"); // 출력: half(10) = 5
    }
    println!("half(7) = {:?}", half(7)); // 출력: half(7) = None

    // ── 7. bool: ok_or (1.98.0부터) ───────────────────────────────────────
    // 문서: `pub fn ok_or<E>(self, err: E) -> Result<(), E>` — 이 툴체인(1.98.1)에서 막 안정화된 메서드입니다.
    // 1.98.0보다 오래된 툴체인에서는 이 줄이 E0599(메서드 없음)로 컴파일되지 않습니다.
    let age = 15;
    println!("{:?}", (age >= 19).ok_or("미성년자")); // 출력: Err("미성년자")

    // `bool`은 `as`로 정수가 됩니다(true = 1, false = 0).
    println!("{} {}", true as i32, false as u8); // 출력: 1 0
}

// 값을 만드는 데 비용이 드는 함수라고 생각합니다. 몇 번 불렸는지 `calls`에 셉니다.
fn make_value(calls: &mut i32) -> i32 {
    *calls += 1;
    *calls * 100
}

// 짝수일 때만 절반을 돌려줍니다. `then_some`의 인자 `n / 2`는 홀수일 때도 계산되지만 비용이 거의 없으므로 괜찮습니다.
// `is_multiple_of`(1.87.0)는 부호 없는 정수의 메서드로 `n % 2 == 0`과 같은 뜻입니다(Clippy가 이쪽을 권합니다).
fn half(n: u32) -> Option<u32> {
    n.is_multiple_of(2).then_some(n / 2)
}
