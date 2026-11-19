// 기대하는 에러: E0451 — field `headers` of struct `Options` is private
// 핵심 메시지: `..Default::default()`를 가리키며 "field `headers` is private"
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0451_default_update_private_field.rs --out-dir /tmp/er-ch7
//
// `http::Options`는 헤더 줄의 형식을 지키려고 `headers` 필드를 비공개로 두고, 헤더는
// `add_header` 메서드로만 넣게 했습니다. 모듈 밖에서는 `headers`를 적지 않고
// `..Default::default()`에 맡겨도 에러입니다. 구조체 갱신 문법은 적지 않은 필드를 기준 값에서
// 하나씩 옮겨 오는 것이라(Part 0 ch5 2.4절), 비공개 필드를 옮기는 일도 모듈 밖에서 하게
// 되기 때문입니다.
// 해결: 비공개 필드가 있는 타입은 바깥에 생성 경로를 따로 줍니다. `Options::new()`나
// `Options::default()`로 만든 뒤 공개 필드를 고치거나, 빌더를 둡니다
// (examples/03_consuming_builder.rs).

mod http {
    use std::time::Duration;

    pub struct Options {
        pub timeout: Duration,
        pub connect_timeout: Duration,
        headers: Vec<String>, // "이름: 값" 형식만 담도록 메서드로만 고칩니다.
    }

    impl Default for Options {
        fn default() -> Self {
            Options {
                timeout: Duration::from_secs(30),
                connect_timeout: Duration::from_secs(5),
                headers: Vec::new(),
            }
        }
    }

    impl Options {
        pub fn add_header(&mut self, line: &str) -> bool {
            if line.contains(": ") {
                self.headers.push(line.to_string());
                true
            } else {
                false
            }
        }

        pub fn header_count(&self) -> usize {
            self.headers.len()
        }
    }
}

fn main() {
    let options = http::Options {
        timeout: std::time::Duration::from_secs(10),
        ..Default::default() // E0451: 비공개 필드 headers를 여기서 옮겨 와야 함
    };
    println!("{:?} {:?}", options.timeout, options.connect_timeout);
    println!("{}", options.header_count());
    let mut more = http::Options::default();
    println!("{}", more.add_header("Accept: text/html"));
}
