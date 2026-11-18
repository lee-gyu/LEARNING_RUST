// 이 예제가 보여 주는 것: 조건 묻기 — is_some_and, is_none_or, is_ok_and, is_err_and, filter
// 실행: cargo run -p p1-ch2-option-result --example 04_queries
//
// is_some()/is_ok()는 "값이 있는가"만 묻습니다. 실제로는 "값이 있고, 그 값이 조건을 만족하는가"를
// 묻는 경우가 많은데, 이때 match나 map_or(false, ...)를 쓰는 대신 아래 메서드를 씁니다.

use std::num::IntErrorKind;

// "전" 버전: map_or로 "없으면 false, 있으면 조건"을 표현했습니다.
// clippy는 이 모양에 is_some_and를 쓰라고 제안합니다(clippy::unnecessary_map_or).
#[expect(clippy::unnecessary_map_or)]
fn is_adult_map_or(age: Option<u32>) -> bool {
    age.map_or(false, |a| a >= 19)
}

// "후" 버전. 시그니처: fn is_some_and(self, f: impl FnOnce(T) -> bool) -> bool
// None이면 클로저를 부르지 않고 false입니다.
fn is_adult(age: Option<u32>) -> bool {
    age.is_some_and(|a| a >= 19)
}

// is_none_or: "없거나, 있다면 조건을 만족하는가". 선택 입력을 검사할 때 맞는 질문입니다.
// 할인 코드는 입력하지 않아도 되지만, 입력했다면 6글자여야 한다고 해 봅시다.
// 시그니처: fn is_none_or(self, f: impl FnOnce(T) -> bool) -> bool  (None이면 클로저 없이 true)
fn valid_coupon(code: Option<&str>) -> bool {
    code.is_none_or(|c| c.chars().count() == 6)
}

// filter: 조건을 만족하는 Some만 남기고 나머지는 None으로 바꿉니다. 결과는 여전히 Option입니다.
// 시그니처: fn filter<P>(self, predicate: P) -> Option<T> where P: FnOnce(&T) -> bool
// 클로저가 &T를 받는 이유: 조건이 참이면 그 값을 그대로 돌려줘야 하므로 들여다보기만 합니다(반복자의 filter와 같음).
fn parse_percent(s: &str) -> Option<u8> {
    s.trim().parse::<u8>().ok().filter(|p| *p <= 100)
}

// Part 0 ch9의 port_of를 콤비네이터로 다시 썼습니다(그때는 ?와 if로 썼습니다).
// split_once는 Option<(&str, &str)>를 돌려주고, filter의 클로저는 그 튜플의 참조를 받아 패턴으로 꺼냅니다.
fn port_of(line: &str) -> Option<u16> {
    line.split_once('=')
        .filter(|(key, _)| key.trim() == "port")
        .and_then(|(_, value)| value.trim().parse().ok())
}

fn main() {
    println!("== 1. is_some_and ==");
    for age in [Some(25), Some(12), None] {
        println!(
            "{age:?} → map_or: {}, is_some_and: {}",
            is_adult_map_or(age),
            is_adult(age)
        );
    }
    // 출력: Some(25) → map_or: true, is_some_and: true
    // 출력: Some(12) → map_or: false, is_some_and: false
    // 출력: None → map_or: false, is_some_and: false

    println!("\n== 2. is_none_or ==");
    for code in [None, Some("SPRING"), Some("A1")] {
        println!("{code:?} → {}", valid_coupon(code));
    }
    // 출력: None → true
    // 출력: Some("SPRING") → true
    // 출력: Some("A1") → false

    println!("\n== 3. is_ok_and, is_err_and ==");
    // Result판도 같습니다. is_err_and는 에러 값을 클로저에 넘기므로 "어떤 에러였는가"를 물을 수 있습니다.
    // ParseIntError의 kind()는 실패 이유를 IntErrorKind 열거형의 참조로 돌려줍니다.
    for input in ["42", "7", "", "4x"] {
        let even = input.parse::<u8>().is_ok_and(|n| n % 2 == 0);
        let empty = input
            .parse::<u8>()
            .is_err_and(|e| *e.kind() == IntErrorKind::Empty);
        println!("{input:?} → 짝수인가: {even}, 빈 입력인가: {empty}");
    }
    // 출력: "42" → 짝수인가: true, 빈 입력인가: false
    // 출력: "7" → 짝수인가: false, 빈 입력인가: false
    // 출력: "" → 짝수인가: false, 빈 입력인가: true
    // 출력: "4x" → 짝수인가: false, 빈 입력인가: false
    // is_ok_and와 is_err_and는 self를 받아 Result를 소비하므로, 여기서는 질문마다 parse를 새로 불렀습니다.
    // 결과 하나에 여러 번 물으려면 5번 구역처럼 참조로 바꾼 뒤 묻습니다.

    println!("\n== 4. filter ==");
    for input in ["80", "120", "백"] {
        println!("{input:?} → {:?}", parse_percent(input));
    }
    // 출력: "80" → Some(80)
    // 출력: "120" → None
    // 출력: "백" → None

    for line in [
        "port=8080",
        "port = 443",
        "host=localhost",
        "port=abc",
        "# 주석",
    ] {
        println!("{line:?} → {:?}", port_of(line));
    }
    // 출력: "port=8080" → Some(8080)
    // 출력: "port = 443" → Some(443)
    // 출력: "host=localhost" → None
    // 출력: "port=abc" → None
    // 출력: "# 주석" → None
    // Part 0 ch9 06_question_mark_option 예제의 port_of와 같은 결과입니다.

    println!("\n== 5. 조건 메서드도 self를 소비합니다 ==");
    // Option<String>에 is_some_and를 그대로 부르면 nickname이 이동해 이후에 쓸 수 없습니다.
    // as_deref()로 Option<&str>을 만들어 물으면 원본은 그대로 남습니다(05 예제).
    let nickname: Option<String> = Some(String::from("페리스"));
    let starts_with_pe = nickname.as_deref().is_some_and(|n| n.starts_with('페'));
    println!("'페'로 시작하는가: {starts_with_pe}, 원본: {nickname:?}");
    // 출력: '페'로 시작하는가: true, 원본: Some("페리스")
}
