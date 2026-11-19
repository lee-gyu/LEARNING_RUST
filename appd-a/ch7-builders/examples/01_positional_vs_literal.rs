// 지침 1: 필드가 많은 타입을 위치 인자 생성자로 만들지 않습니다. 필드 이름이 보이는
// 구조체 리터럴로 시작합니다.
//
// 인자가 여덟 개인 `new`는 같은 타입의 인자(`Duration` 두 개)를 바꿔 넣어도 컴파일되고,
// 틀린 값으로 조용히 만들어집니다. 구조체 리터럴은 `timeout: ..`처럼 이름을 적어야 하므로
// 그런 실수가 코드에 드러납니다. 대신 부르는 곳마다 모든 필드를 적어야 하는데, 3번 구역이
// 그 비용을 보여 줍니다. 이 비용이 예제 02의 출발점입니다.
//
// 실행: cargo run -p er-ch7-builders --example 01_positional_vs_literal

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    Get,
    Post,
}

impl Method {
    fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
        }
    }
}

/// HTTP 요청 하나를 설명하는 값입니다. 이 챕터의 예제는 요청을 실제로 보내지 않고
/// 내용만 출력합니다.
struct Request {
    method: Method,
    url: String,
    headers: Vec<String>,
    body: Option<String>,
    timeout: Duration,         // 요청 전체의 제한 시간
    connect_timeout: Duration, // 연결을 맺기까지의 제한 시간. 전체보다 길면 뜻이 없습니다.
    max_redirects: u8,
    gzip: bool,
}

impl Request {
    /// 비권장: 필드마다 위치 인자 하나를 받는 생성자입니다.
    /// 부르는 쪽 코드에는 값만 남고 이름이 사라지므로, 타입이 같은 인자끼리 순서가
    /// 바뀌어도 컴파일러가 알아챌 방법이 없습니다.
    #[expect(
        clippy::too_many_arguments,
        reason = "인자가 너무 많은 생성자가 이 예제의 비권장 코드입니다"
    )]
    fn new(
        method: Method,
        url: &str,
        headers: Vec<String>,
        body: Option<String>,
        timeout: Duration,
        connect_timeout: Duration,
        max_redirects: u8,
        gzip: bool,
    ) -> Request {
        Request {
            method,
            url: url.to_string(),
            headers,
            body,
            timeout,
            connect_timeout,
            max_redirects,
            gzip,
        }
    }

    fn describe(&self) -> String {
        let body = match &self.body {
            Some(b) => format!("본문 {}바이트", b.len()),
            None => "본문 없음".to_string(),
        };
        format!(
            "{} {} | 헤더 {} | {body} | 전체 {:?}, 연결 {:?} | 리다이렉트 {} | gzip {}",
            self.method.as_str(),
            self.url,
            self.headers.len(),
            self.timeout,
            self.connect_timeout,
            self.max_redirects,
            if self.gzip { "켬" } else { "끔" },
        )
    }
}

/// 두 요청에서 값이 똑같은 필드의 이름을 모읍니다. 3번 구역에서 "두 리터럴이 같은 값을
/// 몇 번 되풀이해 적었는가"를 세는 데 씁니다.
fn same_fields(a: &Request, b: &Request) -> Vec<&'static str> {
    let checks = [
        ("method", a.method == b.method),
        ("url", a.url == b.url),
        ("headers", a.headers == b.headers),
        ("body", a.body == b.body),
        ("timeout", a.timeout == b.timeout),
        ("connect_timeout", a.connect_timeout == b.connect_timeout),
        ("max_redirects", a.max_redirects == b.max_redirects),
        ("gzip", a.gzip == b.gzip),
    ];
    checks
        .iter()
        .filter(|(_, same)| *same)
        .map(|(name, _)| *name)
        .collect()
}

fn main() {
    let url = "https://api.example.com/items";
    println!("의도: 전체 제한 시간 30s, 연결 제한 시간 3s");
    // 출력: 의도: 전체 제한 시간 30s, 연결 제한 시간 3s

    println!("1. 비권장: 인자 여덟 개짜리 new");
    // 다섯째와 여섯째 인자의 순서를 헷갈렸습니다. 둘 다 Duration이라 컴파일됩니다.
    let swapped = Request::new(
        Method::Get,
        url,
        Vec::new(),
        None,
        Duration::from_secs(3),
        Duration::from_secs(30),
        5,
        true,
    );
    println!("    {}", swapped.describe());
    // 출력:     GET https://api.example.com/items | 헤더 0 | 본문 없음 | 전체 3s, 연결 30s | 리다이렉트 5 | gzip 켬

    println!("2. 권장: 필드 이름을 적는 구조체 리터럴");
    // 이름으로 짝을 맞추므로 순서는 상관없고, 값이 어느 필드로 가는지 읽는 사람에게 보입니다.
    let list = Request {
        method: Method::Get,
        url: url.to_string(),
        headers: Vec::new(),
        body: None,
        timeout: Duration::from_secs(30),
        connect_timeout: Duration::from_secs(3),
        max_redirects: 5,
        gzip: true,
    };
    println!("    {}", list.describe());
    // 출력:     GET https://api.example.com/items | 헤더 0 | 본문 없음 | 전체 30s, 연결 3s | 리다이렉트 5 | gzip 켬

    println!("3. 구조체 리터럴의 비용: 부르는 곳마다 여덟 필드를 모두 적습니다");
    // 같은 서버에 보내는 두 번째 요청입니다. 달라진 것은 메서드, 헤더, 본문뿐인데도
    // 나머지 다섯 필드를 다시 적어야 합니다. 필드가 하나 늘면 이런 곳이 모두 E0063으로
    // 컴파일되지 않습니다(Part 0 ch5).
    let create = Request {
        method: Method::Post,
        url: url.to_string(),
        headers: vec!["Content-Type: application/json".to_string()],
        body: Some(r#"{"name":"연필"}"#.to_string()),
        timeout: Duration::from_secs(30),
        connect_timeout: Duration::from_secs(3),
        max_redirects: 5,
        gzip: true,
    };
    println!("    {}", create.describe());
    // 출력:     POST https://api.example.com/items | 헤더 1 | 본문 17바이트 | 전체 30s, 연결 3s | 리다이렉트 5 | gzip 켬
    let repeated = same_fields(&list, &create);
    println!(
        "    두 리터럴이 같은 값을 되풀이한 필드 {}개: {}",
        repeated.len(),
        repeated.join(", ")
    );
    // 출력:     두 리터럴이 같은 값을 되풀이한 필드 5개: url, timeout, connect_timeout, max_redirects, gzip
}
