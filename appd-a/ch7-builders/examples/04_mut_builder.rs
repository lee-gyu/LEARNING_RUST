// 지침 4: 빌더를 주로 어떻게 쓰는지에 맞춰 소비형과 `&mut` 빌더 중 하나를 고릅니다.
//
// - 여러 문장에 걸쳐 조건에 따라 조립하거나, 같은 설정으로 값을 여러 개 만든다면
//   `&mut` 빌더(`&mut self`를 받아 `&mut Self`를 돌려줌)가 편합니다(1·2번 구역).
// - 설정을 채운 빌더를 함수에서 돌려주는 것처럼 한 줄 체인이 주된 쓰임이라면
//   소비형 빌더(`self`를 받아 `Self`를 돌려줌)가 편합니다(3번 구역).
//
// 이 예제의 요청은 주소, 헤더, 제한 시간만 갖고 지킬 규칙도 없으므로, `build()`는
// `Result`가 아니라 `Request`를 바로 돌려줍니다(예제 03과 비교해 보세요).
//
// 실행: cargo run -p er-ch7-builders --example 04_mut_builder

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Duration;
use std::{env, process};

#[derive(Debug, PartialEq)]
pub struct Request {
    url: String,
    headers: Vec<String>,
    timeout: Duration,
}

impl Request {
    fn describe(&self) -> String {
        format!(
            "{} | 헤더 {:?} | 전체 {:?}",
            self.url, self.headers, self.timeout
        )
    }
}

/// 소비형 빌더. 같은 설정으로 여러 개를 만들려면 빌더 자체를 복제해야 하므로 `Clone`을 붙입니다.
mod owned {
    use super::Request;
    use std::time::Duration;

    #[derive(Clone)]
    pub struct RequestBuilder {
        url: String,
        headers: Vec<String>,
        timeout: Duration,
    }

    impl RequestBuilder {
        pub fn new(url: impl Into<String>) -> Self {
            RequestBuilder {
                url: url.into(),
                headers: Vec::new(),
                timeout: Duration::from_secs(30),
            }
        }

        pub fn url(mut self, url: impl Into<String>) -> Self {
            self.url = url.into();
            self
        }

        pub fn header(mut self, line: impl Into<String>) -> Self {
            self.headers.push(line.into());
            self
        }

        pub fn timeout(mut self, d: Duration) -> Self {
            self.timeout = d;
            self
        }

        /// 빌더를 소비하므로 필드를 복제 없이 옮깁니다. 대신 한 번만 부를 수 있습니다.
        pub fn build(self) -> Request {
            Request {
                url: self.url,
                headers: self.headers,
                timeout: self.timeout,
            }
        }
    }
}

/// `&mut` 빌더. 세터는 `&mut self`를 받아 같은 빌더를 `&mut Self`로 돌려줍니다.
mod by_ref {
    use super::Request;
    use std::time::Duration;

    pub struct RequestBuilder {
        url: String,
        headers: Vec<String>,
        timeout: Duration,
    }

    impl RequestBuilder {
        pub fn new(url: impl Into<String>) -> Self {
            RequestBuilder {
                url: url.into(),
                headers: Vec::new(),
                timeout: Duration::from_secs(30),
            }
        }

        pub fn url(&mut self, url: impl Into<String>) -> &mut Self {
            self.url = url.into();
            self
        }

        pub fn header(&mut self, line: impl Into<String>) -> &mut Self {
            self.headers.push(line.into());
            self
        }

        pub fn timeout(&mut self, d: Duration) -> &mut Self {
            self.timeout = d;
            self
        }

        /// `&self`만 빌리므로 몇 번이든 부를 수 있습니다. 그 대가로 빌더가 계속 가진
        /// 필드를 복제해서 내줍니다. 비용이 사라진 것이 아니라 build() 안으로 옮겨 왔습니다.
        pub fn build(&self) -> Request {
            Request {
                url: self.url.clone(),
                headers: self.headers.clone(),
                timeout: self.timeout,
            }
        }
    }
}

/// 3번 구역: 공통 설정을 채운 빌더를 돌려주는 함수. 소비형은 표현식 하나로 끝납니다.
fn json_api_owned(url: &str) -> owned::RequestBuilder {
    owned::RequestBuilder::new(url)
        .header("Accept: application/json")
        .timeout(Duration::from_secs(10))
}

/// 같은 함수를 `&mut` 빌더로 쓰면, 체인이 돌려주는 것은 `&mut RequestBuilder`라서
/// 그대로 돌려줄 수 없습니다. 변수에 담고, 고치고, 변수를 돌려주는 세 문장이 필요합니다.
fn json_api_by_ref(url: &str) -> by_ref::RequestBuilder {
    let mut b = by_ref::RequestBuilder::new(url);
    b.header("Accept: application/json")
        .timeout(Duration::from_secs(10));
    b
}

