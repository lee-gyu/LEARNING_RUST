// 지침 5: 빌더를 손으로 쓰는 반복이 부담스러우면 derive로 만들되, 생성된 빌더가 무엇을
// 하는지(형태, 필수 값을 다루는 방식, 기본값과 검증의 순서)를 확인하고 씁니다.
//
// `derive_builder` 0.20.2(챕터 Cargo.toml의 [dev-dependencies])의 `#[derive(Builder)]`는
// 기본으로 `&mut` 빌더를 만듭니다. 세터는 `&mut self`를 받아 `&mut RequestBuilder`를
// 돌려주고, `build(&self)`는 `Result<Request, RequestBuilderError>`를 돌려줍니다(1번 구역).
// 같은 설정과 같은 규칙이면 예제 04 모양의 손으로 쓴 빌더와 결과가 같습니다(2·3번 구역).
// 다른 점은 필수 값입니다. 손으로 쓴 판은 생성자 인자라 빠뜨리면 컴파일되지 않고,
// derive 판은 실행 중에 `Err`가 됩니다(4번 구역).
//
// 실행: cargo run -p er-ch7-builders --example 05_derive_builder

use std::any::type_name_of_val;
use std::fmt;
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

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_CONNECT: Duration = Duration::from_secs(5);

/// 두 판이 함께 쓰는 규칙입니다. 규칙을 한 함수에 두어 두 판의 판단이 같게 합니다.
#[derive(Debug)]
pub enum RuleError {
    BodyWithGet,
    ConnectLongerThanTotal { connect: Duration, total: Duration },
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleError::BodyWithGet => write!(f, "GET 요청에는 본문을 넣을 수 없습니다"),
            RuleError::ConnectLongerThanTotal { connect, total } => write!(
                f,
                "연결 제한 시간({connect:?})이 전체 제한 시간({total:?})보다 깁니다"
            ),
        }
    }
}

impl std::error::Error for RuleError {}

fn check_rules(
    method: Method,
    has_body: bool,
    timeout: Duration,
    connect: Duration,
) -> Result<(), RuleError> {
    if method == Method::Get && has_body {
        return Err(RuleError::BodyWithGet);
    }
    if connect > timeout {
        return Err(RuleError::ConnectLongerThanTotal {
            connect,
            total: timeout,
        });
    }
    Ok(())
}

fn describe(
    method: Method,
    url: &str,
    headers: &[String],
    body: Option<&str>,
    timeout: Duration,
    connect: Duration,
) -> String {
    format!(
        "{} {url} | 헤더 {headers:?} | 본문 {} | 전체 {timeout:?}, 연결 {connect:?}",
        method.as_str(),
        match body {
            Some(b) => format!("{}바이트", b.len()),
            None => "없음".to_string(),
        }
    )
}

// ---- 손으로 쓴 판: 예제 04와 같은 모양의 `&mut` 빌더 ----

mod by_hand {
    use super::{DEFAULT_CONNECT, DEFAULT_TIMEOUT, Method, RuleError, check_rules};
    use std::time::Duration;

    pub struct Request {
        method: Method,
        url: String,
        headers: Vec<String>,
        body: Option<String>,
        timeout: Duration,
        connect_timeout: Duration,
    }

    impl Request {
        pub fn describe(&self) -> String {
            super::describe(
                self.method,
                &self.url,
                &self.headers,
                self.body.as_deref(),
                self.timeout,
                self.connect_timeout,
            )
        }
    }

    pub struct RequestBuilder {
        method: Method,
        url: String,
        headers: Vec<String>,
        body: Option<String>,
        timeout: Duration,
        connect_timeout: Duration,
    }

    impl RequestBuilder {
        /// 필수 값은 생성자 인자입니다.
        pub fn new(method: Method, url: impl Into<String>) -> Self {
            RequestBuilder {
                method,
                url: url.into(),
                headers: Vec::new(),
                body: None,
                timeout: DEFAULT_TIMEOUT,
                connect_timeout: DEFAULT_CONNECT,
            }
        }

        pub fn header(&mut self, line: impl Into<String>) -> &mut Self {
            self.headers.push(line.into());
            self
        }

        pub fn body(&mut self, body: impl Into<String>) -> &mut Self {
            self.body = Some(body.into());
            self
        }

