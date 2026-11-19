// 기대하는 에러: E0716 — temporary value dropped while borrowed
// 핵심 메시지: creates a temporary value which is freed while still in use
//             / temporary value is freed at the end of this statement
//             / help: consider using a `let` binding to create a longer lived value
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0716_mut_builder_from_temporary.rs --out-dir /tmp/er-ch7
//   (2015·2018·2021 에디션으로 컴파일해도 같은 E0716입니다.)
//
// `&mut` 빌더의 세터는 빌더를 빌린 `&mut RequestBuilder`를 돌려줍니다. `RequestBuilder::new(..)`가
// 만든 빌더는 어느 변수에도 담기지 않은 임시값이라 이 `let` 문이 끝날 때 버려지는데,
// `b`는 그 임시값을 빌린 참조라서 다음 문장에서 쓸 수 없습니다.
// 해결: 빌더를 먼저 변수에 담고 세터는 그 변수에 부릅니다.
//   let mut b = RequestBuilder::new(..);
//   b.header(..);
// 또는 한 문장 안에서 build()까지 끝냅니다(examples/04_mut_builder.rs의 3번 구역).

use std::time::Duration;

struct Request {
    url: String,
    headers: Vec<String>,
    timeout: Duration,
}

struct RequestBuilder {
    url: String,
    headers: Vec<String>,
    timeout: Duration,
}

impl RequestBuilder {
    fn new(url: &str) -> Self {
        RequestBuilder {
            url: url.to_string(),
            headers: Vec::new(),
            timeout: Duration::from_secs(30),
        }
    }

    fn header(&mut self, line: &str) -> &mut Self {
        self.headers.push(line.to_string());
        self
    }

    fn timeout(&mut self, d: Duration) -> &mut Self {
        self.timeout = d;
        self
    }

    fn build(&self) -> Request {
        Request {
            url: self.url.clone(),
            headers: self.headers.clone(),
            timeout: self.timeout,
        }
    }
}

fn main() {
    let slow_network = true;
    let b = RequestBuilder::new("https://api.example.com/me").header("Accept: application/json");
    if slow_network {
        b.timeout(Duration::from_secs(60)); // E0716: b가 가리키는 임시 빌더는 이미 버려졌음
    }
    let request = b.build();
    println!("{} {:?} {:?}", request.url, request.headers, request.timeout);
}
