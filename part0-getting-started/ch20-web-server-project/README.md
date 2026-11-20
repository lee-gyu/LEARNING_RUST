# ch20. 마지막 프로젝트: 멀티스레드 웹 서버 만들기

Part 0의 마지막 챕터입니다. 지금까지 배운 것을 모아 브라우저에 "안녕하세요!" 페이지를 돌려주는 웹 서버를 만듭니다. TCP 연결을 받고, HTTP 요청을 읽고, 응답을 써서 돌려주는 데서 시작해, 느린 요청 하나가 서버 전체를 멈추게 하는 문제를 직접 관찰한 뒤, **스레드 풀(thread pool)** 을 만들어 해결하고, 마지막으로 서버가 하던 일을 마저 끝내고 깔끔하게 멈추는 **우아한 종료(graceful shutdown)** 를 구현합니다.

실제 서비스용 웹 서버나 스레드 풀은 crates.io에 이미 잘 만들어진 크레이트가 있습니다. 이 챕터의 목적은 그런 크레이트가 안에서 무엇을 하는지 직접 만들어 보며 이해하는 것이고, 무엇보다 **각 단계에서 왜 그렇게 설계하는지**를 앞 챕터의 개념과 연결하는 것입니다. 앞 단계(연결 받기부터 단일 스레드의 한계까지)와 잠금에 관한 실험은 `examples/`에 두었고, 완성판은 독립 프로젝트 `projects/hello-server`에 있습니다.

