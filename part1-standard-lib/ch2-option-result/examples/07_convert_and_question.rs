// 이 예제가 보여 주는 것: Option과 Result 사이의 변환(ok_or, ok_or_else, ok, err, transpose, flatten)과
//                        ?와 콤비네이터의 관계(같은 함수를 match, 콤비네이터, ?로 쓰기, ok_or(..)?와 map_err(..)?)
// 실행: cargo run -p p1-ch2-option-result --example 07_convert_and_question
//
// ?는 Option과 Result를 저절로 바꿔 주지 않습니다(Part 0 ch9). 함수의 반환 타입에 맞게 먼저 바꾼 뒤 ?를 붙입니다.
//   Result를 돌려주는 함수 안에서 Option을 만나면 → .ok_or(에러)? 또는 .ok_or_else(|| 에러)?
//   Option을 돌려주는 함수 안에서 Result를 만나면 → .ok()?
//   에러 타입이 함수의 에러 타입과 다르면          → .map_err(바꾸는 함수)?

use std::num::ParseIntError;

// 설정 텍스트("키=값" 줄들)에서 key의 값을 찾습니다. 없으면 None.
// 돌려주는 &str은 text 안을 가리키므로 text와 같은 라이프타임 'a를 붙였습니다(Part 0 ch10).
fn find_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() == key {
            return Some(v.trim());
        }
    }
    None
}

// 에러 메시지를 만드는 함수. 불릴 때 한 줄을 출력해서 언제 실행되는지 보이게 합니다.
fn missing(key: &str) -> String {
    println!("    (에러 메시지를 만듦: {key})");
    format!("{key} 설정이 없습니다")
}

