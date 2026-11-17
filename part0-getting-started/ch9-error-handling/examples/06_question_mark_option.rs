// ? 연산자는 Option에도 쓸 수 있습니다. Some이면 안의 값을 꺼내고, None이면 함수에서 즉시 None을 돌려줍니다.
// 단, Option을 돌려주는 함수 안에서만 씁니다. Result를 돌려주는 함수 안에서는 Result에만 씁니다.
//
// 실행: cargo run -p p0-ch9-error-handling --example 06_question_mark_option

// 첫 줄의 마지막 글자를 돌려줍니다. 빈 문자열처럼 첫 줄이 없거나, 첫 줄이 비어 있으면 None입니다.
// lines()는 줄을 하나씩 내주는 반복자이고, next()는 다음 줄을 Option<&str>로 꺼냅니다(반복자는 ch13).
fn last_char_of_first_line(text: &str) -> Option<char> {
    text.lines().next()?.chars().last()
}

// ch6의 let...else로 쓴 같은 함수입니다. ?가 이 모양을 한 글자로 줄여 준다는 것을 비교하려고 남겼습니다.
// Clippy는 이 모양을 보면 ?를 쓰라고 제안하므로(clippy::question_mark) expect로 표시합니다.
#[expect(clippy::question_mark)]
fn last_char_of_first_line_let_else(text: &str) -> Option<char> {
    let Some(first_line) = text.lines().next() else {
        return None;
    };
    first_line.chars().last()
}

// 벡터의 0번과 1번 요소를 더합니다. 요소가 모자라면 None입니다.
// ch8에서 배운 get은 범위 밖이면 패닉하지 않고 None을 돌려주므로 ?와 잘 어울립니다.
// first()는 get(0)과 같은 일을 하는 메서드입니다(Clippy도 get(0) 대신 first()를 권합니다).
fn sum_first_two(numbers: &[i32]) -> Option<i32> {
    let first = numbers.first()?;
    let second = numbers.get(1)?;
    Some(first + second)
}

// ch6의 07_let_else에서 다룬 "port=8080" 설정 줄을 ?로 처리합니다.
fn port_of(line: &str) -> Option<u16> {
    let (key, value) = line.split_once('=')?;
    if key.trim() != "port" {
        return None;
    }
    // parse는 Result를 돌려주므로 Option을 돌려주는 이 함수에서 바로 ?를 쓸 수 없습니다.
    // ok()로 Result를 Option으로 바꾸면(Ok(v) → Some(v), Err(_) → None) ?를 쓸 수 있습니다.
    // 대신 "왜 실패했는지"라는 에러 정보는 버려집니다.
    let port: u16 = value.trim().parse().ok()?;
    Some(port)
}

fn main() {
    println!("1. 첫 줄의 마지막 글자");
    for text in ["안녕하세요\n반갑습니다", "", "\n둘째 줄"] {
        println!(
            "    {text:?} → ? 버전: {:?}, let...else 버전: {:?}",
            last_char_of_first_line(text),
            last_char_of_first_line_let_else(text)
        );
    }
    // 출력:
    //     "안녕하세요\n반갑습니다" → ? 버전: Some('요'), let...else 버전: Some('요')
    //     "" → ? 버전: None, let...else 버전: None
    //     "\n둘째 줄" → ? 버전: None, let...else 버전: None
    // 빈 문자열은 첫 줄이 없어서 next()에서, "\n둘째 줄"은 첫 줄이 빈 줄이라 last()에서 None이 됩니다.

    println!("2. 요소가 모자라면 첫 번째 None에서 멈춥니다");
    println!("    [10, 20, 30] → {:?}", sum_first_two(&[10, 20, 30]));
    println!("    [10]         → {:?}", sum_first_two(&[10]));
    // 출력:
    //     [10, 20, 30] → Some(30)
    //     [10]         → None

    println!("3. Result는 ok()로 Option으로 바꾼 뒤 ?를 씁니다");
    for line in [
        "port=8080",
        "port = 443",
        "host=localhost",
        "port=abc",
        "# 주석",
    ] {
        println!("    {line:?} → {:?}", port_of(line));
    }
    // 출력:
    //     "port=8080" → Some(8080)
    //     "port = 443" → Some(443)
    //     "host=localhost" → None
    //     "port=abc" → None
    //     "# 주석" → None
}