- 패키지: `p0-ch20-web-server-project`
- 독립 프로젝트: `projects/hello-server` (라이브러리 크레이트의 `ThreadPool` + 바이너리 크레이트의 서버)
- 기준 문서: [The Book 21장 Final Project: Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS. 1.98.1에서 작성했고, 2026-11-20에 예제 출력과 `compile_fail/`의 에러 메시지를 1.100.0에서 다시 확인했습니다(파일 없이 본문에만 인용한 에러 메시지와 `cargo` 출력은 작성 당시 그대로입니다).
- 챕터 번호: 이 저장소는 The Book 14장을 Part 3으로 옮겼기 때문에 Book 21장이 `ch20`입니다. 이 문서에서 "N장"은 The Book의 장을, `chN`은 이 저장소의 챕터를 가리킵니다.
- 네트워크: 모든 예제와 프로젝트는 이 컴퓨터 자신을 가리키는 주소 `127.0.0.1`에서만 동작합니다. 다른 컴퓨터에서는 접속할 수 없습니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `TcpListener`로 연결을 받고, `BufReader`의 `lines`로 HTTP 요청을 빈 줄까지 읽은 뒤, 상태 줄·헤더·본문 형식의 응답을 써서 돌려주는 서버를 작성할 수 있다. 요청 줄에 따라 200과 404 응답을 나눌 수 있다.
2. 단일 스레드 서버에서 느린 요청이 뒤의 요청을 막는 것을 관찰하고, 연결마다 스레드를 만드는 방식의 문제와 스레드 풀이 그 문제를 어떻게 해결하는지 설명할 수 있다.
3. 쓰고 싶은 API를 먼저 쓰고 컴파일 에러를 따라가며 `ThreadPool`을 구현할 수 있다. `execute`의 바운드 `FnOnce() + Send + 'static`, `Worker`, 채널과 `Arc<Mutex<Receiver<Job>>>`, `type Job = Box<dyn FnOnce() + Send + 'static>`의 각 부분이 왜 필요한지 설명할 수 있다.
4. 임시값이 버려지는 시점에 따라 `MutexGuard`의 잠금이 풀리는 시점이 달라진다는 것을 설명하고, 작업을 실행하는 동안 잠금을 쥐고 있지 않는 작업자 반복문을 작성할 수 있다.
5. `Drop`으로 우아한 종료를 구현할 수 있다. 송신자를 `Option`으로 감싸 `take`로 먼저 버리고, 작업자를 꺼내 `join`하는 순서가 왜 중요한지 설명할 수 있다. `new`(패닉)와 `build`(`Result`) 중 무엇을 쓸지 ch9의 기준으로 고를 수 있다.

## 2. 핵심 개념

### 2.1 만들 프로그램과 진행 순서

완성된 서버는 이렇게 동작합니다. 한 터미널에서 서버를 띄우고, 다른 터미널에서 `curl`로 접속합니다.

```console
$ cargo run
http://127.0.0.1:7878 에서 기다립니다 (연결 3개를 처리하면 종료)
```

```console
$ curl http://127.0.0.1:7878/
<!DOCTYPE html>
<html lang="ko">
  ...
    <h1>안녕하세요!</h1>
  ...
```

| 경로 | 응답 |
| --- | --- |
| `/` | `200 OK`와 `hello.html` |
| `/sleep` | 5초 기다린 뒤 `200 OK`와 `hello.html` (오래 걸리는 요청 흉내) |
| 그 밖의 경로 | `404 NOT FOUND`와 `404.html` |

서버는 작업자 스레드 4개로 요청을 동시에 처리하고, 정해진 수의 연결을 처리하면 하던 일을 마저 끝낸 뒤 스스로 종료합니다. 끝없이 도는 서버로 만들지 않은 까닭은 2.9절에서 우아한 종료를 눈으로 확인하기 위해서입니다.

이 챕터는 The Book 21장의 흐름을 따라 다음 순서로 진행합니다.

| 단계 | 파일 | 하는 일 | 절 | The Book |
| --- | --- | --- | --- | --- |
| 1 | [`examples/01_accept_and_read.rs`](examples/01_accept_and_read.rs) | TCP 연결을 받고 HTTP 요청 읽기 | 2.2, 2.3 | 21.1 |
| 2 | [`examples/02_respond.rs`](examples/02_respond.rs) | 응답 쓰기, 요청 줄에 따라 200/404 고르기 | 2.4 | 21.1 |
| 3 | [`examples/03_single_thread_limit.rs`](examples/03_single_thread_limit.rs) | 느린 요청으로 단일 스레드의 한계 관찰, 연결마다 스레드 만들기 | 2.5 | 21.2 |
| 4 | [`projects/hello-server`](projects/hello-server) | 쓰고 싶은 API부터 쓰고 컴파일 에러를 따라 `ThreadPool` 설계 | 2.6 | 21.2 |
| 5 | 〃, [`compile_fail/`](compile_fail/) | 작업자, 채널, 작업 타입 `Job` | 2.7 | 21.2 |
| 6 | [`examples/04_lock_scope.rs`](examples/04_lock_scope.rs) | 작업을 꺼낸 뒤 잠금이 언제 풀리는가 | 2.8 | 21.2 |
| 7 | [`projects/hello-server`](projects/hello-server) | `Drop`으로 우아한 종료, `new`와 `build` | 2.9, 2.10 | 21.3 |

앞 챕터에서 배운 것이 이렇게 쓰입니다.

| 이 챕터에서 쓰이는 곳 | 개념 | 배운 곳 |
| --- | --- | --- |
| `TcpStream`을 넘겨받은 함수가 끝나면 연결이 닫힘 | 소유권, drop | [ch4](../ch4-ownership/README.md), [ch14](../ch14-smart-pointers/README.md) |
| `&request_line[..]`로 요청 줄 비교 | 문자열 슬라이스 | [ch4](../ch4-ownership/README.md) |
| 요청 줄로 응답 고르기, `let...else` | `match`, 패턴 | [ch6](../ch6-enums-pattern-matching/README.md), [ch18](../ch18-patterns/README.md) |
| 서버(바이너리)와 `ThreadPool`(라이브러리) 나누기 | 라이브러리 + 바이너리 패키지 | [ch7](../ch7-modules/README.md), [ch12](../ch12-io-project-minigrep/README.md) |
| `Content-Length`는 바이트 수 | `String`과 UTF-8 | [ch8](../ch8-collections/README.md) |
| `unwrap`, `ThreadPool::build` | `Result`, 패닉과 `Result`의 선택 기준 | [ch9](../ch9-error-handling/README.md) |
| `execute<F>(..) where F: ..` | 제네릭, 트레이트 바운드, `'static` | [ch10](../ch10-generics-traits-lifetimes/README.md) |
| `ThreadPool` 테스트, 문서 테스트 | `#[test]`, `cargo test` | [ch11](../ch11-testing/README.md) |
| `lines().map(..).take_while(..).collect()`, `FnOnce` | 반복자, 클로저 | [ch13](../ch13-closures-iterators/README.md) |
| `Box<dyn ..>`, `impl Drop`, `drop(..)` | 스마트 포인터, `Drop` | [ch14](../ch14-smart-pointers/README.md) |
| 작업자 스레드, 채널, `Arc<Mutex<T>>`, `Send` | 스레드와 동기화 | [ch15](../ch15-concurrency/README.md) |
| `dyn FnOnce()` | 트레이트 객체 | [ch17](../ch17-trait-objects-oop/README.md) |
| `type Job = ..`, 예제의 `fn(TcpListener)` | 별칭, 함수 포인터 | [ch19](../ch19-advanced-features/README.md) |

### 2.2 TCP와 HTTP, 그리고 연결 받기

웹 서버가 쓰는 프로토콜은 두 겹입니다.

- **TCP**(Transmission Control Protocol)는 두 프로그램 사이에 **연결(connection)** 을 만들고, 그 연결로 바이트를 빠짐없이 순서대로 전달합니다. 바이트가 무엇을 뜻하는지는 정하지 않습니다.
- **HTTP**(Hypertext Transfer Protocol)는 TCP 위에서, 주고받는 바이트가 **요청(request)** 과 **응답(response)** 으로서 어떤 모양이어야 하는지 정합니다. 이 챕터에서 다루는 HTTP/1.1은 사람이 읽을 수 있는 텍스트입니다.

한 번의 요청은 이렇게 오갑니다. 클라이언트(브라우저, `curl`, 예제 안의 클라이언트 스레드)가 연결을 맺고 요청을 보내면, 서버가 응답을 보내고 연결을 닫습니다.

Rust 표준 라이브러리의 `std::net`이 TCP를 제공합니다. 연결을 기다리는 쪽은 `TcpListener`입니다.

```rust
use std::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

for stream in listener.incoming() {
    let stream = stream.unwrap();
    println!("연결이 성립했습니다");
}
```

- `127.0.0.1`은 이 컴퓨터 자신을 가리키는 주소이고, `7878`은 **포트(port)** 번호입니다. 포트는 한 컴퓨터 안에서 여러 서버 프로그램을 구분하는 번호입니다. The Book은 7878을 전화기 자판으로 "rust"라고 쳤을 때의 숫자라서 골랐다고 설명합니다.
- 주소와 포트를 차지하고 연결을 기다리기 시작하는 것을 **바인드(bind)** 한다고 합니다. 이미 다른 프로그램이 그 포트를 쓰고 있으면 바인드가 실패하므로 `bind`는 `Result`를 돌려줍니다(4.8절).
- **포트 0**으로 바인드하면 운영체제가 비어 있는 포트를 골라 줍니다. 고른 포트는 `local_addr()`로 알 수 있습니다. 예제는 모두 이 방법을 써서 포트 충돌을 피합니다. 그래서 예제 출력의 포트 번호는 실행할 때마다 다릅니다.
- `incoming()`은 들어오는 **연결 시도**를 하나씩 내놓는 반복자입니다. 연결 시도는 실패할 수 있으므로 각 항목이 `Result<TcpStream, _>`입니다.
- 표준 라이브러리 문서에 따르면 이 반복자는 **절대 `None`을 돌려주지 않습니다.** 그대로 `for` 문에 넣으면 영원히 연결을 기다립니다. 이 챕터의 예제와 서버는 `incoming().take(n)`으로 정해진 수의 연결만 받고 반복을 끝냅니다(ch13의 `take` 어댑터).
- 연결 하나는 `TcpStream` 값 하나입니다. `TcpStream`은 `Read`와 `Write`를 구현해서 파일처럼 읽고 쓸 수 있고, **값이 drop되면 연결이 닫힙니다.** 소유권이 연결의 수명을 정하는 것입니다.

### 2.3 요청 읽기

HTTP 요청은 이런 모양입니다.

```text
메서드 요청-URI HTTP-버전 CRLF
헤더 CRLF
헤더 CRLF
CRLF
본문
```

- 첫 줄을 **요청 줄(request line)** 이라고 합니다. `GET / HTTP/1.1`처럼 메서드(무엇을 해 달라는가), 경로(무엇에 대해), 프로토콜 버전이 들어 있습니다.
- 둘째 줄부터는 `Host: 127.0.0.1:7878`처럼 `이름: 값` 모양의 **헤더(header)** 입니다.
- 줄 끝은 `\n` 하나가 아니라 **CRLF**, 즉 `\r\n`입니다(carriage return + line feed).
- **빈 줄 하나가 헤더의 끝**을 알립니다. `GET` 요청에는 본문이 없습니다.

[`01_accept_and_read.rs`](examples/01_accept_and_read.rs)는 요청을 이렇게 읽습니다.

```rust
use std::io::{BufRead, BufReader};

fn handle_connection(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");
}
```

- `BufReader`는 스트림에서 한 번에 넉넉히 읽어 버퍼에 담아 두고, 그 안에서 줄을 잘라 줍니다. 한 줄을 읽을 때마다 운영체제를 부르지 않아도 됩니다.
- `BufReader::new(&stream)`은 스트림을 **빌려서** 읽습니다. `&TcpStream`도 `Read`를 구현하기 때문입니다. 다음 단계에서 같은 스트림에 응답을 써야 하므로 소유권을 넘기지 않습니다.
- `lines()`(`BufRead` 트레이트의 메서드)는 줄마다 `Result<String>`을 내놓고, 줄 끝의 `\n`이나 `\r\n`을 떼어 냅니다. 01의 출력에서 클라이언트가 보낸 `"GET / HTTP/1.1\r\n..."`이 서버에서는 `"GET / HTTP/1.1"`로 보이는 것이 그 때문입니다.
- `take_while(|line| !line.is_empty())`는 **빈 줄이 나오면 멈춥니다.**

왜 끝까지 읽지 않고 빈 줄에서 멈출까요? 클라이언트는 요청을 다 보낸 뒤에도 **응답을 받으려고 연결을 열어 둔 채 기다립니다.** 서버가 `read_to_string`처럼 "연결이 닫힐 때까지" 읽으면, 서버는 클라이언트가 닫기를 기다리고 클라이언트는 서버의 응답을 기다리게 됩니다. 이 환경에서 그렇게 만든 서버에 `curl --max-time 2`로 요청하자, `curl`은 2초 동안 아무것도 받지 못하고 `Operation timed out`으로 포기했고, 서버의 `read_to_string`은 `curl`이 포기하고 연결을 끊은 뒤에야 돌아왔습니다. HTTP가 빈 줄로 헤더의 끝을 알리는 것은 바로 이런 상황 때문입니다.

### 2.4 응답 쓰기와 요청에 따른 응답

HTTP 응답은 이런 모양입니다.

```text
HTTP-버전 상태-코드 이유-구문 CRLF
헤더 CRLF
CRLF
본문
```

첫 줄을 **상태 줄(status line)** 이라고 합니다. `HTTP/1.1 200 OK`는 "성공", `HTTP/1.1 404 NOT FOUND`는 "그런 것은 없음"이라는 뜻입니다. 가장 짧은 성공 응답은 `"HTTP/1.1 200 OK\r\n\r\n"`이고, 여기에 본문으로 HTML을 붙입니다.

[`02_respond.rs`](examples/02_respond.rs)의 서버는 이렇게 응답합니다.

```rust
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, contents) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", HELLO_HTML),
        _ => ("HTTP/1.1 404 NOT FOUND", NOT_FOUND_HTML),
    };

    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
```

- 무엇을 돌려줄지는 **요청 줄 하나**로 정할 수 있으므로, 이번에는 `lines().next()`로 첫 줄만 읽습니다. `next()`는 `Option`(줄이 없을 수도 있음)을, 그 안은 `Result`(읽다가 실패할 수도 있음)를 돌려주므로 `unwrap`이 두 번입니다.
- `&request_line[..]`은 `String` 전체를 `&str`로 빌린 슬라이스입니다. `match`의 문자열 리터럴 패턴과 비교하려면 `&str`이어야 합니다(`request_line.as_str()`도 같은 일을 합니다).
- `match`는 **200과 404에서 달라지는 것**(상태 줄과 본문)만 고르고, 응답을 만들어 쓰는 코드는 한 번만 씁니다. The Book은 `if`/`else`로 두 갈래를 따로 쓴 다음, 겹치는 코드를 이렇게 정리하는 과정을 보여 줍니다.
- `Content-Length` 헤더는 본문이 몇 바이트인지 알려 줍니다. 연결 하나로 여러 응답을 주고받을 때도 클라이언트가 본문의 끝을 알 수 있게 하는 헤더입니다. **글자 수가 아니라 바이트 수**여야 하는데, `String::len`이 바로 바이트 수를 돌려줍니다(ch8). 02의 출력에서 한글이 든 본문은 바이트 수(136)와 글자 수(122)가 다릅니다.
- `write_all`은 바이트 슬라이스(`&[u8]`)를 받으므로 `as_bytes()`로 바꿔 넘기고, 쓰기는 스트림의 상태를 바꾸므로 매개변수가 `mut stream`입니다.
- 함수가 끝나면 `stream`이 drop되어 연결이 닫히고, 클라이언트의 읽기가 끝납니다.

### 2.5 단일 스레드 서버의 한계와 스레드 풀

지금까지의 서버는 `main` 스레드 하나가 연결을 **하나씩 차례로** 처리합니다. 요청 처리가 오래 걸리면 어떻게 될까요? The Book은 5초를 기다린 뒤 응답하는 `/sleep` 경로로 이를 흉내 냅니다.

```rust
let (status_line, filename) = match &request_line[..] {
    "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
    "GET /sleep HTTP/1.1" => {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    }
    _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
};
```

[`03_single_thread_limit.rs`](examples/03_single_thread_limit.rs)는 예제가 빨리 끝나도록 1초를 기다리게 하고, 클라이언트 A가 `/sleep`을, 0.1초 뒤 클라이언트 B가 `/`를 요청합니다.

```text
1) 연결을 하나씩 처리하는 서버
    0.0초  A가 /sleep 요청
    0.1초  B가 / 요청
    1.0초  A가 응답 받음 (HTTP/1.1 200 OK)
    1.0초  B가 응답 받음 (HTTP/1.1 200 OK)
```

B의 요청은 즉시 처리될 수 있는 것인데도 A가 끝날 때까지 0.9초를 기다렸습니다. 1초가 아니라 한 시간짜리 요청이었다면 그동안 서버는 아무에게도 응답하지 못합니다. B의 `connect`는 0.1초에 바로 성공했다는 점도 눈여겨보세요. 연결 자체는 운영체제가 받아 대기열에 넣어 두지만, **서버 프로그램이 그 연결을 꺼내 처리하는 것**은 A가 끝난 뒤입니다.

가장 쉬운 해결책은 연결마다 스레드를 새로 만드는 것입니다(The Book Listing 21-11).

```rust
for stream in listener.incoming() {
    let stream = stream.unwrap();
    thread::spawn(|| {
        handle_connection(stream);
    });
}
```

```text
2) 연결마다 스레드를 만드는 서버
    0.0초  A가 /sleep 요청
    0.1초  B가 / 요청
    0.1초  B가 응답 받음 (HTTP/1.1 200 OK)
    1.0초  A가 응답 받음 (HTTP/1.1 200 OK)
```

B가 더는 A를 기다리지 않습니다. 하지만 이 방식에는 **상한이 없습니다.** 요청이 1000만 개 들어오면 스레드도 1000만 개를 만들려 하고, 그러다 메모리와 운영체제 자원이 바닥나 서버 전체가 멈출 수 있습니다. 누군가 일부러 요청을 쏟아부어 서버를 멈추게 하는 서비스 거부(denial of service) 공격에 그대로 노출되는 것입니다.

**스레드 풀**은 이 두 극단의 중간입니다.

- 스레드를 정해진 수(예: 4개)만 **미리** 만들어 둡니다. 이 스레드를 작업자 스레드(worker thread)라고 부릅니다.
- 새 요청이 오면 그 처리를 **작업(job)** 으로 만들어 대기열(queue)에 넣고, 손이 빈 작업자가 하나씩 꺼내 처리합니다.
- 동시에 처리하는 요청은 많아야 작업자 수만큼이고, 나머지는 대기열에서 차례를 기다립니다. 느린 요청이 작업자 하나를 붙잡고 있어도 다른 작업자가 빠른 요청을 처리합니다.

작업자 4개가 모두 느린 요청을 처리 중이면 다섯 번째 요청은 기다려야 합니다(3절 프로젝트의 두 번째 실행에서 확인합니다). 그래도 스레드 수가 늘지 않으므로 서버가 자원을 다 써 버리지는 않습니다. The Book은 처리량을 높이는 다른 방법으로 fork/join 모델, 단일 스레드 비동기 I/O 모델, 멀티스레드 비동기 I/O 모델을 소개하고, 많은 비동기 런타임이 안에서 스레드 풀을 쓴다고 설명합니다. 비동기는 [ch16](../ch16-async/README.md)에서 다뤘습니다.

### 2.6 컴파일러가 이끄는 설계: 쓰고 싶은 API부터

이제 `ThreadPool`을 만듭니다. The Book은 구현부터 하지 않고, **이 타입을 쓰는 코드를 먼저 씁니다.** 쓰는 쪽에서 편한 모양을 정해 두고, 컴파일러가 내는 에러를 하나씩 고치며 구현을 채워 가는 방식입니다. The Book은 이것을 컴파일러가 이끄는 개발(compiler-driven development)이라고 부릅니다. 테스트를 먼저 쓰는 ch12의 TDD와 같은 생각을 테스트 대신 타입 검사로 하는 것입니다.

`thread::spawn` 자리를 이렇게 바꾸고 싶습니다.

```rust
let pool = ThreadPool::new(4);

for stream in listener.incoming() {
    let stream = stream.unwrap();

    pool.execute(|| {
        handle_connection(stream);
    });
}
```

`thread::spawn`과 쓰는 법이 거의 같도록 `execute`가 클로저를 받게 했습니다. 이제 `cargo check`를 실행하며 에러를 따라갑니다. (아래 메시지는 이 저장소의 툴체인에서 실제로 확인한 것입니다. 줄 번호는 `main.rs`의 모양에 따라 다릅니다.)

**① 타입이 없다 — E0433**

```text
error[E0433]: cannot find type `ThreadPool` in this scope
 --> src/main.rs:9:16
  |
9 |     let pool = ThreadPool::new(4);
  |                ^^^^^^^^^^ use of undeclared type `ThreadPool`
```

스레드 풀은 웹 서버와 상관없이 다른 프로그램에서도 쓸 수 있는 부품이므로 **라이브러리 크레이트**(`src/lib.rs`)에 둡니다(ch7, ch12의 관심사 분리). `src/lib.rs`에 `pub struct ThreadPool;`을 쓰고, `main.rs` 위에 `use hello_server::ThreadPool;`을 추가합니다. 패키지 이름 `hello-server`의 `-`는 크레이트 이름에서 `_`로 바뀝니다.

**② 연관 함수가 없다 — E0599**

```text
error[E0599]: no associated function or constant named `new` found for struct `ThreadPool` in the current scope
  --> src/main.rs:10:28
   |
10 |     let pool = ThreadPool::new(4);
   |                            ^^^ associated function or constant not found in `ThreadPool`
```

`new(size: usize) -> ThreadPool`을 추가합니다. 스레드 수는 음수일 수 없으므로 `usize`가 알맞습니다.

**③ 메서드가 없다 — E0599**

```text
error[E0599]: no method named `execute` found for struct `ThreadPool` in the current scope
  --> src/main.rs:15:14
   |
15 |         pool.execute(|| {
   |         -----^^^^^^^ method not found in `ThreadPool`
```

`execute`는 클로저를 받아 다른 스레드에서 실행해야 합니다. 같은 일을 하는 표준 라이브러리 함수 `thread::spawn`의 시그니처(ch15 2.4절)를 참고합니다.

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

우리는 작업의 반환값을 돌려줄 생각이 없으므로 `T`는 필요 없고, `F`의 바운드만 가져옵니다.

```rust
pub fn execute<F>(&self, f: F)
where
    F: FnOnce() + Send + 'static,
{
}
```

바운드의 각 부분이 하는 일은 이렇습니다.

| 부분 | 뜻 | 없으면 |
| --- | --- | --- |
| `FnOnce()` | 인자 없이 **한 번** 부를 수 있는 클로저. `()`는 인자가 없다는 뜻이고, 반환 타입을 적지 않았으니 반환값은 `()`입니다. | — |
| `Send` | 클로저를 다른 스레드(작업자)로 **옮겨도** 안전함 | E0277 `` `F` cannot be sent between threads safely`` |
| `'static` | 클로저가 언제 사라질지 모르는 지역 변수의 참조를 품고 있지 않음 | E0310 `` the parameter type `F` may not live long enough`` (4.3절) |

- **왜 `FnOnce`인가**: 요청 하나를 처리하는 클로저는 딱 한 번 실행합니다. ch13에서 배웠듯이 `FnOnce`는 모든 클로저가 구현하는 가장 느슨한 바운드라서, 캡처한 값을 옮겨 버리는 클로저(예: `stream`을 `handle_connection`에 넘기는 클로저)까지 받을 수 있습니다.
- **왜 `Send`인가**: 작업은 `execute`를 부른 스레드에서 만들어져 작업자 스레드에서 실행됩니다. 스레드 경계를 넘으므로 `Send`여야 합니다(ch15 2.9절). 바운드에서 `Send`를 빼면, 뒤에서 작업을 채널에 넣는 줄에서 이런 에러가 납니다.

  ```text
  error[E0277]: `F` cannot be sent between threads safely
     = note: required for the cast from `Box<F>` to `Box<(dyn FnOnce() + Send + 'static)>`
  help: consider further restricting type parameter `F` with trait `Send`
  ```

  (이 환경에서 확인한 메시지에서 핵심 줄만 옮겼습니다.)
- **왜 `'static`인가**: 작업이 언제 실행될지는 작업자의 사정에 달려 있습니다. `execute`를 부른 함수가 이미 끝난 뒤일 수도 있습니다. 그래서 지역 변수를 **빌린** 클로저는 받을 수 없습니다. 실제로 `let name = String::from("러스트"); pool.execute(|| println!("안녕, {name}"));`라고 쓰면, ch15에서 `thread::spawn`으로 만났던 것과 똑같은 E0373(``closure may outlive the current function, but it borrows `name` ``)과 `move`를 붙이라는 제안이 나옵니다.

`main.rs`의 `pool.execute(|| { handle_connection(stream); })`에는 `move`가 없는데도 컴파일됩니다. `handle_connection`이 `stream`의 소유권을 가져가므로, 클로저가 `stream`을 **값으로** 캡처하기 때문입니다(ch13 2.3절). 그래서 이 클로저는 아무것도 빌리지 않고, `'static`을 만족합니다.

본문이 빈 `execute`를 쓰면 경고(쓰지 않은 변수 `size`, `f`)만 남고 컴파일이 됩니다. 하지만 이 환경에서 그 상태의 서버에 `curl`로 요청하자 응답 대신 `curl: (56) Recv failure: Connection reset by peer`가 나왔습니다. `execute`가 받은 클로저를 실행하지 않고 버리면서, 클로저가 캡처한 `stream`도 함께 버려져 연결이 끊겼기 때문입니다. Rust에서는 흔히 "컴파일되면 동작한다"고 하지만, 타입 검사는 **모양**만 확인할 뿐입니다. 이제 실제로 일을 하는 부분을 채웁니다.

### 2.7 작업자, 채널, 작업 타입

#### 작업자: 일을 기다리는 스레드

`thread::spawn`은 넘긴 클로저를 **곧바로** 실행합니다. 스레드 풀은 스레드를 먼저 만들어 두고 일은 나중에 줘야 하므로, 각 스레드는 "작업이 오기를 기다렸다가 실행하고, 다시 기다리는" 반복문을 실행해야 합니다. The Book은 스레드 하나와 그 반복문을 묶어 **`Worker`**(작업자)라고 부릅니다.

```rust
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}
```

`id`는 로그에서 작업자를 구분하는 번호이고, `thread`는 나중에 종료를 기다릴 때(2.9절) 쓸 `JoinHandle`입니다. 스레드 반복문은 `()`를 돌려주므로 `JoinHandle<()>`입니다. `Worker`는 **비공개**입니다. 서버(`main.rs`)는 `ThreadPool`만 알면 되고, 작업자가 어떻게 생겼는지는 스레드 풀 안의 사정이기 때문입니다(ch7의 캡슐화). `ThreadPool::new`는 `Vec::with_capacity(size)`로 미리 공간을 잡은 벡터에 작업자를 `size`개 만들어 넣습니다.

#### 채널: 작업을 나르는 대기열

`execute`가 받은 작업을 작업자에게 전달하는 데는 ch15의 **채널**을 씁니다. `ThreadPool`이 송신자(`Sender`)를, 작업자들이 수신자(`Receiver`)를 가집니다. 채널 안에 쌓인 작업이 곧 2.5절의 대기열입니다.

처음에는 수신자를 작업자마다 그대로 넘기려 할 것입니다.

```rust
let (sender, receiver) = mpsc::channel();
for id in 0..size {
    workers.push(Worker::new(id, receiver)); // E0382
}
```

반복문의 첫 바퀴에서 `receiver`가 첫 번째 작업자에게 **이동**하므로 두 번째 바퀴부터는 넘길 값이 없습니다(E0382, 4.1절). `mpsc`는 "생산자 여럿, 소비자 하나"라서 `Receiver`는 `clone`할 수도 없습니다. 필요한 것은 두 가지입니다.

- **여러 작업자가 같은 수신자를 함께 소유**해야 합니다 → `Arc<T>`
- **한 번에 한 작업자만 수신자에서 작업을 꺼내야** 합니다. `Receiver`는 여러 스레드가 동시에 쓰도록 만들어지지 않았기 때문입니다(아래에서 확인합니다) → `Mutex<T>`

ch15에서 공유 카운터를 만들 때와 똑같은 `Arc<Mutex<T>>` 조합입니다.

```rust
let (sender, receiver) = mpsc::channel();
let receiver = Arc::new(Mutex::new(receiver));

let mut workers = Vec::with_capacity(size);
for id in 0..size {
    workers.push(Worker::new(id, Arc::clone(&receiver)));
}
```

`Mutex` 없이 `Arc<Receiver<Job>>`만 넘기면 어떨까요? 이 환경에서 확인하니 `` `Receiver<..>` cannot be shared between threads safely``(E0277, `` the trait `Sync` is not implemented``)가 났습니다. `Receiver`는 `Sync`가 아니어서, 여러 스레드가 `&Receiver`로 동시에 `recv`를 부르는 것을 타입 수준에서 막습니다. `Mutex`가 "한 번에 하나"를 보장해야 비로소 함께 쓸 수 있습니다(ch15 2.9절의 표).

#### 작업 타입 `Job`

채널로 보내는 값의 타입은 무엇일까요? `execute`가 받는 클로저는 부를 때마다 타입이 다릅니다. 클로저는 저마다 고유한 타입을 가지기 때문입니다(ch13). 그런데 채널 하나는 한 가지 타입만 나를 수 있습니다. 그래서 "어떤 타입이든 `FnOnce() + Send + 'static`을 구현한 것"을 하나의 타입으로 다루는 **트레이트 객체**가 필요합니다.

```rust
type Job = Box<dyn FnOnce() + Send + 'static>;
```

| 부분 | 왜 필요한가 |
| --- | --- |
| `dyn FnOnce()` | 서로 다른 클로저 타입을 하나의 타입으로 다루는 트레이트 객체([ch17](../ch17-trait-objects-oop/README.md)). 실제로 어떤 클로저인지는 실행 중에 정해집니다. |
| `Box<..>` | 트레이트 객체는 크기를 컴파일할 때 알 수 없으므로, 크기가 정해진 포인터 뒤에 둡니다(ch14의 `Box`, ch12의 `Box<dyn Error>`와 같은 이유). |
| `+ Send` | 이 상자를 꺼낼 수 있는 수신자를 작업자 스레드로 보내야 하므로 필요합니다. 빠뜨리면 E0277(4.2절)이 납니다. |
| `+ 'static` | 작업은 채널 안에서 언제까지든 기다릴 수 있으므로, 빌린 참조를 품으면 안 됩니다. |
| `type Job = ..` | 긴 타입에 이름을 붙이는 별칭(type alias, [ch19](../ch19-advanced-features/README.md))입니다. `Sender<Job>`, `Receiver<Job>`처럼 여러 곳에 쓰는 타입을 짧게 씁니다. 새 타입을 만드는 것은 아닙니다. |

`execute`는 받은 클로저를 상자에 담아 보냅니다.

```rust
pub fn execute<F>(&self, f: F)
where
    F: FnOnce() + Send + 'static,
{
    let job = Box::new(f);
    self.sender.send(job).unwrap();
}
```

`Box::new(f)`의 타입은 `Box<F>`이지만, `Sender<Job>`의 `send`에 넘기는 순간 `Box<dyn FnOnce() + Send + 'static>`로 바뀝니다. 앞에서 본 E0277 메시지의 "required for the cast from `Box<F>` to `Box<..>`"가 바로 이 변환을 가리킵니다. 그래서 `F`가 `Send`가 아니면 이 변환이 불가능합니다.

작업자 쪽에서는 꺼낸 `Box<dyn FnOnce()>`를 `job()`처럼 바로 부를 수 있습니다. 완성판의 작업자는 이렇게 생겼습니다.

```rust
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("작업자 {id}: 작업을 받아 실행합니다");
                        job();
                    }
                    Err(_) => {
                        println!("작업자 {id}: 채널이 닫혀 종료합니다");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}
```

`lock()`으로 잠금을 얻고, `recv()`로 작업이 올 때까지 기다렸다가 꺼냅니다. `recv()`가 `Err`를 돌려주는 경우(채널이 닫힘)는 2.9절에서 씁니다. 그런데 이 코드에서 `let message = ...;`를 따로 쓴 데는 중요한 이유가 있습니다. 다음 절에서 봅니다.

### 2.8 잠금은 언제 풀리는가: `let`과 `while let`

The Book Listing 21-21은 작업자 반복문을 이렇게도 쓸 수 있지 않느냐고 묻습니다. 더 짧고 깔끔해 보입니다.

```rust
while let Ok(job) = receiver.lock().unwrap().recv() {
    job();
}
```

컴파일도 되고, 서버도 정상적으로 응답합니다. 하지만 **느린 요청이 다른 요청을 다시 막습니다.** 이유는 `lock()`이 돌려준 `MutexGuard`가 언제 버려지느냐에 있습니다.

`receiver.lock().unwrap()`의 결과인 `MutexGuard`는 변수에 담지 않은 **임시값(temporary)** 입니다. 임시값이 버려지는 시점은 그 임시값을 만든 문장(구문)이 어디서 끝나느냐로 정해집니다.

- **`let` 문**: 문장이 세미콜론에서 끝나므로, 임시값도 **그 줄의 끝에서** 버려집니다. `let message = receiver.lock().unwrap().recv();` 다음 줄에서는 이미 잠금이 풀려 있습니다.
- **`while let`, `if let`, `match`**: 조사 대상(오른쪽 표현식)의 임시값은 **본문이 끝날 때까지** 살아 있습니다(`while let`은 반복 한 바퀴의 본문, `match`는 갈래 전체, `if let`은 then 블록). `job()`을 실행하는 동안에도 `MutexGuard`가 살아 있어 잠금을 쥐고 있고, 다른 작업자들은 모두 `lock()`에서 기다립니다.

[`04_lock_scope.rs`](examples/04_lock_scope.rs)는 스레드 풀의 핵심만 떼어 와서(작업자 2개, 0.2초짜리 작업 4개) 세 가지 반복문을 비교합니다.

```text
1) let 문으로 받기
    동시에 실행된 작업 수(최대): 2, 걸린 시간: 0.4초
2) while let으로 받기
    동시에 실행된 작업 수(최대): 1, 걸린 시간: 0.8초
3) match의 조사 대상에서 바로 받기
    동시에 실행된 작업 수(최대): 1, 걸린 시간: 0.8초
```

`while let`과 `match`로 쓴 반복문은 작업자가 둘인데도 작업을 **하나씩만** 실행했습니다. 스레드 풀을 만든 의미가 사라진 것입니다. 이 버그는 컴파일러도, 정상적인 응답도 알려 주지 않으므로 특히 조심해야 합니다.

- Rust 2024 에디션은 `if let`에서 임시값이 버려지는 시점을 바꿨지만, 바뀐 것은 `else` 블록 쪽입니다. 이 환경(2024 에디션)에서 확인한 결과, `while let`, `match`, `if let`의 본문에서는 여전히 잠금이 유지되었습니다. 2021 에디션으로 컴파일해도 결과가 같았습니다. 에디션별 차이는 [Part 2 ch4](../../part2-editions/ch4-rust-2024-language/README.md)에서 다룹니다.
- clippy는 3번처럼 `loop` 안에 `match`와 `break`만 있는 모양을 보면 `while let`으로 바꾸라고 제안합니다(`clippy::while_let_loop`). 그렇게 바꿔도 2번과 같아질 뿐 잠금 문제는 그대로입니다. 1번처럼 `let`으로 받아 두는 모양에는 이 제안이 나오지 않습니다. 예제는 비교를 위해 3번에 `#[expect(clippy::while_let_loop)]`를 붙여 두었습니다.
- The Book의 Listing 21-20은 `let job = receiver.lock().unwrap().recv().unwrap();`처럼 `let` 문으로 받습니다. 잠금 문제는 없지만, 채널이 닫히면 `recv()`의 `Err`를 `unwrap`해 패닉합니다. 완성판은 2.9절처럼 `match`로 `Err`를 처리합니다.

**정리하면, 잠금을 짧게 쥐어야 할 때는 `MutexGuard`가 만들어지는 표현식을 `let` 문 하나로 끝내고, 필요한 값만 꺼내 다음 줄로 가져갑니다.**

### 2.9 우아한 종료: `Drop`으로 정리하기

#### 정리하지 않으면 생기는 일

ch15에서 보았듯이 **`main`이 끝나면 다른 스레드도 함께 끝납니다.** 서버가 정해진 수의 연결을 받은 뒤 `main`을 끝내 버리면, 작업자가 처리하던 요청은 응답을 보내기도 전에 잘립니다. 이 환경에서 `ThreadPool`의 `Drop` 구현을 지운 서버를 `-- 1`(연결 1개)로 실행하고 `/sleep`을 요청하자, 서버는 곧바로 끝났고 `curl`은 응답을 하나도 받지 못했습니다.

```text
curl: (52) Empty reply from server
```

우아한 종료란 **새 요청은 더 받지 않되, 이미 받은 요청은 끝까지 처리한 뒤** 멈추는 것입니다. `ThreadPool`이 버려질 때 모든 작업자가 일을 마치기를 기다리면 됩니다. "버려질 때 할 일"은 ch14의 `Drop`이 맡습니다.

#### 첫 시도: `join`하기 — E0507

```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("작업자 {}의 종료를 기다립니다", worker.id);
            worker.thread.join().unwrap(); // E0507
        }
    }
}
```

`JoinHandle::join(self)`은 핸들의 **소유권을 가져갑니다.** 그런데 `drop`은 `&mut self`만 받으므로 `worker`도 `&mut Worker`이고, 빌린 값에서 필드를 꺼내 옮길 수는 없습니다(E0507, 4.4절). 해결 방법은 두 가지입니다.

- **`Vec::drain(..)`**: 벡터에서 원소를 빼내면서 **값으로** 내주는 반복자를 돌려줍니다. 범위 `..`는 전부를 뜻하므로 벡터가 비워지고, 우리는 `Worker`를 통째로 소유하게 됩니다. The Book Listing 21-23과 이 저장소의 완성판이 이 방법을 씁니다.
- **`Option<JoinHandle<()>>`와 `take()`**: `thread` 필드를 `Option`으로 감싸 두고, `worker.thread.take()`로 `Some` 안의 값을 꺼내며 그 자리에 `None`을 남깁니다. The Book 21.3절의 마지막 전체 코드는 이 방법으로 쓰여 있습니다. 다만 The Book은 늘 들어 있는 값을 이런 우회 때문에 `Option`으로 감싸면 그 필드를 쓰는 모든 곳에서 `Option`을 다뤄야 하므로, 더 나은 방법(`drain`)을 찾아보라고 권합니다.

#### 두 번째 문제: 작업자가 끝나지 않는다

`drain`으로 컴파일은 되지만, 이 상태로는 서버가 **영원히 끝나지 않습니다.** 작업자의 반복문은 `recv()`로 다음 작업을 기다리는데, `recv()`가 `Err`를 돌려주는 것은 ch15에서 배웠듯이 **송신자가 모두 사라졌을 때**뿐입니다. 송신자는 `ThreadPool`이 들고 있고, `ThreadPool`의 필드는 `drop` 메서드가 **끝난 뒤에야** 버려집니다. `drop` 안의 `join`은 작업자가 끝나기를, 작업자는 송신자가 사라지기를 서로 기다리는 셈입니다.

이 환경에서 송신자를 버리는 줄만 지우고 `-- 1`로 실행해 `/`를 요청하자, 응답은 정상적으로 나갔지만 `작업자 0의 종료를 기다립니다` 뒤로 `채널이 닫혀 종료합니다`도, 다음 작업자를 기다리는 줄도 나오지 않았습니다. 3초가 지나도 끝나지 않아 프로세스를 직접 종료해야 했습니다.

해결책은 **`join`보다 먼저 송신자를 버려 채널을 닫는 것**입니다. 그런데 `&mut self`로는 `self.sender`를 꺼내 버릴 수 없으므로(E0507과 같은 이유), 이번에는 정말로 `Option`이 필요합니다.

```rust
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take()); // ① 채널을 닫습니다.

        for worker in self.workers.drain(..) { // ② 작업자가 끝나기를 기다립니다.
            println!("작업자 {}의 종료를 기다립니다", worker.id);
            worker.thread.join().unwrap();
        }
    }
}
```

- `Option::take()`는 `Some` 안의 값을 꺼내 돌려주고 그 자리에 `None`을 남깁니다. 꺼낸 송신자를 `drop`(ch14의 `std::mem::drop`)에 넘기면 채널이 닫힙니다.
- `execute`에서는 `self.sender.as_ref().unwrap().send(job)`으로 씁니다. `as_ref()`는 `&Option<Sender>`를 `Option<&Sender>`로 바꿔 송신자를 꺼내지 않고 빌려 씁니다. `sender`가 `None`이 되는 것은 `drop` 안에서뿐이고, `drop`된 풀의 `execute`를 부를 방법은 없으므로 이 `unwrap`은 실패하지 않습니다.
- **순서가 중요합니다.** ①과 ②를 바꾸면 위의 "영원히 기다리는" 상태로 돌아갑니다.

채널이 닫히면 작업자의 `recv()`가 `Err`를 돌려주고, 작업자는 `채널이 닫혀 종료합니다`를 출력하고 반복문을 빠져나갑니다. 여기서 ch15 2.5절의 표를 다시 떠올려 봅니다. `recv()`가 `Err`를 돌려주는 조건은 "송신자가 모두 사라지고 **남은 메시지도 없을 때**"입니다. 즉 채널을 닫아도 **대기열에 남아 있던 작업은 모두 꺼내 실행한 뒤에** 작업자가 끝납니다. 이것이 "이미 받은 요청은 끝까지 처리한다"를 보장합니다. 3절 프로젝트의 두 번째 실행에서, 서버가 종료를 선언한 뒤에도 대기열의 다섯 번째 요청이 처리되는 것을 볼 수 있습니다.

#### 서버 쪽

`main.rs`는 정해진 수의 연결만 받고 반복문을 끝냅니다. `main`이 끝나면서 `pool`이 버려지고, 위의 `Drop`이 실행됩니다.

```rust
fn main() {
    // ... (연결 수 읽기, bind)
    let pool = ThreadPool::new(POOL_SIZE);

    for stream in listener.incoming().take(max_connections) {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("연결 {max_connections}개를 받았으므로 서버를 종료합니다.");
} // ← 여기서 pool이 버려지며 ThreadPool의 Drop이 실행됩니다.
```

The Book은 `take(2)`로 고정했지만, 이 저장소의 완성판은 첫 번째 명령줄 인자로 연결 수를 바꿀 수 있게 했습니다(`cargo run -- 10`, 기본값 3). 브라우저로 접속해 볼 때 연결 수를 넉넉히 주기 위해서입니다.

남은 약점이 하나 있습니다. 작업 하나가 패닉하면 그 작업자 스레드는 끝나 버리고, `Drop`의 `join()`이 그 패닉을 `Err`로 돌려줘 `unwrap()`이 **다시 패닉**합니다. The Book은 `drop`이 패닉 도중에도 불릴 수 있어서 이 `unwrap`이 이중 패닉(double panic)을 일으켜 정리 작업을 중단시킬 수 있고, 예제로는 괜찮지만 실제 코드에는 권하지 않는다고 적고 있습니다. 실제로 어떻게 되는지는 4.7절에서 보고, 고치는 것은 연습 문제 1로 남깁니다.

### 2.10 패닉할까, `Result`를 돌려줄까: `new`와 `build`

크기 0인 스레드 풀은 작업을 실행할 수 없습니다. The Book의 `new`는 `assert!`로 이를 막고, 문서 주석에 `# Panics` 절을 둬서 알립니다.

```rust
/// 작업자 스레드 `size`개로 이뤄진 스레드 풀을 만듭니다.
///
/// # Panics
///
/// `size`가 0이면 패닉합니다. ...
pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0, "스레드 풀의 크기는 1 이상이어야 합니다");
    // ...
}
```

ch9의 "패닉과 `Result`의 선택 기준"을 떠올려 봅니다. 호출하는 쪽이 계약을 어긴 것, 즉 **프로그래머의 실수**라면 패닉이 알맞고, 실패가 **예상할 수 있는 일**이고 호출하는 쪽이 대처를 정해야 한다면 `Result`가 알맞습니다. 서버의 `main`은 크기로 상수 4를 넘기므로 0이 들어갈 일이 없습니다. 0이 들어갔다면 코드가 틀린 것이니 패닉으로 바로 알리는 편이 낫습니다.

하지만 크기를 설정 파일이나 명령줄 인자처럼 **프로그램 밖에서** 받는다면 0은 사용자의 입력 실수이고, 패닉 대신 친절한 에러 메시지를 내야 합니다. The Book은 ch12의 `Config::build`처럼 `Result`를 돌려주는 `build`를 만들어 보라고 권합니다. 완성판에 넣은 `build`는 이렇습니다.

```rust
#[derive(Debug, PartialEq)]
pub enum PoolCreationError {
    ZeroSize,
}

pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
    if size == 0 {
        return Err(PoolCreationError::ZeroSize);
    }
    Ok(ThreadPool::new(size))
}
```

검사를 통과한 뒤 `new`를 부르므로 `new`의 `assert!`는 절대 실패하지 않고, 스레드를 만드는 코드는 `new` 한 곳에만 있습니다. `PoolCreationError`에는 `Display`를 구현해(ch10) `"스레드 풀의 크기는 1 이상이어야 합니다"`라는 메시지를 낼 수 있게 했습니다. 에러 타입에 `std::error::Error` 트레이트까지 구현하는 방법은 [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)에서 다룹니다.

스레드를 만드는 일 자체도 실패할 수 있습니다. `thread::spawn`은 운영체제가 스레드를 만들지 못하면 패닉합니다. 표준 라이브러리 문서는 이 실패에서 회복하고 싶다면, 같은 일을 하되 `io::Result<JoinHandle<T>>`를 돌려주는 `std::thread::Builder`의 `spawn`을 쓰라고 안내합니다(연습 문제 3).

### 2.11 실행 위치에 따라 깨지지 않는 파일 경로

The Book의 서버는 HTML 파일을 `fs::read_to_string("hello.html")`로 읽습니다. 이런 **상대 경로는 프로그램 파일이나 소스 파일이 있는 곳이 아니라, 프로그램을 실행한 현재 디렉터리를 기준으로** 찾습니다. 프로젝트 디렉터리에서 `cargo run`하면 잘 동작하지만, 다른 디렉터리에서 실행하면 파일을 찾지 못합니다. 이 저장소처럼 프로젝트가 깊은 곳에 있으면 저장소 루트에서 `cargo run --manifest-path ...`로 실행하기 쉬운데, 그때 이 문제가 생깁니다(4.7절에서 실제 결과를 봅니다).

완성판은 경로를 이렇게 만듭니다.

```rust
fn page_path(filename: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(filename)
}
```

- `env!`는 **컴파일할 때** 환경 변수를 읽어 문자열 리터럴로 바꾸는 매크로입니다(ch12의 `env::var`는 실행할 때 읽습니다). `CARGO_MANIFEST_DIR`은 Cargo가 컴파일할 때 알려 주는, 이 패키지의 `Cargo.toml`이 있는 디렉터리의 절대 경로입니다. Cargo가 설정하는 환경 변수는 Part 3 `ch7-configuration-env`에서 다룹니다.
- `Path::join`은 경로 구분자(`/` 또는 `\`)를 운영체제에 맞게 붙여 줍니다.
- 그래서 저장소 루트에서 `--manifest-path`로 실행하든, `cd /`한 뒤 `target/debug/hello-server`를 직접 실행하든 같은 파일을 찾습니다. 이 환경에서 두 경우 모두 확인했습니다.
- 한계도 있습니다. 컴파일할 때의 절대 경로가 실행 파일에 박히므로, 프로젝트 디렉터리를 옮기거나 실행 파일만 다른 컴퓨터로 가져가면 다시 찾지 못합니다.

다른 방법으로 `include_str!("../hello.html")`이 있습니다. 컴파일할 때 파일 **내용**을 실행 파일 안에 문자열로 넣는 매크로로, 경로는 소스 파일을 기준으로 합니다. 실행할 때 파일을 읽지 않으므로 어디서 실행해도 동작하지만, HTML을 고칠 때마다 다시 컴파일해야 합니다. 이 챕터는 The Book처럼 "요청이 올 때마다 파일을 읽는" 서버를 유지하려고 `env!` 방식을 골랐습니다.

### 2.12 이 챕터에서 다루지 않는 것

| 주제 | 한 줄 소개 | 다루는 곳 |
| --- | --- | --- |
| HTTP의 나머지 | 헤더 해석, `Content-Type`, 연결 재사용(keep-alive), `POST` 본문 등. 실제 서비스에는 crates.io의 HTTP 크레이트를 씀 | 이 저장소에서는 다루지 않음 |
| TCP·UDP 소켓 자세히 | 클라이언트 쪽 `TcpStream`, 타임아웃, UDP 송수신 | [Part 1 ch12](../../part1-standard-lib/ch12-net/README.md) |
| `thread::Builder`, `Condvar`, 원자 타입 | 스레드 이름과 생성 실패 처리, 조건을 기다리는 동기화, `Mutex` 없는 카운터 | [Part 1 ch11](../../part1-standard-lib/ch11-threads-sync/README.md), 연습 문제 3 |
| `Ctrl+C`로 우아하게 종료하기 | 시그널을 받아 반복문을 끝내고 `Drop`이 실행되게 하기 | Part 7 `ch6-signals-config-exit-codes` |
| 사용자 정의 에러 타입 | `std::error::Error` 구현, 에러 변환 | [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) |
| 비동기 I/O 서버 | 스레드 대신 태스크로 수많은 연결을 처리 | [ch16](../ch16-async/README.md), 비동기 런타임 크레이트 |

## 3. 예제 안내

`examples/`의 예제는 모두 저장소 루트에서 실행합니다. 각 예제는 서버와 함께 **클라이언트 역할의 스레드**를 띄워 `127.0.0.1`의 서버에 스스로 접속하고, 정해진 수의 연결을 처리하면 몇 초 안에 끝납니다. 브라우저나 `curl`이 필요 없습니다. 포트는 운영체제가 고르므로 출력의 포트 번호는 실행할 때마다 다릅니다.

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`01_accept_and_read.rs`](examples/01_accept_and_read.rs) | 포트 0으로 바인드, `incoming().take(n)`, 요청을 빈 줄까지 줄 단위로 읽기 | 1 |
| [`02_respond.rs`](examples/02_respond.rs) | 상태 줄·`Content-Length`·본문으로 응답 쓰기, 요청 줄에 따라 200/404 | 1 |
| [`03_single_thread_limit.rs`](examples/03_single_thread_limit.rs) | 느린 요청이 뒤 요청을 막는 단일 스레드 서버, 연결마다 스레드를 만드는 서버 | 2 |
| [`04_lock_scope.rs`](examples/04_lock_scope.rs) | `let`/`while let`/`match`에 따라 잠금이 풀리는 시점과 동시 실행 수 | 3, 4 |
| [`projects/hello-server`](projects/hello-server) | 스레드 풀 서버 완성판, `ThreadPool` 테스트, 우아한 종료 | 2, 3, 5 |
| [`compile_fail/`](compile_fail/) | 스레드 풀을 설계하며 만나는 컴파일 에러 4개(4절) | 3, 5 |

### 01_accept_and_read — 연결 받기와 요청 읽기

```sh
cargo run -p p0-ch20-web-server-project --example 01_accept_and_read
```

```text
127.0.0.1:64616에서 연결을 기다립니다
클라이언트가 보낼 요청: "GET / HTTP/1.1\r\nHost: 127.0.0.1:64616\r\nAccept: */*\r\n\r\n"
클라이언트가 보낼 요청: "GET /favicon.ico HTTP/1.1\r\nHost: 127.0.0.1:64616\r\nAccept: */*\r\n\r\n"
연결 1번이 성립했습니다
Request: [
    "GET / HTTP/1.1",
    "Host: 127.0.0.1:64616",
    "Accept: */*",
]
연결 2번이 성립했습니다
Request: [
    "GET /favicon.ico HTTP/1.1",
    "Host: 127.0.0.1:64616",
    "Accept: */*",
]
연결 두 개를 처리했으므로 서버를 끝냅니다
```

- `64616`은 운영체제가 고른 포트로, 실행할 때마다 다릅니다. 포트 번호를 빼면 출력은 항상 같습니다.
- 클라이언트가 보낸 요청은 `{:?}`로 출력해 `\r\n`이 보이게 했습니다. 서버가 `lines()`로 읽은 줄에서는 `\r\n`이 사라지고, 마지막 빈 줄 앞에서 `take_while`이 멈췄습니다.
- 두 번째 요청의 `/favicon.ico`는 브라우저가 탭에 띄울 작은 아이콘을 받으려고 따로 보내곤 하는 요청을 흉내 낸 것입니다. 실제 브라우저의 요청에는 `User-Agent` 등 헤더가 훨씬 많습니다.
- 클라이언트는 응답을 기다리지 않고 연결을 닫으므로 이 서버는 아직 응답하지 않습니다. 응답은 02에서 씁니다.

### 02_respond — 응답 쓰기와 요청에 따른 응답

```sh
cargo run -p p0-ch20-web-server-project --example 02_respond
```

```text
[서버] GET / HTTP/1.1 → HTTP/1.1 200 OK
[서버] GET /nothing HTTP/1.1 → HTTP/1.1 404 NOT FOUND
--- 클라이언트가 받은 응답 ---
헤더 부분: "HTTP/1.1 200 OK\r\nContent-Length: 136"
본문:
<!DOCTYPE html>
<html lang="ko"><head><meta charset="utf-8"><title>안녕!</title></head>
<body><h1>안녕하세요!</h1></body></html>
본문의 바이트 수: 136, 글자 수: 122
--- 클라이언트가 받은 응답 ---
헤더 부분: "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 126"
본문:
<!DOCTYPE html>
<html lang="ko"><head><meta charset="utf-8"><title>없음</title></head>
<body><h1>이런!</h1></body></html>
본문의 바이트 수: 126, 글자 수: 118
```

- 클라이언트는 요청을 보낸 뒤 `read_to_string`으로 응답을 끝까지 읽습니다. 서버가 응답을 쓰고 스트림을 drop하면 연결이 닫히고, 그때 `read_to_string`이 끝납니다.
- 클라이언트 스레드는 받은 응답을 출력하지 않고 돌려주며, `main`이 서버 출력이 끝난 뒤에 출력합니다. 그래서 줄 순서가 항상 위와 같습니다.
- `Content-Length`가 본문의 바이트 수와 같다는 것, 그리고 한글 한 글자가 UTF-8에서 3바이트라서 글자 수보다 크다는 것을 확인할 수 있습니다(200 응답의 한글 7글자 × 2바이트 = 14바이트 차이).
- 응답의 헤더와 본문은 `\r\n\r\n`(빈 줄)로 나뉩니다. 예제는 `split_once("\r\n\r\n")`로 둘을 나눠 출력했습니다.

### 03_single_thread_limit — 단일 스레드의 한계

```sh
cargo run -p p0-ch20-web-server-project --example 03_single_thread_limit
```

```text
1) 연결을 하나씩 처리하는 서버
    0.0초  A가 /sleep 요청
    0.1초  B가 / 요청
    1.0초  A가 응답 받음 (HTTP/1.1 200 OK)
    1.0초  B가 응답 받음 (HTTP/1.1 200 OK)
2) 연결마다 스레드를 만드는 서버
    0.0초  A가 /sleep 요청
    0.1초  B가 / 요청
    0.1초  B가 응답 받음 (HTTP/1.1 200 OK)
    1.0초  A가 응답 받음 (HTTP/1.1 200 OK)
```

- 시각은 서버를 띄운 순간부터 잰 것이고 0.1초 단위로 반올림했습니다. 이 환경에서 여러 번 실행해도 위와 같았지만, 컴퓨터가 바쁘면 0.1초쯤 다르게 나올 수 있습니다.
- 두 클라이언트 스레드가 기록한 사건을 `main`이 모아 시각순으로 정렬해 출력하므로 줄 순서가 섞이지 않습니다.
- 두 서버는 `fn(TcpListener)` 타입의 함수 포인터로 `experiment`에 넘겨, 같은 실험을 두 번 합니다.
- 2번 서버는 연결 2개만 받으므로 스레드도 2개만 만들지만, `take(2)`가 없다면 연결 수만큼 스레드를 만듭니다. 이것이 2.5절에서 말한 상한 없는 스레드 생성입니다.

### 04_lock_scope — 잠금이 풀리는 시점

```sh
cargo run -p p0-ch20-web-server-project --example 04_lock_scope
```

```text
1) let 문으로 받기
    동시에 실행된 작업 수(최대): 2, 걸린 시간: 0.4초