        pub fn timeout(&mut self, d: Duration) -> &mut Self {
            self.timeout = d;
            self
        }

        pub fn connect_timeout(&mut self, d: Duration) -> &mut Self {
            self.connect_timeout = d;
            self
        }

        pub fn build(&self) -> Result<Request, RuleError> {
            check_rules(
                self.method,
                self.body.is_some(),
                self.timeout,
                self.connect_timeout,
            )?;
            Ok(Request {
                method: self.method,
                url: self.url.clone(),
                headers: self.headers.clone(),
                body: self.body.clone(),
                timeout: self.timeout,
                connect_timeout: self.connect_timeout,
            })
        }
    }
}

// ---- derive 판: 속성으로 같은 빌더를 만들게 합니다 ----

mod derived {
    use super::{DEFAULT_CONNECT, DEFAULT_TIMEOUT, Method, check_rules};
    use derive_builder::Builder;
    use std::time::Duration;

    // 이 정의에서 `RequestBuilder`(필드마다 Option을 담는 빌더, Clone·Default 구현)와
    // `RequestBuilderError`(UninitializedField, ValidationError 배리언트)가 생성됩니다.
    #[derive(Builder)]
    #[builder(build_fn(validate = "Self::validate"))]
    pub struct Request {
        method: Method, // 기본값 없음: 정하지 않고 build()하면 Err
        #[builder(setter(into))] // 세터가 impl Into<String>을 받음
        url: String, // 기본값 없음
        #[builder(setter(each(name = "header", into)), default)] // 한 줄씩 덧붙이는 header()
        headers: Vec<String>,
        #[builder(setter(into, strip_option), default)] // body("..")로 Some을 넣음
        body: Option<String>,
        #[builder(default = "DEFAULT_TIMEOUT")]
        timeout: Duration,
        #[builder(default = "DEFAULT_CONNECT")]
        connect_timeout: Duration,
    }

    impl Request {
        pub fn describe(&self) -> String {
            super::describe(
                self.method,
                &self.url,
                &self.headers,
                self.body.as_deref(),
                self.timeout,
                self.connect_timeout,
            )
        }
    }

    impl RequestBuilder {
        /// `build()`가 필드를 채우기 전에 부르는 검증 함수입니다. 반환 타입의 `String`은
        /// `RequestBuilderError::ValidationError`에 담깁니다.
        ///
        /// 기본값은 이 함수가 끝난 뒤에 채워지므로, 부르는 쪽이 정하지 않은 필드는 여기서
        /// `None`입니다. `Some`끼리만 비교하면 "연결 60s, 전체는 기본값 30s"를 놓치므로
        /// 기본값을 직접 채워 넣고 비교합니다.
        fn validate(&self) -> Result<(), String> {
            let Some(method) = self.method else {
                return Ok(()); // 필수 값이 없으면 build()가 UninitializedField로 알립니다.
            };
            check_rules(
                method,
                matches!(self.body, Some(Some(_))),
                self.timeout.unwrap_or(DEFAULT_TIMEOUT),
                self.connect_timeout.unwrap_or(DEFAULT_CONNECT),
            )
            .map_err(|e| e.to_string())
        }

        /// 3번 구역에서 검증 함수가 보는 값을 보여 주려고 둔 메서드입니다.
        pub fn held_timeouts(&self) -> (Option<Duration>, Option<Duration>) {
            (self.timeout, self.connect_timeout)
        }
    }
}