// 이 예제의 에러 타입. 설계 방법은 ch8-errors-panic에서 다루고, 여기서는 derive(Debug)만 붙입니다.
#[derive(Debug)]
enum ConfigError {
    Missing(&'static str),
    BadNumber(&'static str, ParseIntError),
}

struct Config {
    port: u16,
    timeout: u32,
}

// 같은 일(두 문자열을 정수로 읽어 곱하기)을 세 가지로 씁니다.
// (1) match: Part 0 ch9 1단계의 모양입니다. 단계마다 "Err면 그대로 돌려준다"를 직접 적습니다.
fn multiply_match(a: &str, b: &str) -> Result<i32, ParseIntError> {
    match a.parse::<i32>() {
        Ok(x) => match b.parse::<i32>() {
            Ok(y) => Ok(x * y),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

// (2) 콤비네이터: 바깥 and_then의 클로저 안에서 x를 쓰려면 둘째 단계가 그 안으로 들어가야 해서 한 겹 중첩됩니다.
fn multiply_combinators(a: &str, b: &str) -> Result<i32, ParseIntError> {
    a.parse::<i32>()
        .and_then(|x| b.parse::<i32>().map(|y| x * y))
}

// (3) ?: 값마다 이름을 붙여 위에서 아래로 씁니다. 여러 값을 함께 써야 할 때는 이쪽이 읽기 쉽습니다.
fn multiply_question(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x = a.parse::<i32>()?;
    let y = b.parse::<i32>()?;
    Ok(x * y)
}

// 필수 설정: 없으면 Missing, 숫자가 아니면 BadNumber.
// - find_value는 Option을 돌려주므로 ok_or로 Result로 바꾼 뒤 ?를 붙입니다.
// - parse의 에러는 ParseIntError라서 이 함수의 에러 타입(ConfigError)과 다릅니다. map_err로 감싼 뒤 ?를 붙입니다.
//   (?는 From 트레이트로 정의된 변환이 있으면 에러를 저절로 바꿔 줍니다. ConfigError에는 그런 변환을 정의하지
//    않았으므로 map_err로 직접 바꿉니다. From은 ch6-core-traits, 에러 변환 설계는 ch8-errors-panic에서 다룹니다.)
fn required_port(text: &str) -> Result<u16, ConfigError> {
    let raw = find_value(text, "port").ok_or(ConfigError::Missing("port"))?;
    let port = raw
        .parse::<u16>()
        .map_err(|e| ConfigError::BadNumber("port", e))?;
    Ok(port)
}

// 선택 설정: 없으면 Ok(None), 있으면 숫자여야 합니다.
// map까지 하면 Option<Result<u32, ConfigError>>이고, transpose가 이것을 Result<Option<u32>, ConfigError>로 뒤집습니다.
//   None          → Ok(None)
//   Some(Ok(v))   → Ok(Some(v))
//   Some(Err(e))  → Err(e)
fn optional_timeout(text: &str) -> Result<Option<u32>, ConfigError> {
    find_value(text, "timeout")
        .map(|raw| {
            raw.parse::<u32>()
                .map_err(|e| ConfigError::BadNumber("timeout", e))
        })
        .transpose()
}

// 앞의 두 함수를 ?로 잇습니다. transpose 덕분에 "에러면 전파, 없으면 기본값"을 한 줄에 씁니다.
fn load(text: &str) -> Result<Config, ConfigError> {
    let port = required_port(text)?;
    let timeout = optional_timeout(text)?.unwrap_or(30);
    Ok(Config { port, timeout })
}

fn describe(result: &Result<Config, ConfigError>) -> String {
    match result {
        Ok(config) => format!("port={}, timeout={}", config.port, config.timeout),
        Err(ConfigError::Missing(key)) => format!("{key} 설정이 없습니다"),
        Err(ConfigError::BadNumber(key, e)) => format!("{key} 값이 숫자가 아닙니다: {e}"),
    }
}

fn main() {
    let text = "port = 8080\nname = demo";

    println!("== 1. Option → Result: ok_or, ok_or_else ==");
    // 시그니처: fn ok_or<E>(self, err: E) -> Result<T, E>
    //          fn ok_or_else<E, F>(self, err: F) -> Result<T, E> where F: FnOnce() -> E
    for key in ["port", "host"] {
        println!("키 {key}");
        let eager: Result<&str, String> = find_value(text, key).ok_or(missing(key));
        println!("  ok_or      → {eager:?}");
        let lazy: Result<&str, String> = find_value(text, key).ok_or_else(|| missing(key));
        println!("  ok_or_else → {lazy:?}");
    }
    // 출력: 키 port
    // 출력:     (에러 메시지를 만듦: port)
    // 출력:   ok_or      → Ok("8080")
    // 출력:   ok_or_else → Ok("8080")
    // 출력: 키 host
    // 출력:     (에러 메시지를 만듦: host)
    // 출력:   ok_or      → Err("host 설정이 없습니다")
    // 출력:     (에러 메시지를 만듦: host)
    // 출력:   ok_or_else → Err("host 설정이 없습니다")
    // ok_or는 값을 찾았는데도(키 port) 에러 메시지 String을 만들었다가 버립니다. 03 예제와 같은 규칙입니다.

    println!("\n== 2. Result → Option: ok, err ==");
    // ok(): Ok(v) → Some(v), Err(_) → None   (에러를 버림)
    // err(): Ok(_) → None,   Err(e) → Some(e) (성공 값을 버림)
    for input in ["42", "사십이"] {
        let ok = input.parse::<i32>().ok();
        let err = input.parse::<i32>().err();
        println!("{input:?} → ok(): {ok:?}, err(): {err:?}");
    }
    // 출력: "42" → ok(): Some(42), err(): None
    // 출력: "사십이" → ok(): None, err(): Some(ParseIntError { kind: InvalidDigit })

    println!("\n== 3. transpose: Option<Result>와 Result<Option> 뒤집기 ==");
    for text in ["timeout=5", "port=1", "timeout=abc"] {
        println!("{text:?} → {:?}", optional_timeout(text));
    }
    // 출력: "timeout=5" → Ok(Some(5))
    // 출력: "port=1" → Ok(None)
    // 출력: "timeout=abc" → Err(BadNumber("timeout", ParseIntError { kind: InvalidDigit }))
    // 반대 방향(Result<Option<T>, E> → Option<Result<T, E>>)도 transpose라는 같은 이름입니다.
    let back: Option<Result<u32, ConfigError>> = optional_timeout("timeout=5").transpose();
    println!("되돌리기: {back:?}");
    // 출력: 되돌리기: Some(Ok(5))

    println!("\n== 4. flatten: 두 겹을 한 겹으로 ==");
    // Result의 flatten은 Rust 1.89.0에서 안정화되었습니다. map에 실패할 수 있는 클로저를 넘기면 두 겹이 됩니다.
    for (a, b) in [("6", "7"), ("6", "칠")] {
        let nested: Result<Result<i32, ParseIntError>, ParseIntError> =
            a.parse::<i32>().map(|x| b.parse::<i32>().map(|y| x * y));
        println!(
            "{a} × {b}: map → {nested:?}, flatten → {:?}",
            nested.clone().flatten()
        );
    }
    // 출력: 6 × 7: map → Ok(Ok(42)), flatten → Ok(42)
    // 출력: 6 × 칠: map → Ok(Err(ParseIntError { kind: InvalidDigit })), flatten → Err(ParseIntError { kind: InvalidDigit })
    // 바깥 map을 and_then으로 바꾸면 처음부터 한 겹입니다. 그것이 아래 multiply_combinators입니다.

    println!("\n== 5. 같은 함수를 match, 콤비네이터, ?로 ==");
    for (a, b) in [("6", "7"), ("육", "7"), ("6", "")] {
        println!(
            "{a:?} × {b:?}: match {:?} / 콤비네이터 {:?} / ? {:?}",
            multiply_match(a, b),
            multiply_combinators(a, b),
            multiply_question(a, b)
        );
    }
    // 출력: "6" × "7": match Ok(42) / 콤비네이터 Ok(42) / ? Ok(42)
    // 출력: "육" × "7": match Err(ParseIntError { kind: InvalidDigit }) / 콤비네이터 Err(ParseIntError { kind: InvalidDigit }) / ? Err(ParseIntError { kind: InvalidDigit })
    // 출력: "6" × "": match Err(ParseIntError { kind: Empty }) / 콤비네이터 Err(ParseIntError { kind: Empty }) / ? Err(ParseIntError { kind: Empty })

    println!("\n== 6. ?로 잇기: ok_or(..)?, map_err(..)?, transpose()? ==");
    for text in [
        "port=8080\ntimeout=5",
        "port=8080",
        "timeout=5",
        "port=80a",
        "port=8080\ntimeout=-1",
    ] {
        println!("{text:?} → {}", describe(&load(text)));
    }
    // 출력: "port=8080\ntimeout=5" → port=8080, timeout=5
    // 출력: "port=8080" → port=8080, timeout=30
    // 출력: "timeout=5" → port 설정이 없습니다
    // 출력: "port=80a" → port 값이 숫자가 아닙니다: invalid digit found in string
    // 출력: "port=8080\ntimeout=-1" → timeout 값이 숫자가 아닙니다: invalid digit found in string
    // timeout은 u32라서 음수 부호도 잘못된 글자(invalid digit)로 봅니다.
}
