// 지침 3: 기본값이 없는 필수 값, 비공개 필드, 필드 사이의 규칙이 있으면 빌더를 만듭니다.
// 필수 값은 빌더 생성자의 인자로 받고, 선택 값은 메서드로 하나씩 정하고, 규칙은 `build()`가
// 한곳에서 검사해 `Result`로 알립니다.
//
// 이 예제의 빌더는 소비형입니다. 메서드가 `self`를 받아 `Self`를 돌려주므로 `builder(..)`부터
// `build()`까지 한 줄 체인으로 씁니다. 소비형과 `&mut` 빌더의 득실은 예제 04에서 비교합니다.
// 예제 01·02보다 필드를 줄였습니다.
//
// 실행: cargo run -p er-ch7-builders --example 03_consuming_builder

use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
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

// ---- 비권장: 예제 02처럼 필드를 공개하고 리터럴로 만드는 타입 ----

mod loose {
    use super::Method;
    use std::time::Duration;

    /// 필드가 모두 공개라서 누구든 아무 조합이나 만들 수 있습니다.
    pub struct Request {
        pub method: Method,
        pub url: String,
        pub body: Option<String>,
        pub timeout: Duration,
        pub connect_timeout: Duration,
    }

    impl Request {
        pub fn describe(&self) -> String {
            format!(
                "{} {} | 본문 {} | 전체 {:?}, 연결 {:?}",
                self.method.as_str(),
                self.url,
                if self.body.is_some() {
                    "있음"
                } else {
                    "없음"
                },
                self.timeout,
                self.connect_timeout,
            )
        }
    }
}

// ---- 권장: 빌더로만 만들 수 있는 타입 ----

mod http {
    use super::Method;
    use std::fmt;
    use std::time::Duration;

    /// 필드가 모두 비공개입니다. 모듈 밖에서는 구조체 리터럴로 만들 수 없으므로
    /// (E0451, Part 0 ch7) 만드는 길은 `RequestBuilder::build`뿐이고, 그래서 이 타입의
    /// 값은 모두 `build`의 검사를 통과한 값입니다(불변식).
    #[derive(Debug)]
    pub struct Request {
        method: Method,
        url: String,
        headers: Vec<String>,
        body: Option<String>,
        timeout: Duration,
        connect_timeout: Duration,
    }

    impl Request {
        /// `Request::builder(..)`로 빌더를 얻는 것은 흔한 관례입니다.
        pub fn builder(method: Method, url: impl Into<String>) -> RequestBuilder {
            RequestBuilder::new(method, url)
        }

        pub fn describe(&self) -> String {
            format!(
                "{} {} | 헤더 {:?} | 본문 {} | 전체 {:?}, 연결 {:?}",
                self.method.as_str(),
                self.url,
                self.headers,
                match &self.body {
                    Some(b) => format!("{}바이트", b.len()),
                    None => "없음".to_string(),
                },
                self.timeout,
                self.connect_timeout,
            )
        }
    }

    /// 소비형 빌더: 메서드마다 `self`를 받아 고친 뒤 `Self`로 돌려줍니다.
    pub struct RequestBuilder {
        method: Method,
        url: String,
        headers: Vec<String>,
        body: Option<String>,
        timeout: Duration,
        connect_timeout: Duration,
    }

    impl RequestBuilder {
        /// 기본값이 없는 값(메서드와 주소)은 여기서 받습니다. 빠뜨리면 컴파일되지 않습니다.
        pub fn new(method: Method, url: impl Into<String>) -> Self {
            RequestBuilder {
                method,
                url: url.into(),
                headers: Vec::new(),
                body: None,
                timeout: Duration::from_secs(30),
                connect_timeout: Duration::from_secs(5),
            }
        }

        /// 필드에 값을 넣는 것보다 많은 일도 메서드라면 할 수 있습니다. 여기서는 누적합니다.
        pub fn header(mut self, line: impl Into<String>) -> Self {
            self.headers.push(line.into());
            self
        }

        pub fn body(mut self, body: impl Into<String>) -> Self {
            self.body = Some(body.into());
            self
        }

        pub fn timeout(mut self, d: Duration) -> Self {
            self.timeout = d;
            self
        }

        pub fn connect_timeout(mut self, d: Duration) -> Self {
            self.connect_timeout = d;
            self
        }

