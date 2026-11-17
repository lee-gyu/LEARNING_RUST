// String이 UTF-8 바이트로 저장된다는 사실과, 그래서 `s[0]` 같은 정수 인덱싱이 없는 이유를 보여 줍니다.
// 같은 문자열을 바이트 단위, 문자(char) 단위로 각각 살펴보고, 슬라이스를 안전하게 자르는 방법을 확인합니다.
//
// 실행: cargo run -p p0-ch8-collections --example 07_string_utf8

fn main() {
    println!("1. len()은 글자 수가 아니라 바이트 수입니다");
    let english = String::from("hello");
    let korean = String::from("안녕하세요");
    // chars()는 문자열을 char 단위로 하나씩 내주고, count()는 그 개수를 셉니다.
    println!(
        "    \"{english}\": {}바이트, {}글자",
        english.len(),
        english.chars().count()
    );
    println!(
        "    \"{korean}\": {}바이트, {}글자",
        korean.len(),
        korean.chars().count()
    );
    // 출력:
    //     "hello": 5바이트, 5글자
    //     "안녕하세요": 15바이트, 5글자

    println!("2. UTF-8에서는 글자마다 차지하는 바이트 수가 다릅니다");
    for c in ['A', 'é', '한', '😀'] {
        // len_utf8()은 그 char를 UTF-8로 저장할 때 필요한 바이트 수입니다.
        println!("    '{c}' → {}바이트", c.len_utf8());
    }
    // 출력:
    //     'A' → 1바이트
    //     'é' → 2바이트
    //     '한' → 3바이트
    //     '😀' → 4바이트

    println!("3. String 안에 실제로 들어 있는 것은 바이트(u8)입니다");
    let han = "한";
    println!("    \"한\".as_bytes() = {:?}", han.as_bytes()); // 출력: "한".as_bytes() = [237, 149, 156]
    // 만약 han[0]이 허용된다면 돌려줄 수 있는 것은 237이라는 바이트뿐입니다.
    // 237은 '한'의 일부일 뿐 글자가 아니므로, Rust는 정수 인덱싱 자체를 컴파일 에러로 막습니다
    // (compile_fail/e0277_string_integer_index.rs).
    // 또 n번째 글자를 찾으려면 앞에서부터 바이트를 해석해 나가야 해서,
    // "인덱싱은 즉시(상수 시간에) 끝난다"는 기대도 지킬 수 없습니다.

    println!("4. bytes()와 chars()로 원하는 단위를 골라 순회합니다");
    print!("    bytes():");
    for b in "안녕".bytes() {
        print!(" {b}");
    }
    println!();
    print!("    chars():");
    for c in "안녕".chars() {
        print!(" {c}");
    }
    println!();
    // 출력:
    //     bytes(): 236 149 136 235 133 149
    //     chars(): 안 녕

    println!("5. n번째 글자가 필요하면 chars().nth(n)을 씁니다");
    // nth(n)은 앞에서부터 n개를 건너뛰고 다음 글자를 Option<char>로 돌려줍니다.
    // 글자 수가 n보다 적으면 None입니다. 앞에서부터 세므로 문자열이 길수록 오래 걸립니다.
    println!("    korean.chars().nth(2) = {:?}", korean.chars().nth(2)); // 출력: korean.chars().nth(2) = Some('하')
    println!("    korean.chars().nth(9) = {:?}", korean.chars().nth(9)); // 출력: korean.chars().nth(9) = None

    println!("6. 슬라이스 범위는 바이트 단위이고, 글자 경계에 맞아야 합니다");
    println!("    &korean[0..3] = {}", &korean[0..3]); // 출력: &korean[0..3] = 안
    println!("    &korean[3..9] = {}", &korean[3..9]); // 출력: &korean[3..9] = 녕하
    // 1번 바이트는 '안'(0~2번 바이트)의 중간이므로 글자 경계가 아닙니다.
    let boundary = korean.is_char_boundary(1);
    println!("    1번 바이트가 글자 경계인가? {boundary}"); // 출력: 1번 바이트가 글자 경계인가? false
    // 아래 줄의 주석을 풀면 컴파일은 되지만, 실행하면 다음 메시지로 패닉합니다(종료 코드 101).
    //     end byte index 1 is not a char boundary; it is inside '안' (bytes 0..3 of string)
    // println!("{}", &korean[0..1]);
    // 범위가 경계에 맞는지 모를 때는 get(범위)를 쓰면, 패닉 대신 None을 받습니다.
    println!("    korean.get(0..1) = {:?}", korean.get(0..1)); // 출력: korean.get(0..1) = None
    println!("    korean.get(0..3) = {:?}", korean.get(0..3)); // 출력: korean.get(0..3) = Some("안")

    println!("7. 사람이 보는 한 글자가 char 하나라는 보장도 없습니다");
    // 같은 '한'을 완성형(코드 포인트 1개)과 조합형(초성 ㅎ + 중성 ㅏ + 종성 ㄴ, 코드 포인트 3개)으로 적어 봅니다.
    // \u{...}는 코드 포인트 번호로 글자를 쓰는 문법입니다. 화면에는 둘 다 '한'으로 보이는 것이 보통입니다.
    let composed = "한";
    let decomposed = "\u{1112}\u{1161}\u{11AB}";
    println!("    화면에 보이는 모양: [{composed}] [{decomposed}]");
    // 출력: 화면에 보이는 모양: [한] [한]
    // (두 번째 '한'은 이 주석에도 조합형 문자 3개로 적혀 있습니다.
    //  조합형이 어떻게 보이는지는 터미널과 글꼴에 따라 다르며, 따로 떨어진 ㅎㅏㄴ으로 보일 수도 있습니다.)
    println!(
        "    완성형: {}바이트, char {}개",
        composed.len(),
        composed.chars().count()
    );
    println!(
        "    조합형: {}바이트, char {}개",
        decomposed.len(),
        decomposed.chars().count()
    );
    print!("    조합형의 char:");
    for c in decomposed.chars() {
        print!(" U+{:04X}", c as u32); // char를 u32로 바꾸면 코드 포인트 번호가 됩니다.
    }
    println!();
    println!("    완성형 == 조합형? {}", composed == decomposed);
    // 출력:
    //     완성형: 3바이트, char 1개
    //     조합형: 9바이트, char 3개
    //     조합형의 char: U+1112 U+1161 U+11AB
    //     완성형 == 조합형? false
    //
    // 사람이 한 글자로 인식하는 단위를 자소 클러스터(grapheme cluster)라고 부릅니다.
    // 표준 라이브러리는 이 단위로 나누는 기능을 제공하지 않으며, 필요하면 crates.io의 크레이트를 씁니다.
}