2) while let으로 받기
    동시에 실행된 작업 수(최대): 1, 걸린 시간: 0.8초
3) match의 조사 대상에서 바로 받기
    동시에 실행된 작업 수(최대): 1, 걸린 시간: 0.8초
```

- 네트워크를 쓰지 않습니다. 작업자 스레드 2개와 0.2초짜리 작업 4개로, 스레드 풀의 작업자 반복문만 비교합니다.
- "동시에 실행된 작업 수(최대)"는 작업이 시작할 때 늘리고 끝날 때 줄이는 공유 카운터(`Arc<Mutex<(usize, usize)>>`)로 셉니다. 1번에서는 두 작업자가 동시에 일해 2, 나머지는 1입니다.
- 걸린 시간은 0.1초 단위로 반올림했습니다. 이 환경에서는 매번 위와 같았지만 컴퓨터가 바쁘면 조금 늘어날 수 있습니다. 동시 실행 수가 1이면 시간이 두 배 걸린다는 관계는 변하지 않습니다.

### projects/hello-server — 완성판

워크스페이스와 분리된 독립 프로젝트이므로 그 디렉터리에서 실행합니다.

```sh
cd part0-getting-started/ch20-web-server-project/projects/hello-server
```

| 파일 | 내용 |
| --- | --- |
| [`src/lib.rs`](projects/hello-server/src/lib.rs) | `ThreadPool`(`new`, `build`, `execute`, `Drop`), `Worker`, `Job`, `PoolCreationError`, 테스트 |
| [`src/main.rs`](projects/hello-server/src/main.rs) | 서버: 연결 받기, 요청 처리(`handle_connection`), 파일 경로(`page_path`) |
| [`hello.html`](projects/hello-server/hello.html), [`404.html`](projects/hello-server/404.html) | 돌려줄 페이지 |

The Book의 최종 코드와 다른 점은 다음과 같습니다.

- `Drop`에서 `Option<JoinHandle>` 대신 `drain(..)`을 씁니다(2.9절).
- `ThreadPool::build`와 `PoolCreationError`를 더했습니다(2.10절).
- HTML 파일 경로를 `CARGO_MANIFEST_DIR` 기준으로 만듭니다(2.11절).
- 처리할 연결 수를 명령줄 인자로 바꿀 수 있습니다(기본 3, The Book은 2).
- 요청 줄 없이 닫히는 연결(포트가 열려 있는지만 확인하는 `nc -z` 등)에서 패닉하지 않도록, 요청 줄을 `let Some(Ok(request_line)) = ... else { return; }`로 읽습니다. The Book처럼 `unwrap`하면 작업자 스레드가 패닉합니다.
- 7878 포트를 쓸 수 없으면 패닉 대신 에러 메시지를 출력하고 종료 코드 1로 끝납니다(4.8절).
- 로그를 한국어로 출력하고, 응답을 다 쓴 뒤 `응답 완료:` 줄을 남깁니다.

**테스트**: `ThreadPool`의 단위 테스트 7개와 문서 테스트 1개입니다.

```console
$ cargo test
```

```text
     Running unittests src/lib.rs (target/debug/deps/hello_server-67c09c5672a308f4)