        /// 필드 사이의 규칙은 모든 값이 정해진 이 시점에만 검사할 수 있습니다.
        /// 세터마다 검사하면 `timeout`과 `connect_timeout`을 정하는 순서에 따라 결과가 달라집니다.
        pub fn build(self) -> Result<Request, BuildError> {
            if !(self.url.starts_with("http://") || self.url.starts_with("https://")) {
                return Err(BuildError::BadScheme(self.url));
            }
            if self.method == Method::Get && self.body.is_some() {
                return Err(BuildError::BodyWithGet);
            }
            if self.connect_timeout > self.timeout {
                return Err(BuildError::ConnectLongerThanTotal {
                    connect: self.connect_timeout,
                    total: self.timeout,
                });
            }
            // 소비형이라 필드를 복제하지 않고 그대로 옮깁니다.
            Ok(Request {
                method: self.method,
                url: self.url,
                headers: self.headers,
                body: self.body,
                timeout: self.timeout,
                connect_timeout: self.connect_timeout,
            })
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum BuildError {
        BadScheme(String),
        BodyWithGet,
        ConnectLongerThanTotal { connect: Duration, total: Duration },
    }

    impl fmt::Display for BuildError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                BuildError::BadScheme(url) => {
                    write!(f, "주소는 http:// 또는 https://로 시작해야 합니다: {url:?}")
                }
                BuildError::BodyWithGet => write!(f, "GET 요청에는 본문을 넣을 수 없습니다"),
                BuildError::ConnectLongerThanTotal { connect, total } => write!(
                    f,
                    "연결 제한 시간({connect:?})이 전체 제한 시간({total:?})보다 깁니다"
                ),
            }
        }
    }

    impl std::error::Error for BuildError {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = "https://api.example.com/items";
    let ftp = "ftp://files.example.com/a.zip";
    let json = r#"{"name":"연필"}"#;
    let secs = Duration::from_secs;

    println!("1. 비권장: 공개 필드 타입은 규칙을 어긴 값도 받습니다");
    let loose_cases = [
        ("ftp 주소", Method::Get, ftp, None, secs(30), secs(5)),
        (
            "본문이 있는 GET",
            Method::Get,
            api,
            Some(json),
            secs(30),
            secs(5),
        ),
        (
            "연결 60s > 전체 10s",
            Method::Get,
            api,
            None,
            secs(10),
            secs(60),
        ),
    ];
    for (label, method, url, body, timeout, connect_timeout) in loose_cases {
        let r = loose::Request {
            method,
            url: url.to_string(),
            body: body.map(str::to_string),
            timeout,
            connect_timeout,
        };
        println!("    {label} → 만들어짐: {}", r.describe());
    }
    // 출력:     ftp 주소 → 만들어짐: GET ftp://files.example.com/a.zip | 본문 없음 | 전체 30s, 연결 5s
    // 출력:     본문이 있는 GET → 만들어짐: GET https://api.example.com/items | 본문 있음 | 전체 30s, 연결 5s
    // 출력:     연결 60s > 전체 10s → 만들어짐: GET https://api.example.com/items | 본문 없음 | 전체 10s, 연결 60s

    println!("2. 권장: 빌더의 build()가 규칙을 한곳에서 검사합니다");
    // 한 줄 체인: 소비형 빌더는 임시값에서 시작해 build()까지 그대로 이어 씁니다.
    let ok = http::Request::builder(Method::Post, api)
        .header("Content-Type: application/json")
        .body(json)
        .timeout(secs(10))
        .build()?;
    println!("    정상 → {}", ok.describe());
    // 출력:     정상 → POST https://api.example.com/items | 헤더 ["Content-Type: application/json"] | 본문 17바이트 | 전체 10s, 연결 5s

    let results = [
        ("ftp 주소", http::Request::builder(Method::Get, ftp).build()),
        (
            "본문이 있는 GET",
            http::Request::builder(Method::Get, api).body(json).build(),
        ),
        (
            "연결 60s > 전체 10s",
            http::Request::builder(Method::Get, api)
                .timeout(secs(10))
                .connect_timeout(secs(60))
                .build(),
        ),
    ];
    for (label, result) in results {
        match result {
            Ok(r) => println!("    {label} → {}", r.describe()),
            Err(e) => println!("    {label} → 에러: {e}"),
        }
    }
    // 출력:     ftp 주소 → 에러: 주소는 http:// 또는 https://로 시작해야 합니다: "ftp://files.example.com/a.zip"
    // 출력:     본문이 있는 GET → 에러: GET 요청에는 본문을 넣을 수 없습니다
    // 출력:     연결 60s > 전체 10s → 에러: 연결 제한 시간(60s)이 전체 제한 시간(10s)보다 깁니다

    println!("3. 표준 라이브러리의 소비형 빌더: std::thread::Builder");
    // name(self, String) -> Builder, spawn(self, f) -> io::Result<JoinHandle<T>>입니다.
    // spawn이 빌더를 소비하므로 빌더 하나가 스레드 하나를 만들고, 이름 String은 복제 없이
    // 새 스레드로 옮겨 갑니다. 스레드를 만들지 못하면(운영체제 자원 부족 등) Err입니다.
    let handle = thread::Builder::new()
        .name("fetcher-1".to_string())
        .stack_size(256 * 1024)
        .spawn(|| thread::current().name().map(str::to_string))?;
    match handle.join() {
        Ok(name) => println!("    새 스레드가 본 자기 이름: {name:?}"),
        Err(_) => println!("    스레드가 패닉했습니다"),
    }
    // 출력:     새 스레드가 본 자기 이름: Some("fetcher-1")
    Ok(())
}
