// 지침 2: `Default`와 구조체 갱신 문법(`..Default::default()`)은 모든 필드에 뜻이 있는
// 기본값이 있는 타입에만 씁니다. 기본값이 없는 필수 값은 따로 받습니다.
//
// 기본값이 없는 필드(URL)까지 들어 있는 타입에 `#[derive(Default)]`를 붙이면, URL을 빠뜨린
// 리터럴도 컴파일되고 빈 문자열로 만들어집니다. derive가 고른 제한 시간 0도 뜻이 없습니다.
// 필수 값은 생성자 인자로 받고, 나머지만 모은 옵션 구조체에 뜻이 있는 기본값을 손으로
// 구현하면 `..Default::default()`를 안심하고 쓸 수 있습니다. 3번 구역은 그래도 남는
// 한계(필드 사이의 규칙)를 보여 줍니다.
//
// 실행: cargo run -p er-ch7-builders --example 02_default_update

use std::time::Duration;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Method {
    #[default] // 열거형은 기본 배리언트를 골라야 Default를 derive할 수 있습니다.
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

// ---- 비권장: 필수 값까지 한 타입에 넣고 Default를 derive ----

/// 필드 타입이 모두 `Default`라서 derive는 성공합니다. 하지만 `url`의 기본값(빈 문자열)과
/// `timeout`의 기본값(0)은 "값을 정하지 않았다"는 뜻일 뿐, 쓸 수 있는 값이 아닙니다.
#[derive(Default)]
struct DefaultedRequest {
    method: Method,
    url: String,
    headers: Vec<String>,
    body: Option<String>,
    timeout: Duration,
    connect_timeout: Duration,
    max_redirects: u8,
    gzip: bool,
}

impl DefaultedRequest {
    fn describe(&self) -> String {
        let o = Options {
            headers: self.headers.clone(),
            body: self.body.clone(),
            timeout: self.timeout,
            connect_timeout: self.connect_timeout,
            max_redirects: self.max_redirects,
            gzip: self.gzip,
        };
        describe(self.method, &self.url, &o)
    }
}

// ---- 권장: 필수 값은 인자로, 선택 값은 옵션 구조체로 ----

/// 모든 필드에 뜻이 있는 기본값이 있는 타입만 모았습니다.
struct Options {
    headers: Vec<String>,
    body: Option<String>,
    timeout: Duration,
    connect_timeout: Duration,
    max_redirects: u8,
    gzip: bool,
}

/// derive가 주는 0이 아니라 실제로 쓸 기본값을 손으로 구현합니다(Part 1 ch6 2.12절).
impl Default for Options {
    fn default() -> Self {
        Options {
            headers: Vec::new(),
            body: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(5),
            max_redirects: 5,
            gzip: true,
        }
    }
}

struct Request {
    method: Method,
    url: String,
    options: Options,
}

impl Request {
    /// 인자는 셋이고 타입이 모두 달라서 순서를 바꾸면 컴파일되지 않습니다.
    fn new(method: Method, url: &str, options: Options) -> Request {
        Request {
            method,
            url: url.to_string(),
            options,
        }
    }

    fn describe(&self) -> String {
        describe(self.method, &self.url, &self.options)
    }
}

fn describe(method: Method, url: &str, o: &Options) -> String {
    let url = if url.is_empty() { "(URL 없음)" } else { url };
    let body = match &o.body {
        Some(b) => format!("본문 {}바이트", b.len()),
        None => "본문 없음".to_string(),
    };
    format!(
        "{} {url} | 헤더 {} | {body} | 전체 {:?}, 연결 {:?} | 리다이렉트 {} | gzip {}",
        method.as_str(),
        o.headers.len(),
        o.timeout,
        o.connect_timeout,
        o.max_redirects,
        if o.gzip { "켬" } else { "끔" },
    )
}

fn main() {
    let url = "https://api.example.com/items";
    let json = r#"{"name":"연필"}"#;

    println!("1. 비권장: 필수 값까지 Default를 derive한 타입");
    // URL을 깜빡했습니다. 컴파일러는 `..`가 나머지를 채웠다고 보고 아무 말도 하지 않습니다.
    let forgot_url = DefaultedRequest {
        method: Method::Post,
        body: Some(json.to_string()),
        ..Default::default()
    };
    println!("    {}", forgot_url.describe());
    // 출력:     POST (URL 없음) | 헤더 0 | 본문 17바이트 | 전체 0ns, 연결 0ns | 리다이렉트 0 | gzip 끔

    println!("2. 권장: 필수 값은 인자로, 나머지는 뜻이 있는 기본값을 가진 옵션 구조체로");
    // 예제 01의 두 요청입니다. 기본값과 다른 필드만 적으면 됩니다.
    let list = Request::new(Method::Get, url, Options::default());
    let create = Request::new(
        Method::Post,
        url,
        Options {
            headers: vec!["Content-Type: application/json".to_string()],
            body: Some(json.to_string()),
            ..Default::default()
        },
    );
    println!("    {}", list.describe());
    // 출력:     GET https://api.example.com/items | 헤더 0 | 본문 없음 | 전체 30s, 연결 5s | 리다이렉트 5 | gzip 켬
    println!("    {}", create.describe());
    // 출력:     POST https://api.example.com/items | 헤더 1 | 본문 17바이트 | 전체 30s, 연결 5s | 리다이렉트 5 | gzip 켬

    println!("3. 남는 한계: 필드 사이의 규칙을 검사할 곳이 없습니다");
    // 필드가 모두 공개라서 부르는 쪽이 어떤 조합이든 만들 수 있습니다. `Options`에 비공개
    // 필드를 두어 막으려 하면 이번에는 `..Default::default()` 자체가 E0451로 막힙니다
    // (compile_fail/e0451_default_update_private_field.rs).
    let slow_connect = Request::new(
        Method::Get,
        url,
        Options {
            timeout: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(60),
            ..Default::default()
        },
    );
    println!(
        "    연결 60s > 전체 10s도 만들어짐: {}",
        slow_connect.describe()
    );
    // 출력:     연결 60s > 전체 10s도 만들어짐: GET https://api.example.com/items | 헤더 0 | 본문 없음 | 전체 10s, 연결 60s | 리다이렉트 5 | gzip 켬
    let get_with_body = Request::new(
        Method::Get,
        url,
        Options {
            body: Some(json.to_string()),
            ..Default::default()
        },
    );
    println!(
        "    본문이 있는 GET도 만들어짐: {}",
        get_with_body.describe()
    );
    // 출력:     본문이 있는 GET도 만들어짐: GET https://api.example.com/items | 헤더 0 | 본문 17바이트 | 전체 30s, 연결 5s | 리다이렉트 5 | gzip 켬
}
