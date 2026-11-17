// The Book 8장 끝의 연습 과제 풀이 ②: 영어 단어를 피그 라틴(Pig Latin)으로 바꾸기
// 규칙: 자음으로 시작하는 단어는 첫 자음을 끝으로 옮기고 "ay"를 붙입니다(first → irst-fay).
//       모음으로 시작하는 단어는 끝에 "hay"를 붙입니다(apple → apple-hay).
// 첫 글자를 꺼내고 나머지를 자르는 과정에서 UTF-8 글자 경계를 지키는 방법에 주목하세요.
//
// 실행: cargo run -p p0-ch8-collections --example 11_pig_latin

// 영어 모음인지 확인합니다. 대문자도 소문자로 바꿔서 비교합니다.
fn is_vowel(c: char) -> bool {
    // "aeiou".contains(문자)는 그 문자가 "aeiou" 안에 있으면 true입니다.
    "aeiou".contains(c.to_ascii_lowercase())
}

// 단어 하나를 피그 라틴으로 바꿉니다. 새 문자열을 만들어 돌려주므로 반환 타입은 String입니다.
fn pig_latin_word(word: &str) -> String {
    // chars()로 글자를 하나씩 내주는 반복자를 만들고, next()로 첫 글자를 Option<char>로 꺼냅니다(반복자는 ch13).
    let first = match word.chars().next() {
        Some(c) => c,
        None => return String::new(), // 빈 단어면 빈 문자열을 돌려줍니다.
    };

    // 영어 알파벳으로 시작하지 않는 단어(한글, 숫자 등)는 규칙을 적용하지 않고 그대로 돌려줍니다.
    if !first.is_ascii_alphabetic() {
        return word.to_string();
    }

    if is_vowel(first) {
        format!("{word}-hay")
    } else {
        // 첫 글자를 뺀 나머지를 슬라이스로 얻습니다. 슬라이스 범위는 바이트 단위이므로,
        // 1을 쓰는 대신 첫 글자가 실제로 차지하는 바이트 수(len_utf8)만큼 건너뜁니다.
        // 이렇게 하면 나중에 규칙을 넓혀 'é' 같은 2바이트 글자를 받아도 글자 중간을 자르지 않습니다.
        let rest = &word[first.len_utf8()..];
        format!("{rest}-{first}ay")
    }
}

// 문장의 단어마다 pig_latin_word를 적용하고, 공백 하나로 이어 붙입니다.
fn pig_latin(sentence: &str) -> String {
    let mut result = String::new();
    for word in sentence.split_whitespace() {
        if !result.is_empty() {
            result.push(' '); // 두 번째 단어부터는 앞에 공백을 넣습니다.
        }
        result.push_str(&pig_latin_word(word)); // String을 &로 빌려 &str 자리에 넘깁니다.
    }
    result
}

fn main() {
    println!("1. 단어 하나씩 바꿔 봅니다");
    for word in ["first", "apple", "string", "Hello"] {
        println!("    {word} → {}", pig_latin_word(word));
    }
    // 출력:
    //     first → irst-fay
    //     apple → apple-hay
    //     string → tring-say
    //     Hello → ello-Hay

    println!("2. 문장 전체를 바꿉니다");
    let sentence = "rust is fun and easy";
    println!("    {sentence}");
    println!("    → {}", pig_latin(sentence));
    // 출력:
    //     rust is fun and easy
    //     → ust-ray is-hay un-fay and-hay easy-hay

    println!("3. 영어가 아닌 단어가 섞여도 패닉하지 않습니다");
    // 알파벳 검사 없이 &word[1..]로 잘랐다면 "러스트"에서 '러'(0~2번 바이트)의 중간을 잘라 패닉했을 것입니다.
    //     start byte index 1 is not a char boundary; it is inside '러' (bytes 0..3 of string)
    // 이 풀이는 알파벳으로 시작하지 않는 단어를 그대로 두고, 자를 때도 len_utf8을 써서 두 겹으로 막습니다.
    let mixed = "Hello 러스트 world";
    println!("    {mixed}");
    println!("    → {}", pig_latin(mixed));
    // 출력:
    //     Hello 러스트 world
    //     → ello-Hay 러스트 orld-way
}
