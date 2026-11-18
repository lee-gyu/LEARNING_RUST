// 이 예제가 보여 주는 것: 새 String을 만들어 돌려주는 메서드 — 바꾸기(replace, replacen,
//                        대소문자 변환, repeat)와 잇기(concat, join, char를 모아 String 만들기)
// 실행: cargo run -p p1-ch3-strings-fmt --example 03_new_strings
//
// 시그니처가 `fn replace(&self, ..) -> String`처럼 "&self를 받아 String을 돌려주면" 새 힙 메모리를
// 할당해 결과를 만든다는 뜻입니다. 원본은 빌려 읽기만 하므로 그대로 남습니다.

fn main() {
    // 1. replace, replacen: 원본은 그대로, 바뀐 사본을 돌려줍니다.
    println!("== 1. replace, replacen ==");
    let phone = "010-1234-5678";
    let digits = phone.replace('-', ""); // 검색 패턴 자리라서 char, &str, 클로저 등을 넣을 수 있습니다.
    println!("phone = {phone}, digits = {digits}"); // 출력: phone = 010-1234-5678, digits = 01012345678
    let masked = phone.replacen(|c: char| c.is_ascii_digit(), "*", 3); // 앞에서부터 3번만 바꿉니다.
    println!("replacen(숫자, \"*\", 3) = {masked}"); // 출력: replacen(숫자, "*", 3) = ***-1234-5678
    let spaced = "a  b   c".replace("  ", " "); // 겹치지 않게 앞에서부터 한 번 훑으며 바꿉니다.
    println!("{spaced:?}"); // 출력: "a b  c"

    // 2. 대소문자: 유니코드 규칙을 따르므로 글자 수와 바이트 수가 달라질 수 있습니다.
    // 그래서 &str을 제자리에서 바꾸지 못하고 새 String을 돌려줍니다.
    println!("\n== 2. 대소문자 ==");
    println!("{}", "Hello, 러스트!".to_uppercase()); // 출력: HELLO, 러스트!
    let street = "Straße";
    let upper = street.to_uppercase();
    println!(
        "{street} ({}글자) → {upper} ({}글자)",
        street.chars().count(),
        upper.chars().count()
    ); // 출력: Straße (6글자) → STRASSE (7글자)
    let liga = "ﬁ"; // f와 i를 붙여 쓴 합자 한 글자(U+FB01)
    let liga_upper = liga.to_uppercase();
    println!(
        "{liga} ({}바이트) → {liga_upper} ({}바이트)",
        liga.len(),
        liga_upper.len()
    ); // 출력: ﬁ (3바이트) → FI (2바이트)
    // 그리스어 시그마는 단어 끝에서 다른 소문자(ς)가 됩니다. 글자 하나씩이 아니라 문맥을 봅니다.
    println!("{}", "ΣΑΣ".to_lowercase()); // 출력: σας
    // ASCII만 바꾸는 버전은 바이트 수가 변하지 않으므로 제자리 수정(make_*)도 있습니다.
    println!("{}", "Straße".to_ascii_uppercase()); // 출력: STRAßE
    let mut code = String::from("abc-ß");
    code.make_ascii_uppercase(); // &mut self: 새 String을 만들지 않습니다.
    println!("{code}"); // 출력: ABC-ß

    // 3. repeat
    println!("\n== 3. repeat ==");
    println!("{}", "=".repeat(12)); // 출력: ============
    println!("{}", "하하".repeat(3)); // 출력: 하하하하하하

    // 4. 잇기: concat과 join은 슬라이스(배열·벡터)의 메서드입니다.
    println!("\n== 4. concat, join ==");
    let parts = ["2026", "11", "18"];
    println!("concat() = {}", parts.concat()); // 출력: concat() = 20261118
    println!("join(\"-\") = {}", parts.join("-")); // 출력: join("-") = 2026-11-18
    // Vec<String>도 같습니다. 원소가 String이어도 결과는 새 String 하나입니다.
    let names = vec![
        String::from("페리스"),
        String::from("코랄"),
        String::from("러스티"),
    ];
    println!("join(\", \") = {}", names.join(", ")); // 출력: join(", ") = 페리스, 코랄, 러스티
    println!("names는 그대로: {names:?}"); // 출력: names는 그대로: ["페리스", "코랄", "러스티"]

    // 5. char와 String 사이
    println!("\n== 5. char를 모아 String 만들기 ==");
    // chars()가 내주는 char를 collect로 모으면 String이 됩니다(결과 타입을 적어 줘야 합니다).
    let reversed: String = "가나다".chars().rev().collect();
    println!("뒤집기: {reversed}"); // 출력: 뒤집기: 다나가
    // 빈 문자열이면 첫 글자가 없으므로 next()는 Option<char>를 돌려줍니다.
    let initials: String = ["Rust", "Is", "Fun"]
        .iter()
        .map(|w| w.chars().next().unwrap_or('?'))
        .collect();
    println!("머리글자: {initials}"); // 출력: 머리글자: RIF
    // char 하나를 String으로: to_string() 또는 String::from
    let c = '漢';
    let s1 = c.to_string();
    let s2 = String::from(c);
    println!("{s1} {s2} (len_utf8 = {})", c.len_utf8()); // 출력: 漢 漢 (len_utf8 = 3)

    // 6. 소유한 String이 필요할 때와 빌린 &str이면 될 때
    println!("\n== 6. String ↔ &str ==");
    let borrowed: &str = "리터럴";
    let owned: String = borrowed.to_string(); // 복사본을 새로 할당합니다. to_owned(), String::from도 같습니다.
    let view: &str = owned.as_str(); // 할당 없이 다시 빌립니다. &owned로 써도 &str 자리에 들어갑니다.
    println!("{borrowed} / {owned} / {view}"); // 출력: 리터럴 / 리터럴 / 리터럴
}
