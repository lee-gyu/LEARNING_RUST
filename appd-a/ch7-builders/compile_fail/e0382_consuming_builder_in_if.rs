// 기대하는 에러: E0382 — use of moved value: `b`
// 핵심 메시지: `b` moved due to this method call
//             / note: `RequestBuilder::header` takes ownership of the receiver `self`, which moves `b`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0382_consuming_builder_in_if.rs --out-dir /tmp/er-ch7
//
// 소비형 빌더의 세터는 `self`를 받아 고친 빌더를 돌려줍니다. 조건에 따라 세터를 부를 때
// `&mut` 빌더처럼 `b.header(..);`만 쓰면, 빌더는 그 호출로 옮겨 가고 돌려받은 빌더는
// 버려집니다. 그 뒤의 `b.build()`는 이미 옮겨 간 값을 쓰려는 것이라 거부됩니다.
// 해결: 돌려받은 빌더를 다시 담습니다. `let mut b = ..;` 그리고 `b = b.header(..);`
// (examples/04_mut_builder.rs의 1번 구역). 이런 조립이 잦다면 `&mut` 빌더를 고려합니다.

struct Request {
    url: String,
    headers: Vec<String>,
}

struct RequestBuilder {
    url: String,
    headers: Vec<String>,
}

impl RequestBuilder {
    fn new(url: &str) -> Self {
        RequestBuilder {
            url: url.to_string(),
            headers: Vec::new(),
        }
    }

    fn header(mut self, line: String) -> Self {
        self.headers.push(line);
        self
    }

    fn build(self) -> Request {
        Request {
            url: self.url,
            headers: self.headers,
        }
    }
}

fn main() {
    let token: Option<&str> = Some("abc123");
    let b = RequestBuilder::new("https://api.example.com/me");
    if let Some(t) = token {
        let line = format!("Authorization: Bearer {t}");
        b.header(line); // 빌더가 여기서 옮겨 감
    }
    let request = b.build(); // E0382
    println!("{} {:?}", request.url, request.headers);
}