fn main() -> std::io::Result<()> {
    let base = "https://api.example.com";
    let token: Option<&str> = Some("abc123");
    let slow_network = true;

    println!("1. 여러 문장에 걸쳐 조건에 따라 조립하기");
    // 소비형: 메서드가 빌더를 가져가므로 결과를 매번 같은 변수에 다시 담아야 합니다.
    // `b = `를 빠뜨리면 E0382입니다(compile_fail/e0382_consuming_builder_in_if.rs).
    let mut b = owned::RequestBuilder::new(format!("{base}/me"));
    if let Some(t) = token {
        b = b.header(format!("Authorization: Bearer {t}"));
    }
    if slow_network {
        b = b.timeout(Duration::from_secs(60));
    }
    let from_owned = b.build();

    // `&mut` 빌더: 빌더는 제자리에 있고 메서드가 그것을 고칩니다. 다시 담을 필요가 없습니다.
    let mut b = by_ref::RequestBuilder::new(format!("{base}/me"));
    if let Some(t) = token {
        b.header(format!("Authorization: Bearer {t}"));
    }
    if slow_network {
        b.timeout(Duration::from_secs(60));
    }
    let from_by_ref = b.build();

    println!("    소비형: {}", from_owned.describe());
    // 출력:     소비형: https://api.example.com/me | 헤더 ["Authorization: Bearer abc123"] | 전체 60s
    println!("    &mut:   {}", from_by_ref.describe());
    // 출력:     &mut:   https://api.example.com/me | 헤더 ["Authorization: Bearer abc123"] | 전체 60s
    println!("    결과가 같은가: {}", from_owned == from_by_ref);
    // 출력:     결과가 같은가: true

    println!("2. 같은 설정으로 여러 개 만들기");
    let paths = ["items", "tags", "users"];
    // 소비형: build()가 빌더를 소비하므로, 만들 때마다 공통 빌더를 복제합니다.
    let common = owned::RequestBuilder::new(base)
        .header("Accept: application/json")
        .timeout(Duration::from_secs(10));
    let many_owned: Vec<Request> = paths
        .iter()
        .map(|p| common.clone().url(format!("{base}/{p}")).build())
        .collect();
    // `&mut` 빌더: 주소만 바꿔 가며 같은 빌더로 build()를 여러 번 부릅니다.
    let mut b = by_ref::RequestBuilder::new(base);
    b.header("Accept: application/json")
        .timeout(Duration::from_secs(10));
    let mut many_by_ref = Vec::new();
    for p in paths {
        b.url(format!("{base}/{p}"));
        many_by_ref.push(b.build());
    }
    for r in &many_by_ref {
        println!("    {}", r.describe());
    }
    // 출력:     https://api.example.com/items | 헤더 ["Accept: application/json"] | 전체 10s
    // 출력:     https://api.example.com/tags | 헤더 ["Accept: application/json"] | 전체 10s
    // 출력:     https://api.example.com/users | 헤더 ["Accept: application/json"] | 전체 10s
    println!("    두 방식의 결과가 같은가: {}", many_owned == many_by_ref);
    // 출력:     두 방식의 결과가 같은가: true
    // 주의: 빌더를 다시 쓰면 앞에서 넣은 설정이 그대로 남습니다. 한 요청에만 넣으려던 헤더가
    // 다음 요청에도 들어갑니다.
    b.header("X-Trace: on");
    let _traced = b.build();
    b.url(format!("{base}/next"));
    println!("    그다음 요청: {}", b.build().describe());
    // 출력:     그다음 요청: https://api.example.com/next | 헤더 ["Accept: application/json", "X-Trace: on"] | 전체 10s

    println!("3. 한 줄 체인");
    // 함수가 돌려준 빌더에 이어 쓰는 것은 두 형태 모두 됩니다. 차이는 함수 쪽에 있습니다
    // (json_api_owned는 표현식 하나, json_api_by_ref는 세 문장).
    let a = json_api_owned(&format!("{base}/items"))
        .header("X-Page: 2")
        .build();
    // `&mut` 빌더도 한 문장 안에서 build()까지 끝내면 됩니다. 임시값인 빌더는 문장이
    // 끝날 때까지 살아 있기 때문입니다. 체인의 중간 결과(`&mut RequestBuilder`)를 변수에
    // 담아 다음 문장에서 쓰면 E0716입니다(compile_fail/e0716_mut_builder_from_temporary.rs).
    let b = json_api_by_ref(&format!("{base}/items"))
        .header("X-Page: 2")
        .build();
    println!("    소비형: {}", a.describe());
    // 출력:     소비형: https://api.example.com/items | 헤더 ["Accept: application/json", "X-Page: 2"] | 전체 10s
    println!("    &mut:   {}", b.describe());
    // 출력:     &mut:   https://api.example.com/items | 헤더 ["Accept: application/json", "X-Page: 2"] | 전체 10s

    println!("4. 표준 라이브러리의 &mut 빌더: std::fs::OpenOptions");
    // create/append(&mut self, bool) -> &mut Self, open(&self, path) -> io::Result<File>입니다.
    // open이 &self만 빌리므로, 같은 설정으로 몇 번이고 파일을 열 수 있습니다.
    let dir = env::temp_dir().join(format!("er-ch7-builders-{}", process::id()));
    fs::create_dir_all(&dir)?;
    let path = dir.join("notes.txt");
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);
    for line in ["첫째 줄", "둘째 줄"] {
        let mut file = opts.open(&path)?;
        writeln!(file, "{line}")?;
    }
    let text = fs::read_to_string(&path)?;
    fs::remove_dir_all(&dir)?;
    println!("    같은 OpenOptions로 두 번 열어 덧붙인 내용: {text:?}");
    // 출력:     같은 OpenOptions로 두 번 열어 덧붙인 내용: "첫째 줄\n둘째 줄\n"
    Ok(())
}
