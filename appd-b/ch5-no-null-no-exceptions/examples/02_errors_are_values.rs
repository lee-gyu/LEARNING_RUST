// Q5.3 · Q5.4 · Q5.9 — 에러는 던지는 것이 아니라 돌려주는 값이다
//
// 불안정한 서버에 설정을 요청하는데, 처음 두 번은 시간 초과로 실패하고 세 번째에 성공한다고 해 봅니다.
// 실패가 `Result` 값이므로 변수에 담고, 종류를 비교하고, 반복문의 조건으로 쓸 수 있습니다.
//
//   1. 다시 시도하기: 에러의 종류를 보고 다시 시도할지 정함
//   2. 다시 시도할 수 없는 종류의 에러는 바로 돌려줌
//
// `match`로 전파한 판과 `?`로 전파한 판을 나란히 둔 예는 Part 0 ch9 예제 05에 있습니다. 여기서는
// 예외라면 `try`/`catch`를 반복문 안에 넣었을 일을, 에러 값에 대한 `match`로 하는 것을 봅니다.
//
// 실행: cargo run -p faq-ch5-no-null-no-exceptions --example 02_errors_are_values

#[derive(Debug)]
enum FetchError {
    Timeout,
    NotFound,
}

/// `attempt`번째 시도의 결과. 처음 두 번은 시간 초과, 그다음은 성공합니다.
fn fetch_config(key: &str, attempt: u32) -> Result<u32, FetchError> {
    match (key, attempt) {
        ("port", 1..=2) => Err(FetchError::Timeout),
        ("port", _) => Ok(8080),
        _ => Err(FetchError::NotFound),
    }
}

/// 시간 초과면 최대 `max`번까지 다시 시도하고, 다른 에러는 바로 돌려줍니다.
fn fetch_with_retry(key: &str, max: u32) -> Result<u32, FetchError> {
    let mut attempt = 1;
    loop {
        match fetch_config(key, attempt) {
            Err(FetchError::Timeout) if attempt < max => {
                println!("  {attempt}번째 시도: 시간 초과, 다시 시도");
                attempt += 1;
            }
            other => return other, // 성공했거나, 다시 시도할 수 없는 에러
        }
    }
}

fn main() {
    println!("1. 에러 값을 보고 다시 시도하기");
    let result = fetch_with_retry("port", 3);
    println!("  결과: {result:?}");
    // 출력:   1번째 시도: 시간 초과, 다시 시도
    // 출력:   2번째 시도: 시간 초과, 다시 시도
    // 출력:   결과: Ok(8080)

    println!("2. 다시 시도할 수 없는 에러는 바로 돌려줌");
    println!("  결과: {:?}", fetch_with_retry("host", 3));
    // 출력:   결과: Err(NotFound)
}
