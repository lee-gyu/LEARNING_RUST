// 이 예제가 보여 주는 것: 실패할 수 있는 단계 잇기 — and_then, map과 and_then의 차이, 단락 평가,
//                        에러 타입이 다른 단계를 map_err로 맞춰 잇기
// 실행: cargo run -p p1-ch2-option-result --example 02_and_then
//
// map은 "절대 실패하지 않는 변환"을, and_then은 "그 자체도 실패할 수 있는 다음 단계"를 붙입니다.
// 두 메서드의 차이는 클로저가 돌려주는 타입 하나뿐입니다.
//   map:      F: FnOnce(T) -> U          → 결과 Option<U>  (map이 Some으로 싸 줌)
//   and_then: F: FnOnce(T) -> Option<U>  → 결과 Option<U>  (클로저가 돌려준 Option을 그대로 씀)

use std::num::ParseIntError;

// 텍스트 첫 줄의 첫 단어를 정수로 읽습니다. 세 단계 모두 실패할 수 있습니다.
// 1) 첫 줄이 없을 수 있음  2) 그 줄에 단어가 없을 수 있음  3) 단어가 숫자가 아닐 수 있음
//
// "전" 버전: Part 0에서 배운 match만으로 쓰면 단계마다 한 겹씩 깊어집니다.
// 마지막 match(Ok면 Some, Err면 None)는 Result::ok()와 같아서 clippy가 ok()를 쓰라고 제안합니다(clippy::manual_ok_err).
#[expect(clippy::manual_ok_err)]
fn first_number_match(text: &str) -> Option<i32> {
    match text.lines().next() {
        Some(line) => match line.split_whitespace().next() {
            Some(word) => match word.parse::<i32>() {
                Ok(n) => Some(n),
                Err(_) => None,
            },
            None => None,
        },
        None => None,
    }
}

// "후" 버전: 실패할 수 있는 단계를 and_then으로 위에서 아래로 잇습니다.
// 어느 단계에서든 None이 나오면 그 뒤의 클로저는 부르지 않고 None이 끝까지 흘러갑니다.
fn first_number(text: &str) -> Option<i32> {
    text.lines()
        .next()
        .and_then(|line| line.split_whitespace().next())
        .and_then(|word| word.parse::<i32>().ok()) // ok(): Result → Option (07 예제)
}

// map을 잘못 고르면: 클로저가 이미 Option을 돌려주는데 map이 그것을 Some으로 한 번 더 쌉니다.
// flatten()은 Option<Option<T>>를 Option<T>로 한 겹 벗깁니다. 그래서 and_then = map + flatten입니다.
// 중간 결과를 보려고 변수에 담았지만, .map(..).flatten()으로 이어 쓰면 clippy가 and_then을 쓰라고
// 제안합니다(clippy::map_flatten).
fn first_word_map_flatten(text: &str) -> Option<&str> {
    let nested: Option<Option<&str>> = text
        .lines()
        .next()
        .map(|line| line.split_whitespace().next());
    println!("  map만 쓴 중간 결과: {nested:?}");
    nested.flatten()
}

// 나이를 읽는 두 단계. 두 단계의 에러 종류가 다르므로, 둘 다 담을 수 있는 열거형을 만듭니다.
// (에러 타입을 제대로 설계하는 방법은 Part 1 ch8-errors-panic에서 다룹니다. 여기서는 derive(Debug)만 붙입니다.)
#[derive(Debug)]
enum AgeError {
    NotNumber(ParseIntError),
    OutOfRange(i32),
}

fn check_range(n: i32) -> Result<i32, AgeError> {
    if (0..=150).contains(&n) {
        Ok(n)
    } else {
        Err(AgeError::OutOfRange(n))
    }
}

