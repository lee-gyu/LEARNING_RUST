// 문자열 슬라이스(`&str`)는 문자열의 연속된 일부를 가리키는 참조입니다.
// 원본과 연결된 참조이므로, 대여 규칙이 "원본이 바뀌어 슬라이스가 무의미해지는 일"을 막아 줍니다.
//
// 실행: cargo run -p p0-ch4-ownership --example 06_string_slices

// 공백 앞까지의 첫 단어를 돌려줍니다. 공백이 없으면 문자열 전체가 한 단어입니다.
// 매개변수를 `&String`이 아닌 `&str`로 받으면 String과 문자열 리터럴을 모두 받을 수 있습니다.
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes(); // 문자열을 바이트 배열의 슬라이스(&[u8])로 봅니다.

    // `iter().enumerate()`는 (인덱스, 원소의 참조) 쌍을 차례로 내줍니다(반복자는 ch13).
    // `&item`은 참조를 벗겨 바이트 값만 꺼내는 패턴입니다(패턴은 ch18).
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            // b' '는 공백 문자 하나의 바이트 값(32)입니다.
            return &s[..i];
        }
    }

    // 공백이 없으면 전체가 한 단어입니다. s는 이미 &str이므로 &s[..] 대신 s를 그대로 돌려줍니다.
    s
}

// 비교용: 슬라이스 대신 "첫 공백의 위치"라는 숫자를 돌려주는 버전입니다.
fn first_word_end(s: &str) -> usize {
    for (i, &item) in s.as_bytes().iter().enumerate() {
        if item == b' ' {
            return i;
        }
    }
    s.len()
}

fn main() {
    println!("1. 범위 문법으로 문자열의 일부를 빌립니다");
    let s = String::from("hello world");
    let hello = &s[0..5]; // 0번 바이트부터 5번 바이트 앞까지
    let world = &s[6..11];
    println!("    hello = '{hello}', world = '{world}'");
    // 시작이 0이면 생략, 끝이 len이면 생략, 둘 다 생략하면 전체입니다.
    println!(
        "    [..5] = '{}', [6..] = '{}', [..] = '{}'",
        &s[..5],
        &s[6..],
        &s[..]
    );
    // 출력:
    //     hello = 'hello', world = 'world'
    //     [..5] = 'hello', [6..] = 'world', [..] = 'hello world'

    println!("2. 슬라이스는 (시작 주소, 길이)만 가진 참조라서 복사 없이 만들어집니다");
    println!(
        "    world는 s의 6번 바이트를 가리키는가? {}",
        world.as_ptr() == s[6..].as_ptr()
    );
    println!("    world의 길이: {}", world.len());
    // 출력:
    //     world는 s의 6번 바이트를 가리키는가? true
    //     world의 길이: 5

    println!("3. 숫자(인덱스)를 돌려주면 원본과의 연결이 끊어집니다");
    let mut text = String::from("hello world");
    let end = first_word_end(&text); // 5. 이 숫자는 text와 아무 관계가 없는 그냥 usize입니다.
    text.clear(); // text가 빈 문자열이 되었지만 컴파일러는 end가 낡았다는 것을 모릅니다.
    println!("    text = '{text}', end = {end} (이제 의미 없는 값)");
    // 출력:
    //     text = '', end = 5 (이제 의미 없는 값)
    // 여기서 &text[..end]를 하면 실행 중에 패닉이 납니다.
    // first_word로 슬라이스를 받아 둔 채 clear()를 부르면 컴파일 단계에서 E0502로 막힙니다.
    // (compile_fail/e0502_clear_while_slice_alive.rs)

    println!("4. 슬라이스를 돌려주는 first_word");
    let sentence = String::from("hello world");
    let word = first_word(&sentence);
    println!("    첫 단어: '{word}'");
    // 출력:
    //     첫 단어: 'hello'

    println!("5. 문자열 리터럴의 타입이 바로 &str입니다");
    let literal: &str = "Hello, Rust";
    // 리터럴은 프로그램 바이너리 안에 들어 있는 문자열을 가리키는 슬라이스입니다.
    // 불변 참조이므로 리터럴을 수정할 수 없습니다.
    println!("    첫 단어: '{}'", first_word(literal));
    // 출력:
    //     첫 단어: 'Hello,'

    println!("6. &str 매개변수는 String, String의 일부, 리터럴을 모두 받습니다");
    let my_string = String::from("hello world");
    println!("    &my_string        → '{}'", first_word(&my_string)); // &String이 &str로 자동 변환됩니다.
    println!("    &my_string[..]    → '{}'", first_word(&my_string[..]));
    println!("    &my_string[6..]   → '{}'", first_word(&my_string[6..]));
    println!("    \"good morning\"    → '{}'", first_word("good morning"));
    // 출력:
    //     &my_string        → 'hello'
    //     &my_string[..]    → 'hello'
    //     &my_string[6..]   → 'world'
    //     "good morning"    → 'good'

    println!("7. 인덱스는 글자가 아니라 바이트 단위입니다 (한글 주의)");
    let hangul = String::from("안녕 세상");
    // 한글 한 글자는 UTF-8로 3바이트입니다. 4글자 x 3바이트 + 공백 1바이트 = 13바이트.
    println!("    \"{hangul}\"의 len(): {}", hangul.len());
    println!("    &hangul[0..3] = '{}'", &hangul[0..3]); // 첫 글자 '안'이 정확히 0..3 바이트입니다.
    println!(
        "    1번 바이트는 글자 경계인가? {}",
        hangul.is_char_boundary(1)
    );
    // &hangul[0..1]처럼 글자 중간을 자르면 컴파일은 되지만 실행 중에 패닉이 납니다.
    println!("    첫 단어: '{}'", first_word(&hangul)); // 공백(1바이트) 위치에서 자르므로 안전합니다.
    // 출력:
    //     "안녕 세상"의 len(): 13
    //     &hangul[0..3] = '안'
    //     1번 바이트는 글자 경계인가? false
    //     첫 단어: '안녕'
}
