// 이 예제가 보여 주는 것: 대여 검사기가 "참조가 쓰이는 구간"과 "값이 살아 있는 구간"을 비교하는 방식,
//                        함수 시그니처의 라이프타임 표기(longest)가 참조 사이의 관계를 설명할 뿐 수명을 바꾸지 않는다는 점
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 07_longest_lifetime

// 반환값이 x를 빌린 것일 수도, y를 빌린 것일 수도 있습니다. 그래서 둘 다에 같은 라이프타임 'a를 붙여
// "반환된 참조는 x와 y가 모두 유효한 동안만 쓸 수 있다"고 적습니다.
// 이 표기가 없으면 E0106입니다(compile_fail/e0106_longest_missing_lifetime.rs).
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// 반환값이 text만 빌리고 prefix와는 상관없을 때는 text에만 'a를 붙입니다.
// strip_prefix는 text가 prefix로 시작하면 그 뒷부분을 Some으로, 아니면 None을 돌려줍니다(Option은 ch6).
fn strip_tag<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

// 함수 안에서 새로 만든 문자열은 함수가 끝나면 drop되므로 참조로 돌려줄 수 없습니다(E0515).
// 소유한 값(String)으로 돌려주면 소유권이 호출한 쪽으로 넘어가므로 아무 문제가 없습니다.
fn emphasize(word: &str) -> String {
    format!("**{word}**")
}

fn main() {
    println!("1. 참조는 가리키는 값이 살아 있는 동안에만 쓸 수 있습니다");
    let r;
    {
        let x = 5;
        r = &x;
        println!("    블록 안에서 r = {r}"); // r의 마지막 사용이 x가 살아 있는 블록 안이므로 괜찮습니다.
    } // x가 여기서 drop됩니다.
    // 출력: 블록 안에서 r = 5
    // 아래 줄의 주석을 풀면 E0597(`x` does not live long enough)이 납니다.
    // r을 쓰는 구간이 x가 살아 있는 구간 밖까지 늘어나기 때문입니다.
    // println!("{r}");

    println!("2. longest는 두 문자열 슬라이스 중 긴 쪽을 돌려줍니다");
    let string1 = String::from("abcd");
    let string2 = "xyz";
    let result = longest(string1.as_str(), string2);
    println!("    더 긴 문자열: {result}");
    // 출력: 더 긴 문자열: abcd

    println!("3. 'a는 두 인자가 모두 유효한 구간, 즉 더 짧은 쪽에 맞춰집니다");
    let long_lived = String::from("long string is long");
    {
        let short_lived = String::from("xyz");
        let result = longest(long_lived.as_str(), short_lived.as_str());
        // result를 short_lived가 살아 있는 이 블록 안에서만 쓰므로 괜찮습니다.
        println!("    블록 안에서: {result}");
    }
    // 출력: 블록 안에서: long string is long
    // result를 블록 밖에서 쓰면 E0597입니다(compile_fail/e0597_result_outlives_argument.rs).
    // 실제로는 long_lived를 가리키더라도, 컴파일러는 본문이 아니라 시그니처만 보고 판단합니다.

    println!("4. 라이프타임 표기는 관계를 설명할 뿐, 값을 더 오래 살게 하지 않습니다");
    let line = String::from("[공지] 내일은 휴강입니다");
    let body;
    {
        let tag = String::from("[공지] ");
        body = strip_tag(&line, &tag);
    } // tag가 여기서 drop되지만, body는 line만 빌렸으므로 계속 쓸 수 있습니다.
    println!("    본문: {body}");
    // 출력: 본문: 내일은 휴강입니다
    // strip_tag의 prefix에도 'a를 붙이면 본문은 그대로인데 이 코드가 E0597로 거부됩니다.
    // 시그니처가 "결과는 tag도 빌린다"고 말하게 되기 때문입니다.

    println!("5. 함수 안에서 만든 값은 소유한 값으로 돌려줍니다");
    let word = emphasize("러스트");
    println!("    {word}");
    // 출력: **러스트**
}