// Result의 and_then 시그니처: fn and_then<U, F>(self, op: F) -> Result<U, E> where F: FnOnce(T) -> Result<U, E>
// 앞 단계와 다음 단계의 에러 타입 E가 같아야 합니다. parse의 에러는 ParseIntError,
// check_range의 에러는 AgeError이므로 그대로는 이을 수 없습니다(compile_fail/e0308_and_then_error_type.rs).
// map_err로 먼저 ParseIntError를 AgeError로 감싸 E를 맞춘 뒤 and_then으로 잇습니다.
// AgeError::NotNumber는 ParseIntError를 받아 AgeError를 돌려주는 함수처럼 쓸 수 있고(Part 0 ch19),
// check_range도 함수 이름 그대로 넘깁니다.
fn parse_age(input: &str) -> Result<i32, AgeError> {
    input
        .trim()
        .parse::<i32>()
        .map_err(AgeError::NotNumber)
        .and_then(check_range)
}

// derive(Debug)로 출력만 하면 필드를 읽지 않은 것으로 쳐서 dead_code 경고가 나므로, match로 직접 꺼내 씁니다.
fn describe(result: &Result<i32, AgeError>) -> String {
    match result {
        Ok(age) => format!("{age}살"),
        Err(AgeError::NotNumber(e)) => format!("숫자가 아님 ({e})"),
        Err(AgeError::OutOfRange(n)) => format!("범위 밖 ({n})"),
    }
}

fn main() {
    println!("== 1. match 세 겹과 and_then 체인 ==");
    for text in ["42 개\n둘째 줄", "", "   \n7", "사십이"] {
        println!(
            "{text:?} → match: {:?}, and_then: {:?}",
            first_number_match(text),
            first_number(text)
        );
    }
    // 출력: "42 개\n둘째 줄" → match: Some(42), and_then: Some(42)
    // 출력: "" → match: None, and_then: None
    // 출력: "   \n7" → match: None, and_then: None
    // 출력: "사십이" → match: None, and_then: None
    // 둘째 입력은 첫 줄이 없어서, 셋째는 첫 줄에 단어가 없어서, 넷째는 숫자가 아니어서 None입니다.

    println!("\n== 2. map을 쓰면 Option이 두 겹이 됩니다 ==");
    let first_word = first_word_map_flatten("안녕 세상");
    println!("flatten 후: {first_word:?}");
    let first_word = first_word_map_flatten("");
    println!("flatten 후: {first_word:?}");
    // 출력:   map만 쓴 중간 결과: Some(Some("안녕"))
    // 출력: flatten 후: Some("안녕")
    // 출력:   map만 쓴 중간 결과: None
    // 출력: flatten 후: None

    println!("\n== 3. 단락 평가: 실패한 뒤의 단계는 실행되지 않습니다 ==");
    // checked_mul은 곱셈이 넘치면 None을 돌려줍니다(Part 0 ch3에서 본 checked_* 메서드).
    for input in ["3", "5000", "x"] {
        println!("입력 {input:?}");
        let result = input
            .parse::<i32>()
            .ok()
            .and_then(|n| {
                println!("  2단계 실행: {n} × 1000");
                n.checked_mul(1000)
            })
            .and_then(|n| {
                println!("  3단계 실행: {n} × 1000");
                n.checked_mul(1000)
            });
        println!("  결과: {result:?}");
    }
    // 출력: 입력 "3"
    // 출력:   2단계 실행: 3 × 1000
    // 출력:   3단계 실행: 3000 × 1000
    // 출력:   결과: Some(3000000)
    // 출력: 입력 "5000"
    // 출력:   2단계 실행: 5000 × 1000
    // 출력:   3단계 실행: 5000000 × 1000
    // 출력:   결과: None
    // 출력: 입력 "x"
    // 출력:   결과: None
    // "5000"은 3단계에서 i32 범위를 넘어 None, "x"는 1단계에서 None이 되어 2·3단계의 클로저가 한 번도 불리지 않았습니다.

    println!("\n== 4. Result 단계 잇기: map_err로 에러 타입을 맞춘 뒤 and_then ==");
    for input in ["27", "200", "스물"] {
        let age = parse_age(input);
        println!("{input:?} → {}", describe(&age));
    }
    // 출력: "27" → 27살
    // 출력: "200" → 범위 밖 (200)
    // 출력: "스물" → 숫자가 아님 (invalid digit found in string)
}
