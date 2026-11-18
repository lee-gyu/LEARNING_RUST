// 기대하는 에러: error[E0308]: mismatched types (두 개)
// 핵심 메시지: expected `ParseIntError`, found `&str`
//             expected `Result<i32, &str>`, found `Result<i32, ParseIntError>`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0308_and_then_error_type.rs --out-dir /tmp/rust-p1-ch2
//
// Result::and_then의 시그니처는 fn and_then<U, F>(self, op: F) -> Result<U, E> where F: FnOnce(T) -> Result<U, E>입니다.
// 클로저가 돌려주는 Result의 에러 타입이 앞 단계와 같은 E여야 합니다. 앞 단계(parse)의 E는 ParseIntError인데,
// 클로저는 Err("...")로 &str 에러를 돌려주므로 첫 번째 에러가 납니다. 그 결과 체인 전체의 타입도
// Result<i32, ParseIntError>로 남아 함수의 반환 타입 Result<i32, &'static str>와 맞지 않는다는 두 번째 에러가 납니다.
// 고치는 방법: and_then 앞에서 map_err로 에러 타입을 먼저 맞춥니다.
//   s.trim().parse::<i32>()
//       .map_err(|_| "숫자가 아닙니다")
//       .and_then(|n| if n > 0 { Ok(n) } else { Err("양수가 아닙니다") })
// 두 에러를 모두 담을 열거형을 만들어 map_err(열거형::배리언트)로 감싸는 방법은 examples/02_and_then.rs에 있습니다.

fn parse_positive(s: &str) -> Result<i32, &'static str> {
    s.trim()
        .parse::<i32>()
        .and_then(|n| if n > 0 { Ok(n) } else { Err("양수가 아닙니다") })
}

fn main() {
    println!("{:?}", parse_positive("42"));
}