running 7 tests
test tests::build_returns_err_on_zero_size ... ok
test tests::new_panics_on_zero_size - should panic ... ok
test tests::build_returns_ok_on_positive_size ... ok
test tests::new_creates_requested_number_of_workers ... ok
test tests::drop_waits_for_all_queued_jobs ... ok
test tests::jobs_run_in_parallel ... ok
test tests::execute_does_not_wait_for_job ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s

     Running unittests src/main.rs (target/debug/deps/hello_server-d51bb08fae919dd2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests hello_server

running 1 test
test src/lib.rs - ThreadPool (line 17) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

all doctests ran in 0.39s; merged doctests compilation took 0.14s
```

(맨 앞의 `Compiling`, `Finished` 줄은 뺐습니다.) 테스트는 여러 스레드에서 동시에 실행되므로 `test ... ok` 줄의 순서와 걸린 시간은 실행할 때마다 다릅니다. `target/debug/deps/` 뒤의 해시도 환경마다 다릅니다. 작업자가 출력하는 로그는 테스트 러너가 가로채므로 보이지 않습니다.

| 테스트 | 확인하는 것 |
| --- | --- |
| `new_creates_requested_number_of_workers` | 작업자가 요청한 수만큼 생김. 테스트 모듈은 같은 파일 안이라 비공개 필드 `workers`를 볼 수 있음(ch11) |
| `new_panics_on_zero_size` | `new(0)`이 정해진 메시지로 패닉함(`#[should_panic(expected = ..)]`) |
| `build_returns_err_on_zero_size`, `build_returns_ok_on_positive_size` | `build`가 0이면 `Err(ZeroSize)`, 아니면 `Ok` |
| `execute_does_not_wait_for_job` | `execute`는 0.3초짜리 작업을 맡기고도 0.1초 안에 돌아옴(채널에 넣기만 함) |
| `jobs_run_in_parallel` | 작업자 4개에 0.2초짜리 작업 4개를 맡기면 0.6초 안에 끝남(하나씩이면 0.8초 이상) |
| `drop_waits_for_all_queued_jobs` | 작업자 2개에 작업 8개를 맡기고 풀을 버리면, 대기열의 작업까지 8개가 모두 끝난 뒤에 `drop`이 돌아옴 |

시간을 재는 두 테스트는 넉넉한 여유를 두었지만, 컴퓨터가 매우 바쁘면 실패할 수 있습니다. 문서 테스트는 `ThreadPool`의 문서 주석에 넣은 사용 예입니다. `cargo doc --open`으로 생성된 문서에서 `ThreadPool`을 열면 이 예와 `# Panics`, `# Errors` 절을 볼 수 있습니다.

**실행 ①: 세 경로와 동시 처리, 종료**

터미널 두 개를 엽니다. 첫 번째 터미널에서 서버를 띄웁니다.

```console
$ cargo run
```

두 번째 터미널에서 다음을 차례로 입력합니다. 첫 줄은 `/sleep`을 백그라운드(`&`)로 요청하고, 0.3초 뒤 `/`와 없는 경로를 요청합니다. 마지막 `wait`은 백그라운드 요청이 끝날 때까지 기다립니다.

```sh
curl -s -o /dev/null -w "/sleep: %{http_code}, %{time_total}초\n" http://127.0.0.1:7878/sleep &
sleep 0.3
curl -s -w "/: %{http_code}, %{time_total}초\n" http://127.0.0.1:7878/
curl -s -i http://127.0.0.1:7878/nothing
wait
```

두 번째 터미널의 출력입니다. (대화형 셸에서는 `&`로 띄운 작업의 번호와 완료 알림이 몇 줄 더 나올 수 있습니다.)

```text
<!DOCTYPE html>
<html lang="ko">
  <head>
    <meta charset="utf-8">
    <title>안녕하세요!</title>
  </head>
  <body>
    <h1>안녕하세요!</h1>
    <p>Rust로 만든 서버가 보낸 페이지입니다.</p>
  </body>
</html>
/: 200, 0.000761초
HTTP/1.1 404 NOT FOUND
Content-Length: 226

<!DOCTYPE html>
<html lang="ko">
  <head>
    <meta charset="utf-8">
    <title>찾을 수 없음</title>
  </head>
  <body>
    <h1>이런!</h1>
    <p>요청하신 페이지를 찾을 수 없습니다.</p>
  </body>
</html>
/sleep: 200, 5.002052초
```

첫 번째 터미널(서버)의 출력입니다. (`cargo run`이 출력하는 `Finished`, `Running` 줄은 뺐습니다.)

```text
http://127.0.0.1:7878 에서 기다립니다 (연결 3개를 처리하면 종료)
작업자 0: 작업을 받아 실행합니다
작업자 1: 작업을 받아 실행합니다
응답 완료: GET / HTTP/1.1 → HTTP/1.1 200 OK
연결 3개를 받았으므로 서버를 종료합니다.
작업자 0의 종료를 기다립니다
작업자 3: 채널이 닫혀 종료합니다
작업자 2: 작업을 받아 실행합니다
작업자 1: 채널이 닫혀 종료합니다
응답 완료: GET /nothing HTTP/1.1 → HTTP/1.1 404 NOT FOUND
작업자 2: 채널이 닫혀 종료합니다
응답 완료: GET /sleep HTTP/1.1 → HTTP/1.1 200 OK
작업자 0: 채널이 닫혀 종료합니다
작업자 1의 종료를 기다립니다
작업자 2의 종료를 기다립니다
작업자 3의 종료를 기다립니다
```

여러 스레드가 함께 출력하므로 **줄의 순서와 작업자 번호는 실행할 때마다 다릅니다.** 이 환경에서 여러 번 실행했을 때 변하지 않은 점은 다음과 같습니다.

- `/`는 `/sleep`이 처리되는 중에 요청했는데도 1밀리초 안에 응답했고, `응답 완료: GET /`가 `응답 완료: GET /sleep`보다 먼저 나옵니다. 느린 요청이 빠른 요청을 막지 않습니다.
- `연결 3개를 받았으므로 서버를 종료합니다.`는 세 번째 연결을 **받자마자** 나옵니다. 그 뒤에도 `/nothing`과 `/sleep`의 처리가 끝까지 진행되고, `/sleep`을 처리하던 작업자를 기다린 다음에야 서버가 끝납니다. 서버 전체는 약 5초 뒤 종료 코드 0으로 끝났습니다.
- `작업자 N의 종료를 기다립니다`는 `drain`이 벡터 순서대로 꺼내므로 항상 0, 1, 2, 3 순서입니다. 다른 줄은 그 사이사이에 섞입니다.
- 작업을 받지 않은 작업자도 채널이 닫히면 `채널이 닫혀 종료합니다`를 출력하고 끝납니다.

**실행 ②: 작업자보다 느린 요청이 많을 때**

연결 수를 5로 주고 서버를 띄운 뒤, `/sleep`을 다섯 개 동시에 요청합니다.

```console
$ cargo run -- 5
```

```sh
for n in 1 2 3 4 5; do
  curl -s -o /dev/null -w "요청 $n: %{http_code}, %{time_total}초\n" http://127.0.0.1:7878/sleep &
done; wait
```

```text
요청 2: 200, 5.001109초
요청 3: 200, 5.000947초
요청 1: 200, 5.001206초
요청 4: 200, 5.003488초
요청 5: 200, 10.000462초
```

```text
http://127.0.0.1:7878 에서 기다립니다 (연결 5개를 처리하면 종료)
작업자 2: 작업을 받아 실행합니다
작업자 0: 작업을 받아 실행합니다
작업자 1: 작업을 받아 실행합니다
작업자 3: 작업을 받아 실행합니다
연결 5개를 받았으므로 서버를 종료합니다.
작업자 0의 종료를 기다립니다
응답 완료: GET /sleep HTTP/1.1 → HTTP/1.1 200 OK
응답 완료: GET /sleep HTTP/1.1 → HTTP/1.1 200 OK
응답 완료: GET /sleep HTTP/1.1 → HTTP/1.1 200 OK
작업자 2: 작업을 받아 실행합니다
작업자 1: 채널이 닫혀 종료합니다
작업자 0: 채널이 닫혀 종료합니다
작업자 1의 종료를 기다립니다
작업자 2의 종료를 기다립니다
응답 완료: GET /sleep HTTP/1.1 → HTTP/1.1 200 OK
작업자 3: 채널이 닫혀 종료합니다
응답 완료: GET /sleep HTTP/1.1 → HTTP/1.1 200 OK
작업자 2: 채널이 닫혀 종료합니다
작업자 3의 종료를 기다립니다
```

- 작업자는 4개뿐이므로 네 요청은 5초에, 나머지 하나는 앞의 요청이 끝나 작업자가 비기를 기다렸다가 10초에 응답을 받았습니다. 스레드 수가 늘지 않는다는 것이 스레드 풀의 상한입니다. 어느 요청이 10초가 걸리는지(위에서는 `요청 5`)와 줄 순서는 실행마다 다릅니다.
- 서버는 다섯 번째 연결을 받자마자 종료를 선언하고 채널을 닫았습니다. 그런데도 5초 뒤 손이 빈 작업자 하나가 `작업을 받아 실행합니다`를 출력하고 대기열에 남아 있던 다섯 번째 작업을 처리했습니다. 채널을 닫아도 남은 메시지는 받을 수 있기 때문입니다(2.9절). 서버는 약 10초 뒤 종료 코드 0으로 끝났습니다.

**그 밖의 실행**

```console
$ cargo run -- abc
연결 수는 0 이상의 정수여야 합니다: abc
$ echo $?
1
```

`nc -z 127.0.0.1 7878`처럼 요청을 보내지 않고 닫히는 연결은 `요청 줄 없이 닫힌 연결을 건너뜁니다`를 출력하고 넘어갑니다. 이 연결도 처리한 연결 수에 들어갑니다.

요청을 보내지 않고 **열어만 둔** 연결이 있으면, 그 연결을 맡은 작업자는 상대가 연결을 닫을 때까지 요청 줄을 기다립니다. 이 환경에서 `nc`로 3초 동안 아무것도 보내지 않는 연결을 열어 둔 채 `-- 2`로 띄운 서버에 `/`를 요청하자, 서버는 종료를 선언한 뒤에도 그 연결이 닫힐 때까지(약 3초) 기다렸다가 끝났습니다. 우아한 종료는 "맡은 일을 끝까지 한다"는 뜻이므로, 끝나지 않는 일이 있으면 종료도 늦어집니다.

**브라우저로 보기**: 서버를 띄우고 브라우저에서 `http://127.0.0.1:7878`(또는 `/sleep`, `/아무거나`)을 엽니다. 브라우저는 페이지 말고도 `/favicon.ico`를 따로 요청하는 등 연결을 여러 개 쓸 수 있으므로 `cargo run -- 20`처럼 연결 수를 넉넉히 주세요. 브라우저가 요청 없이 연결을 열어 두면 위처럼 종료가 늦어질 수 있습니다. (브라우저에서의 동작은 이 환경에서 확인하지 않았습니다. 이 절의 결과는 모두 `curl`과 `nc`로 확인했습니다.)

실습이 끝나면 서버가 남아 있지 않은지 확인하세요. 서버는 정해진 수의 연결을 처리하면 스스로 끝나지만, 연결이 모자라 기다리는 중이라면 첫 번째 터미널에서 `Ctrl+C`로 끝낼 수 있습니다. 이때는 프로세스가 곧바로 끝나므로 `Drop`이 실행되지 않습니다(이 환경에서 `작업자 N의 종료를 기다립니다` 줄이 나오지 않는 것을 확인했습니다). 프로젝트의 `target/`은 `cargo clean`으로 지울 수 있습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드로, 스레드 풀을 설계하며 실제로 만나는 에러를 하나씩 담았습니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 코드를 고쳐 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part0-getting-started/ch20-web-server-project
rustc --edition 2024 compile_fail/e0382_receiver_moved_into_workers.rs --out-dir /tmp/rust-ch20
```

각 파일 맨 위의 주석에 고치는 방법을 적어 두었습니다. 이 환경에서 그 방법대로 고친 코드가 경고 없이 컴파일되는 것도 확인했습니다.

### 4.1 수신자를 작업자마다 그대로 넘김 — E0382

파일: [`compile_fail/e0382_receiver_moved_into_workers.rs`](compile_fail/e0382_receiver_moved_into_workers.rs)

```text
error[E0382]: use of moved value: `receiver`
  --> compile_fail/e0382_receiver_moved_into_workers.rs:27:42
   |
23 |         let (sender, receiver) = mpsc::channel();
   |                      -------- move occurs because `receiver` has type `std::sync::mpsc::Receiver<Box<dyn FnOnce() + Send>>`, which does not implement the `Copy` trait
...
26 |         for id in 0..size {
   |         ----------------- inside of this loop
27 |             workers.push(Worker::new(id, receiver));
   |                                          ^^^^^^^^ value moved here, in previous iteration of loop
   |
note: consider changing this parameter type in method `new` to borrow instead if owning the value isn't necessary
  --> compile_fail/e0382_receiver_moved_into_workers.rs:47:33
   |
47 |     fn new(id: usize, receiver: mpsc::Receiver<Job>) -> Worker {
   |        --- in this method       ^^^^^^^^^^^^^^^^^^^ this parameter takes ownership of the value
```

- **읽는 법**: "value moved here, in previous iteration of loop"가 핵심입니다. 반복문의 이전 바퀴에서 `receiver`가 `Worker::new`로 이동했습니다. 타입 이름의 `Box<dyn FnOnce() + Send>`는 별칭 `Job`을 풀어 쓴 것입니다(`'static`은 트레이트 객체의 기본값이라 생략되어 보입니다).
- **`note`를 그대로 따르면 안 되는 이유**: 컴파일러는 매개변수를 참조(`&Receiver<Job>`)로 바꾸라고 제안합니다. 하지만 작업자 스레드는 `'static`이어야 하므로(2.6절) 지역 변수를 빌린 참조를 스레드에 넘길 수 없습니다. 컴파일러의 제안은 일반적인 해결책이라, 상황에 맞는지 판단하는 것은 우리 몫입니다.
- **해결**: `Arc::new(Mutex::new(receiver))`로 감싸고 작업자마다 `Arc::clone(&receiver)`를 넘깁니다. `Worker::new`의 매개변수 타입은 `Arc<Mutex<mpsc::Receiver<Job>>>`가 됩니다(2.7절).

### 4.2 `Job`에 `Send`를 빠뜨림 — E0277

파일: [`compile_fail/e0277_job_without_send.rs`](compile_fail/e0277_job_without_send.rs)

```text
error[E0277]: `(dyn FnOnce() + 'static)` cannot be sent between threads safely
  --> compile_fail/e0277_job_without_send.rs:24:36
   |
24 |           let thread = thread::spawn(move || {
   |  ______________________-------------_^
   | |                      |
   | |                      required by a bound introduced by this call
...
32 | |         });
   | |_________^ `(dyn FnOnce() + 'static)` cannot be sent between threads safely
   |
   = help: the trait `Send` is not implemented for `(dyn FnOnce() + 'static)`
   = note: required for `std::ptr::Unique<(dyn FnOnce() + 'static)>` to implement `Send`
note: required because it appears within the type `Box<(dyn FnOnce() + 'static)>`
   = note: required for `std::sync::mpsc::Receiver<Box<(dyn FnOnce() + 'static)>>` to implement `Send`
   = note: required for `std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn FnOnce() + 'static)>>>` to implement `Sync`
   = note: 1 redundant requirement hidden
   = note: required for `Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn FnOnce() + 'static)>>>>` to implement `Send`
note: required because `receiver` is used within this closure
note: required by a bound in `spawn`
help: use parentheses to call this trait object
   |
24 ~         let thread = thread::spawn((move || {
...
32 ~         })());
```

(가운데의 소스 줄과 표준 라이브러리 경로 몇 줄은 생략했습니다.)

- **읽는 법**: 메시지가 길지만 `= note:` 줄을 위에서 아래로 읽으면 **이유의 사슬**이 보입니다. `dyn FnOnce()`가 `Send`가 아님 → 그것을 담은 `Box`가 `Send`가 아님 → 그 `Box`를 받는 `Receiver`가 `Send`가 아님 → 그것을 감싼 `Mutex`가 `Sync`가 아님 → 그 `Mutex`를 가리키는 `Arc`가 `Send`가 아님 → 그 `Arc`를 캡처한 클로저를 `spawn`에 넘길 수 없음. ch15 2.9절의 표에서 본 규칙(`Arc<T>`는 `T: Send + Sync`일 때만 `Send`, `Mutex<T>`는 `T: Send`일 때 `Sync`)이 한 줄씩 적용된 것입니다.
- **`help`를 따르면 안 되는 이유**: 마지막 `help`는 클로저 뒤에 `()`를 붙여 **그 자리에서 불러 버리라고** 제안합니다. 그러면 작업자의 반복문이 새 스레드가 아니라 지금 스레드에서 돌게 되어 전혀 다른 프로그램이 됩니다. 4.1의 `note`처럼, 컴파일러의 제안은 "컴파일이 되게 하는 한 가지 방법"이지 언제나 의도에 맞는 수정은 아닙니다.
- **막은 버그**: `Send`가 아닌 클로저(예: ch15에서 본 `Rc`를 캡처한 클로저)가 작업으로 들어와 다른 스레드에서 실행되면, `Rc`의 참조 카운트를 두 스레드가 동시에 고치는 일이 생길 수 있습니다.
- **해결**: `type Job = Box<dyn FnOnce() + Send + 'static>;`처럼 별칭에 `Send`를 넣습니다. 그러면 `execute`의 바운드에도 `Send`가 있어야 `Box<F>`를 `Job`으로 바꿀 수 있습니다(2.6절).

### 4.3 `execute`의 바운드에 `'static`을 빠뜨림 — E0310

파일: [`compile_fail/e0310_execute_without_static.rs`](compile_fail/e0310_execute_without_static.rs)

```text
error[E0310]: the parameter type `F` may not live long enough
  --> compile_fail/e0310_execute_without_static.rs:26:26
   |
26 |         self.sender.send(job).unwrap();
   |                          ^^^
   |                          |
   |                          the parameter type `F` must be valid for the static lifetime...
   |                          ...so that the type `F` will meet its required lifetime bounds
   |
help: consider adding an explicit lifetime bound
   |
23 |         F: FnOnce() + Send + 'static,
   |                            +++++++++
```

- **읽는 법**: "`F`가 충분히 오래 살지 않을 수 있다"는 뜻입니다. 채널로 보내는 `Job`은 `'static`이어야 하는데, `F`에 그런 약속이 없으니 지역 변수를 빌린 클로저가 들어올 수도 있다는 것입니다. 에러는 `send` 줄에서 나지만 고칠 곳은 바운드입니다.
- **막은 버그**: `execute`를 부른 함수가 끝나 지역 변수가 사라진 뒤에 작업자가 그 클로저를 실행하면, 이미 해제된 메모리를 읽게 됩니다(해제 후 사용).
- **해결**: 이번에는 `help`가 정확히 맞습니다. `F: FnOnce() + Send + 'static`으로 고칩니다. 그 결과 `execute`에 지역 변수를 빌리는 클로저를 넘기면, 이번에는 호출하는 쪽에서 E0373과 `move`를 붙이라는 제안이 나옵니다(2.6절).

### 4.4 `Drop`에서 `&mut`로 빌린 `JoinHandle`을 `join` — E0507

파일: [`compile_fail/e0507_join_in_drop.rs`](compile_fail/e0507_join_in_drop.rs)

```text
error[E0507]: cannot move out of `worker.thread` which is behind a mutable reference
  --> compile_fail/e0507_join_in_drop.rs:27:13
   |
27 |             worker.thread.join().unwrap();
   |             ^^^^^^^^^^^^^ ------ `worker.thread` moved due to this method call
   |             |
   |             move occurs because `worker.thread` has type `JoinHandle<()>`, which does not implement the `Copy` trait
   |
note: `std::thread::JoinHandle::<T>::join` takes ownership of the receiver `self`, which moves `worker.thread`
  --> /rustc/0ef1bc386059596dada8fb65b50425aab2ea637a/library/std/src/thread/join_handle.rs:149:16
```

- **읽는 법**: "가변 참조 뒤에 있는 `worker.thread`를 옮길 수 없다"는 뜻입니다. `note`가 이유를 알려 줍니다. `join`은 `self`를 값으로 받아 소유권을 가져갑니다. `/rustc/...` 경로는 표준 라이브러리 소스의 위치로, 내 코드의 위치가 아닙니다.
- **막은 버그**: 빌린 값에서 필드를 꺼내 갈 수 있다면, 빌려준 쪽(`ThreadPool`)에는 이미 옮겨 간 `JoinHandle`이 그대로 남아 있는 것처럼 보입니다. 그 뒤 필드가 버려질 때 같은 핸들을 두 번 정리하게 됩니다.
- **해결**: `for worker in self.workers.drain(..)`로 벡터를 비우며 `Worker`를 값으로 꺼냅니다. 또는 `thread` 필드를 `Option<thread::JoinHandle<()>>`로 바꾸고 `if let Some(thread) = worker.thread.take() { thread.join().unwrap(); }`로 꺼냅니다(2.9절).

### 4.5 `while let`으로 작업을 받음 — 컴파일되지만 하나씩만 실행

```rust
while let Ok(job) = receiver.lock().unwrap().recv() {
    job();
}
```

에러도 경고도 없고 응답도 정상이라 알아채기 어렵습니다. `job()`을 실행하는 내내 잠금이 유지되어 작업이 하나씩만 실행됩니다. `match receiver.lock().unwrap().recv() { .. }`도 같습니다. [`04_lock_scope.rs`](examples/04_lock_scope.rs)에서 동시 실행 수가 1로 떨어지는 것을 확인하고, `let message = receiver.lock().unwrap().recv();`로 받아 두는 모양으로 고칩니다(2.8절).

### 4.6 송신자를 버리지 않고 `join`함 — 서버가 끝나지 않음

`Drop`에서 `drop(self.sender.take());`를 빠뜨리거나 `join` 반복문 **뒤에** 두면, 작업자는 다음 작업을 영원히 기다리고 `drop`은 작업자를 영원히 기다립니다. 컴파일 에러도 패닉도 없이, `작업자 0의 종료를 기다립니다`가 나온 뒤 어떤 작업자도 `채널이 닫혀 종료합니다`를 출력하지 않고 서버가 멈춰 있습니다. 송신자를 먼저 버려 채널을 닫습니다(2.9절). ch15에서 `for msg in rx`가 끝나지 않던 것과 같은 원인입니다.

### 4.7 HTML 파일을 상대 경로로 읽음 — 실행 위치에 따라 패닉

The Book처럼 `fs::read_to_string(filename)`으로 읽는 서버를 프로젝트 디렉터리가 **아닌** 곳에서 실행하고 `/`를 요청하면 이렇게 됩니다(이 환경에서 그렇게 바꾼 사본으로 확인했습니다).

```console
$ curl -sS http://127.0.0.1:7878/
curl: (52) Empty reply from server
```

```text
http://127.0.0.1:7878 에서 기다립니다 (연결 1개를 처리하면 종료)
연결 1개를 받았으므로 서버를 종료합니다.
작업자 0의 종료를 기다립니다
작업자 0: 작업을 받아 실행합니다
작업자 1: 채널이 닫혀 종료합니다
작업자 2: 채널이 닫혀 종료합니다
작업자 3: 채널이 닫혀 종료합니다

thread '<unnamed>' (6612242) panicked at src/main.rs:84:49:
called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (6612239) panicked at src/lib.rs:129:34:
called `Result::unwrap()` on an `Err` value: Any { .. }
```

- **읽는 법**: 패닉이 두 번 났습니다. 첫 번째는 작업자 스레드(`'<unnamed>'`, 이름 없는 스레드)에서 `hello.html`을 찾지 못한 것(`NotFound`)입니다. 응답을 쓰기 전에 패닉했으므로 `curl`은 빈 응답을 받았습니다. 두 번째는 `main` 스레드가 `Drop`에서 그 작업자를 `join`했다가 `Err`(패닉한 스레드)를 `unwrap`한 것입니다(2.9절의 "남은 약점"). 서버는 종료 코드 101로 끝났습니다. 괄호 안의 숫자는 스레드 ID로 실행마다 다릅니다.
- **원인**: 상대 경로 `"hello.html"`은 현재 디렉터리를 기준으로 찾습니다(2.11절).
- **해결**: 완성판처럼 `Path::new(env!("CARGO_MANIFEST_DIR")).join(filename)`으로 경로를 만들거나, 프로젝트 디렉터리에서 실행합니다.

### 4.8 포트가 이미 쓰이는 중 — `Address already in use`

이전에 띄운 서버가 아직 끝나지 않았는데 서버를 또 띄우면 7878 포트를 차지할 수 없습니다. 완성판은 이렇게 알려 주고 종료 코드 1로 끝납니다(괄호 안의 에러 번호는 macOS에서 확인한 값으로, 운영체제마다 다를 수 있습니다).

```console
$ cargo run -q
127.0.0.1:7878에서 기다릴 수 없습니다: Address already in use (os error 48)
```

The Book처럼 `bind(..).unwrap()`으로 쓰면 같은 내용으로 패닉합니다. 앞의 서버를 끝내거나(`Ctrl+C`), 다른 포트를 쓰세요. 어떤 프로그램이 포트를 쓰고 있는지는 macOS에서 `lsof -i :7878`로 볼 수 있습니다.

## 5. 연습 문제

The Book은 이 프로젝트를 더 발전시킬 과제로 문서 보강, 테스트 추가, `unwrap`을 견고한 에러 처리로 바꾸기, 웹 요청이 아닌 일에 스레드 풀 쓰기, crates.io의 스레드 풀 크레이트와 비교하기를 제안합니다. 아래 연습은 이를 구체화한 것입니다. `projects/hello-server`를 저장소 밖으로 복사해서 풀어 보세요.

### 연습 1. `unwrap`을 걷어 내 패닉에 강한 서버 만들기

다음 두 가지를 고치세요.

1. `handle_connection`이 `std::io::Result<()>`를 돌려주게 하고, 파일 읽기와 쓰기의 `unwrap`을 `?`로 바꿉니다. `main`의 클로저에서 `Err`를 받으면 `eprintln!`으로 에러를 출력합니다. 4.7절처럼 파일을 찾지 못해도 작업자 스레드가 패닉하지 않아야 합니다.
2. 작업 자체가 패닉해도 `ThreadPool`의 `drop`이 패닉하지 않게 합니다. `join()`이 `Err`를 돌려주면 `작업자 N은 패닉으로 끝났습니다`처럼 출력만 하고 넘어갑니다. 이를 확인하는 테스트도 추가하세요. 예를 들어 작업자 2개짜리 풀에 `pool.execute(|| panic!("일부러"))`와 평범한 작업 4개를 맡긴 뒤 풀을 버리고, 평범한 작업 4개가 모두 끝났는지 확인합니다.

<details>
<summary>힌트</summary>

1번: `fn handle_connection(mut stream: TcpStream) -> io::Result<()>`로 바꾸고, `let contents = fs::read_to_string(page_path(filename))?;`, `stream.write_all(response.as_bytes())?;`, 마지막에 `Ok(())`를 씁니다. 요청 줄을 읽는 `let...else`의 `return;`은 `return Ok(());`가 됩니다. `main`에서는 `pool.execute(|| { if let Err(err) = handle_connection(stream) { eprintln!("요청 처리 실패: {err}"); } });`처럼 씁니다(ch9).

2번: `Drop`의 반복문 안을 `if worker.thread.join().is_err() { println!("작업자 {}은 패닉으로 끝났습니다", worker.id); }`로 바꿉니다. 이 환경에서 확인한 결과, 작업이 패닉하면 그 작업자 스레드만 끝나고 남은 작업자는 계속 작업을 받아 처리했습니다. 작업자가 `job()`을 부르기 **전에** 잠금을 풀었기 때문에 `Mutex`가 포이즈닝되지 않은 덕분입니다(2.8절). 작업자 반복문을 `while let`으로 바꿔 같은 실험을 해 보니, 잠금을 쥔 채 패닉한 탓에 `Mutex`가 포이즈닝되어 다른 작업자도 `lock().unwrap()`에서 `PoisonError`로 패닉했고, 평범한 작업은 하나도 끝나지 않았습니다. 잠금을 짧게 쥐는 것은 성능뿐 아니라 견고함에도 중요합니다. 테스트의 패닉 메시지가 출력되는 것은 정상입니다. 테스트에서 끝난 작업 수는 2.9절의 `drop_waits_for_all_queued_jobs`처럼 `Arc<Mutex<Vec<_>>>`나 `Arc<Mutex<usize>>`로 셉니다.

</details>

### 연습 2. 웹 요청이 아닌 일에 스레드 풀 쓰기

`hello_server::ThreadPool`을 써서 1부터 200000까지의 소수 개수를 세는 프로그램을 `src/bin/primes.rs`에 작성하세요. 범위를 50000개씩 네 조각으로 나눠 작업 4개로 맡기고, 각 작업은 자기 조각의 소수 개수를 채널로 보냅니다. `main`은 받은 값을 더해 `소수 개수: 17984`를 출력해야 합니다. `cargo run --bin primes`로 실행합니다.

<details>
<summary>힌트</summary>

`src/bin/` 아래의 파일은 각각 별도의 바이너리 크레이트가 되고, `use hello_server::ThreadPool;`로 라이브러리를 쓸 수 있습니다(ch7). 소수 판정은 `fn is_prime(n: u64) -> bool`로 2부터 `d * d <= n`인 동안 나누어떨어지는지 확인하면 됩니다. `let (tx, rx) = mpsc::channel();`을 만들고, `for chunk in 0..4u64` 안에서 `let tx = tx.clone();`한 뒤 `pool.execute(move || { let start = chunk * 50_000 + 1; let count = (start..start + 50_000).filter(|&n| is_prime(n)).count(); tx.send(count).unwrap(); });`처럼 맡깁니다. 반복문이 끝나면 `drop(tx);`로 원래 송신자를 버리고 `let total: usize = rx.iter().sum();`으로 모읍니다(ch15). `drop(tx)`를 빠뜨리면 무슨 일이 생기는지도 생각해 보세요. 작업자의 로그(`작업자 N: 작업을 받아 실행합니다`)도 함께 출력되는데, 라이브러리가 출력하는 것이므로 정상입니다.

</details>

### 연습 3. `thread::Builder`로 작업자에 이름 붙이기

`Worker::new`에서 `thread::spawn` 대신 `thread::Builder::new().name(format!("작업자-{id}")).spawn(..)`을 쓰세요. 그리고 `handle_connection`의 `응답 완료:` 로그 앞에 `thread::current().name()`으로 얻은 스레드 이름을 붙여, 어느 작업자가 응답했는지 보이게 합니다. 4.7절처럼 작업이 패닉하면 패닉 메시지의 `thread '<unnamed>'`가 어떻게 바뀌는지도 확인하세요. 여유가 있으면 `Builder::spawn`이 돌려주는 `io::Result`의 `Err`를 `ThreadPool::build`까지 전달해 보세요.

<details>
<summary>힌트</summary>

`Builder::spawn`은 `io::Result<JoinHandle<T>>`를 돌려주므로, 우선은 `.unwrap()`이나 `.expect("스레드를 만들 수 없습니다")`를 붙이면 `thread::spawn`과 똑같이 동작합니다(`thread::spawn`이 안에서 하는 일이 바로 이것입니다). `thread::current().name()`은 `Option<&str>`를 돌려주므로 `unwrap_or("이름 없음")`으로 꺼냅니다. 에러를 전달하려면 `Worker::new`가 `io::Result<Worker>`를 돌려주게 하고, `PoolCreationError`에 `Spawn(std::io::Error)` 배리언트를 더한 뒤 `build`에서 `Worker::new(..).map_err(PoolCreationError::Spawn)?`처럼 바꿉니다. 이때 `PoolCreationError`의 `#[derive(PartialEq)]`는 `io::Error`가 `PartialEq`를 구현하지 않아 지워야 하고, 테스트의 `assert_eq!`도 `assert!(matches!(..))`나 `match`로 바꿔야 합니다.

</details>

### 연습 4. crates.io의 스레드 풀과 비교하기

[crates.io](https://crates.io/)에서 `thread pool`로 검색해 스레드 풀 크레이트를 하나 고르고, 그 크레이트의 문서를 읽은 뒤 `projects/hello-server`의 `ThreadPool`을 그 크레이트로 바꿔 보세요. 다음을 비교해 정리합니다.

- 풀을 만들고 작업을 맡기는 API의 모양(`new`/`execute`에 해당하는 것)
- 작업이 패닉하면 어떻게 되는가
- 풀이 버려질 때 남은 작업을 기다리는가, 기다리게 하려면 무엇을 불러야 하는가
- 크기 0을 어떻게 다루는가

<details>
<summary>힌트</summary>

의존성은 `cargo add 크레이트이름`으로 추가합니다(ch2에서 `rand`를 추가한 방법과 같습니다. 의존성 관리는 [Part 3 ch2](../../part3-cargo/ch2-dependencies/README.md)에서 자세히 다룹니다). 크레이트 문서는 crates.io 페이지의 Documentation 링크나 `cargo doc --open`으로 볼 수 있습니다. 문서에서 `Drop`, `join`, `panic`이라는 단어를 찾아 보면 비교할 내용이 빨리 보입니다. 이 프로젝트의 테스트(`drop_waits_for_all_queued_jobs` 등)를 그 크레이트로 옮겨 실행해 보는 것도 좋은 방법입니다.

</details>

## Part 0를 마치며

여기까지 왔다면 Rust의 기초 전체를 한 바퀴 돈 것입니다. 이 마지막 프로젝트에 소유권(연결의 수명), 트레이트와 제네릭(`execute`의 바운드), 클로저(작업), 스마트 포인터와 `Drop`(우아한 종료), 스레드와 동기화(작업자와 채널)가 모두 들어 있었습니다. 다음 단계는 [로드맵](../../docs/roadmap.md)의 Part 1 이후입니다. **Part 1 표준 라이브러리**에서는 Part 0에서 맛만 본 타입들을 넓고 깊게 다루는데, 이 챕터와 직접 이어지는 것은 스레드와 동기화 도구 전반(`ch11-threads-sync`), TCP·UDP 소켓(`ch12-net`), 사용자 정의 에러 타입(`ch8-errors-panic`)입니다. **Part 3 Cargo**에서는 이 챕터에서 쓴 `CARGO_MANIFEST_DIR`, 의존성 추가, 워크스페이스를 제대로 배우고, **Part 7 명령줄 애플리케이션**에서는 `clap`, `anyhow` 같은 크레이트로 실전 도구를 만들며 `Ctrl+C` 처리(`ch6-signals-config-exit-codes`)까지 다룹니다. Part 7과 Part 8은 서로 독립적이므로 관심 있는 쪽을 먼저 해도 됩니다. 언어 규칙을 명세 수준에서 정확히 알고 싶어지면 Part 9 레퍼런스로, `unsafe`의 세계가 궁금해지면 Part 10 Rustonomicon으로 넘어갑니다.

## 6. 참고 자료

- The Book (기준 문서)
  - [21. Final Project: Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html)
  - [21.1 Building a Single-Threaded Web Server](https://doc.rust-lang.org/book/ch21-01-single-threaded.html): TCP와 HTTP, 요청 읽기, 응답 쓰기, 200/404
  - [21.2 From Single-Threaded to Multithreaded Server](https://doc.rust-lang.org/book/ch21-02-multithreaded.html): `/sleep`, 스레드 풀 설계, 컴파일러가 이끄는 개발, `Worker`와 채널, `while let`의 잠금 문제
  - [21.3 Graceful Shutdown and Cleanup](https://doc.rust-lang.org/book/ch21-03-graceful-shutdown-and-cleanup.html): `Drop`, `Option::take`, `drain`, 개선 과제 목록
- Rust By Example에는 이 챕터에 딱 맞는 절이 없습니다. 스레드와 채널 복습은 [ch15](../ch15-concurrency/README.md)의 참고 자료에 모아 두었습니다.
- Rustlings에도 이 챕터에 대응하는 연습은 없습니다. 가장 가까운 것은 ch15에서 안내한 [`20_threads`](https://github.com/rust-lang/rustlings/tree/main/exercises/20_threads)입니다.
- 표준 라이브러리 문서
  - [`std::net::TcpListener`](https://doc.rust-lang.org/std/net/struct.TcpListener.html): `bind`(포트 0의 의미), `local_addr`, `incoming`이 `None`을 돌려주지 않는다는 설명
  - [`std::net::TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html): 값이 drop되면 연결이 닫힘, 읽기·쓰기, 타임아웃
  - [`std::io::BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html): `lines`가 줄 끝의 `\n`과 `\r\n`을 떼어 낸다는 설명
  - [`std::thread::Builder`](https://doc.rust-lang.org/std/thread/struct.Builder.html): 스레드 이름 붙이기, 생성 실패를 `io::Result`로 받기(연습 3)
  - [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.drain)의 `drain`
- 에러 설명: `rustc --explain E0507`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
