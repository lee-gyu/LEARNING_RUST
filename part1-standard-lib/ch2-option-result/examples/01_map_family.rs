// 이 예제가 보여 주는 것: 안의 값 바꾸기 — map, map_err, map_or, map_or_else, map_or_default
// 실행: cargo run -p p1-ch2-option-result --example 01_map_family
//
// 콤비네이터(combinator)는 Option이나 Result를 받아 새 Option/Result(또는 값 하나)를 돌려주는 메서드입니다.
// match로 "Some/Ok일 때"와 "None/Err일 때"를 매번 직접 나누는 대신, 바꾸고 싶은 쪽에서 할 일만 클로저로 넘깁니다.
// 넘기지 않은 쪽(map이라면 None/Err)은 손대지 않고 그대로 통과시킵니다.

// Part 0 ch6의 plus_one을 그대로 옮겼습니다. "Some이면 안의 값을 바꾸고, None이면 None"이라는 모양은
// 너무 흔해서 clippy가 map으로 바꾸라고 제안합니다(clippy::manual_map). 비교하려고 남겨 둡니다.
#[expect(clippy::manual_map)]
fn plus_one_match(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}

// 같은 함수를 map으로 쓴 것입니다.
// 시그니처: fn map<U, F>(self, f: F) -> Option<U> where F: FnOnce(T) -> U
// - f는 안의 값 T를 받아 U를 돌려줍니다. 반환 타입이 Option<U>이므로 결과를 Some으로 다시 싸는 일은 map이 합니다.
// - FnOnce인 이유: Some일 때 한 번, None이면 아예 부르지 않기 때문입니다.
fn plus_one(x: Option<i32>) -> Option<i32> {
    x.map(|i| i + 1)
}

fn main() {
    println!("== 1. map: Some/Ok 안의 값만 바꾸기 ==");
    for x in [Some(5), None] {
        println!(
            "{x:?} → match: {:?}, map: {:?}",
            plus_one_match(x),
            plus_one(x)
        );
    }
    // 출력: Some(5) → match: Some(6), map: Some(6)
    // 출력: None → match: None, map: None

    // map의 클로저가 다른 타입을 돌려주면 결과의 안쪽 타입도 바뀝니다(T = usize → U = &str).
    // find는 찾은 바이트 위치를 Option<usize>로 돌려주고, map이 그 위치 앞부분을 잘라 Option<&str>로 만듭니다.
    let text = "key=value";
    for needle in ['=', ':'] {
        let before: Option<&str> = text.find(needle).map(|pos| &text[..pos]);
        println!("find({needle:?}) 앞부분: {before:?}");
    }
    // 출력: find('=') 앞부분: Some("key")
    // 출력: find(':') 앞부분: None

    // Result의 map은 Ok 안의 값만 바꾸고, Err는 그대로 통과시킵니다.
    // 시그니처: fn map<U, F>(self, op: F) -> Result<U, E> where F: FnOnce(T) -> U  (E는 그대로)
    for input in ["21", "스물하나"] {
        let doubled = input.parse::<i32>().map(|n| n * 2);
        println!("{input:?} → {doubled:?}");
    }
    // 출력: "21" → Ok(42)
    // 출력: "스물하나" → Err(ParseIntError { kind: InvalidDigit })

    println!("\n== 2. map_err: Err 안의 값만 바꾸기 ==");
    // 시그니처: fn map_err<F, O>(self, op: O) -> Result<T, F> where O: FnOnce(E) -> F
    // 이번에는 에러 타입이 E에서 F로 바뀌고, Ok는 손대지 않습니다. map과 정확히 반대쪽입니다.
    // 표준 라이브러리의 에러에 "무엇을 하다가 실패했는지"를 덧붙여 String 에러로 바꾸는 데 자주 씁니다.
    for input in ["8080", "80a", "70000"] {
        let port: Result<u16, String> = input
            .parse::<u16>()
            .map_err(|e| format!("포트 값 '{input}' 읽기 실패: {e}"));
        println!("{port:?}");
    }
    // 출력: Ok(8080)
    // 출력: Err("포트 값 '80a' 읽기 실패: invalid digit found in string")
    // 출력: Err("포트 값 '70000' 읽기 실패: number too large to fit in target type")
    // ParseIntError의 메시지(invalid digit ...)는 표준 라이브러리가 정한 영어 문장입니다.

    println!("\n== 3. map_or, map_or_else: 바꾼 뒤 바로 꺼내기 ==");
    // map(f).unwrap_or(기본값)을 한 번에 하는 메서드입니다. 결과는 Option이 아니라 U 값 하나입니다.
    // 시그니처: fn map_or<U, F>(self, default: U, f: F) -> U where F: FnOnce(T) -> U
    // 기본값이 첫 번째 인자, 클로저가 두 번째 인자라는 순서에 주의합니다.
    let words = ["사과", "바나나"];
    for i in [1, 5] {
        let len = words.get(i).map_or(0, |w| w.chars().count());
        println!("words.get({i})의 글자 수: {len}");
    }
    // 출력: words.get(1)의 글자 수: 3
    // 출력: words.get(5)의 글자 수: 0

    // map_or_else는 기본값도 클로저로 받습니다.
    // Option판: fn map_or_else<U, D, F>(self, default: D, f: F) -> U where D: FnOnce() -> U, F: FnOnce(T) -> U
    // Result판: D: FnOnce(E) -> U  ← 에러 값을 받으므로 Ok와 Err를 각각 다른 문장으로 바꿀 수 있습니다.
    for input in ["42", ""] {
        let message = input.parse::<i32>().map_or_else(
            |e| format!("실패({e})"), // Err일 때: 에러 값을 받습니다.
            |n| format!("성공({n})"), // Ok일 때
        );
        println!("{input:?} → {message}");
    }
    // 출력: "42" → 성공(42)
    // 출력: "" → 실패(cannot parse integer from empty string)

    println!("\n== 4. map_or_default: None/Err면 타입의 기본값 ==");
    // Rust 1.98.0에서 안정화된 메서드입니다. map_or(0, f), map_or(String::new(), f)처럼
    // "타입의 기본값"을 쓰는 경우를 줄여 씁니다. 기본값은 Default 트레이트가 정합니다(ch6-core-traits).
    // 시그니처: fn map_or_default<U, F>(self, f: F) -> U where U: Default, F: FnOnce(T) -> U
    for i in [0, 9] {
        let len: usize = words.get(i).map_or_default(|w| w.len());
        println!("words.get({i})의 바이트 수: {len}");
    }
    // 출력: words.get(0)의 바이트 수: 6
    // 출력: words.get(9)의 바이트 수: 0
}