fn main() {
    let url = "https://api.example.com/items";
    let json = r#"{"name":"연필"}"#;
    let secs = Duration::from_secs;
    // type_name이 붙이는 크레이트 이름(예제 이름)을 떼어 읽기 쉽게 합니다.
    let short = |name: &str| name.replace(concat!(module_path!(), "::"), "");

    println!("1. derive_builder가 만든 빌더의 모양");
    let mut g = derived::RequestBuilder::default();
    // `&g.url(url)`처럼 한 번 더 빌려 넘깁니다. `g.url(url)`을 그대로 넘기면
    // `&mut RequestBuilder`가 `&RequestBuilder`로 강제 변환되어 `&mut`이 보이지 않습니다.
    println!(
        "    세터의 반환 타입: {}",
        short(type_name_of_val(&g.url(url)))
    );
    // 출력:     세터의 반환 타입: &mut derived::RequestBuilder
    println!(
        "    build의 반환 타입: {}",
        short(type_name_of_val(&g.build()))
    );
    // 출력:     build의 반환 타입: core::result::Result<derived::Request, derived::RequestBuilderError>

    println!("2. 같은 사용법, 같은 결과");
    // 시작만 다릅니다. 손으로 쓴 판은 필수 값을 new에, derive 판은 세터에 넣습니다.
    let mut h = by_hand::RequestBuilder::new(Method::Post, url);
    let mut g = derived::RequestBuilder::default();
    g.method(Method::Post).url(url);
    // 여기부터는 글자 하나 다르지 않은 코드입니다.
    h.header("Content-Type: application/json")
        .body(json)
        .timeout(secs(10));
    g.header("Content-Type: application/json")
        .body(json)
        .timeout(secs(10));
    match (h.build(), g.build()) {
        (Ok(a), Ok(b)) => {
            println!("    손으로 쓴 판: {}", a.describe());
            println!("    derive 판:   {}", b.describe());
            println!("    같은가: {}", a.describe() == b.describe());
        }
        (a, b) => println!("    예상과 다름: {:?} / {:?}", a.err(), b.err()),
    }
    // 출력:     손으로 쓴 판: POST https://api.example.com/items | 헤더 ["Content-Type: application/json"] | 본문 17바이트 | 전체 10s, 연결 5s
    // 출력:     derive 판:   POST https://api.example.com/items | 헤더 ["Content-Type: application/json"] | 본문 17바이트 | 전체 10s, 연결 5s
    // 출력:     같은가: true

    println!("3. 같은 규칙으로 거절");
    // 본문이 있는 GET
    let mut h = by_hand::RequestBuilder::new(Method::Get, url);
    h.body(json);
    let mut g = derived::RequestBuilder::default();
    g.method(Method::Get).url(url).body(json);
    report("본문이 있는 GET", h.build().err(), g.build().err());
    // 출력:     본문이 있는 GET
    // 출력:       손으로 쓴 판: GET 요청에는 본문을 넣을 수 없습니다
    // 출력:       derive 판:   GET 요청에는 본문을 넣을 수 없습니다

    // 연결 제한 시간만 60s로 정하고 전체 제한 시간은 기본값(30s)에 맡김
    let mut h = by_hand::RequestBuilder::new(Method::Get, url);
    h.connect_timeout(secs(60));
    let mut g = derived::RequestBuilder::default();
    g.method(Method::Get).url(url).connect_timeout(secs(60));
    report("연결 60s, 전체는 기본값", h.build().err(), g.build().err());
    // 출력:     연결 60s, 전체는 기본값
    // 출력:       손으로 쓴 판: 연결 제한 시간(60s)이 전체 제한 시간(30s)보다 깁니다
    // 출력:       derive 판:   연결 제한 시간(60s)이 전체 제한 시간(30s)보다 깁니다
    let (timeout, connect) = g.held_timeouts();
    println!("      검증 함수가 보는 값: timeout = {timeout:?}, connect_timeout = {connect:?}");
    // 출력:       검증 함수가 보는 값: timeout = None, connect_timeout = Some(60s)

    println!("4. 다른 점: 필수 값을 빠뜨리면");
    // 손으로 쓴 판에서는 `by_hand::RequestBuilder::new(Method::Get)`처럼 주소를 빠뜨리면
    // E0061(인자 개수가 다름)로 컴파일되지 않습니다. derive 판은 컴파일되고, build()가 알립니다.
    let mut g = derived::RequestBuilder::default();
    g.method(Method::Get);
    match g.build() {
        Ok(r) => println!("    만들어짐: {}", r.describe()),
        Err(e) => println!("    derive 판: Err({e:?}) → {e}"),
    }
    // 출력:     derive 판: Err(UninitializedField("url")) → `url` must be initialized
}

fn report(label: &str, by_hand: Option<RuleError>, derived: Option<derived::RequestBuilderError>) {
    let show = |e: Option<String>| e.unwrap_or_else(|| "통과(거절하지 못함)".to_string());
    println!("    {label}");
    println!(
        "      손으로 쓴 판: {}",
        show(by_hand.map(|e| e.to_string()))
    );
    println!(
        "      derive 판:   {}",
        show(derived.map(|e| e.to_string()))
    );
}
