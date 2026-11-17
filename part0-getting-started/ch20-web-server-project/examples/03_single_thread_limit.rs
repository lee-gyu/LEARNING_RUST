// 단일 스레드 서버에서는 느린 요청 하나가 뒤의 요청을 모두 막습니다. (The Book 21.2, Listing 21-10·21-11 단계)
//
// 실행: cargo run -p p0-ch20-web-server-project --example 03_single_thread_limit
//
// 클라이언트 두 개가 거의 동시에 접속합니다. A는 1초가 걸리는 "/sleep"을, B는 0.1초 뒤에 바로
// 끝나는 "/"를 요청합니다. 같은 실험을 두 가지 서버로 해서 B가 응답을 받는 시각을 비교합니다.
//   1) 연결을 하나씩 차례로 처리하는 서버
//   2) 연결마다 스레드를 새로 만드는 서버
// 시각은 0.1초 단위로 반올림해 출력합니다. 컴퓨터가 바쁘면 0.1초 정도 다르게 나올 수 있습니다.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

// The Book은 5초를 쓰지만, 예제가 빨리 끝나도록 1초로 줄였습니다.
const SLOW: Duration = Duration::from_secs(1);

fn main() {
    println!("1) 연결을 하나씩 처리하는 서버");
    experiment(serve_one_by_one);
    // 출력: 1) 연결을 하나씩 처리하는 서버
    // 출력:     0.0초  A가 /sleep 요청
    // 출력:     0.1초  B가 / 요청
    // 출력:     1.0초  A가 응답 받음 (HTTP/1.1 200 OK)
    // 출력:     1.0초  B가 응답 받음 (HTTP/1.1 200 OK)

    println!("2) 연결마다 스레드를 만드는 서버");
    experiment(serve_thread_per_connection);
    // 출력: 2) 연결마다 스레드를 만드는 서버
    // 출력:     0.0초  A가 /sleep 요청
    // 출력:     0.1초  B가 / 요청
    // 출력:     0.1초  B가 응답 받음 (HTTP/1.1 200 OK)
    // 출력:     1.0초  A가 응답 받음 (HTTP/1.1 200 OK)
}

/// 서버 하나를 띄우고, 클라이언트 A와 B를 접속시킨 뒤 일어난 일을 시각순으로 출력합니다.
/// serve는 함수 포인터(ch19)입니다. 두 가지 서버를 같은 실험에 넣어 보려고 매개변수로 받습니다.
fn experiment(serve: fn(TcpListener)) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = thread::spawn(move || serve(listener));

    let start = Instant::now();
    let client_a = thread::spawn(move || request(addr, "A", "/sleep", start));
    thread::sleep(Duration::from_millis(100));
    let client_b = thread::spawn(move || request(addr, "B", "/", start));

    // 클라이언트가 기록한 사건을 모아 시각순으로 정렬한 뒤 main에서 출력합니다.
    // 스레드 안에서 바로 출력하면 줄 순서가 실행마다 섞일 수 있기 때문입니다.
    let mut events = client_a.join().unwrap();
    events.extend(client_b.join().unwrap());
    events.sort_by_key(|(at, _)| *at);
    for (at, what) in events {
        println!("    {:.1}초  {what}", round_to_tenth(at));
    }

    server.join().unwrap();
}

/// 클라이언트 하나: 요청을 보내고 응답을 받기까지의 사건을 (시각, 설명)으로 기록해 돌려줍니다.
fn request(addr: SocketAddr, name: &str, path: &str, start: Instant) -> Vec<(Duration, String)> {
    let mut stream = TcpStream::connect(addr).unwrap();
    let sent_at = start.elapsed();
    stream
        .write_all(format!("GET {path} HTTP/1.1\r\n\r\n").as_bytes())
        .unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let status_line = response.lines().next().unwrap_or("");

    vec![
        (sent_at, format!("{name}가 {path} 요청")),
        (
            start.elapsed(),
            format!("{name}가 응답 받음 ({status_line})"),
        ),
    ]
}

/// [1] Listing 21-10의 서버: main 스레드 하나가 연결을 차례로 처리합니다.
/// A를 처리하는 1초 동안은 B의 연결을 받지도 못합니다.
fn serve_one_by_one(listener: TcpListener) {
    for stream in listener.incoming().take(2) {
        handle_connection(stream.unwrap());
    }
}

/// [2] Listing 21-11의 서버: 연결마다 스레드를 새로 만들어 맡깁니다.
/// B가 A를 기다리지 않습니다. 하지만 연결 수만큼 스레드가 생기고, 그 수에 상한이 없습니다.
fn serve_thread_per_connection(listener: TcpListener) {
    let mut handles = Vec::new();
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        // handle_connection이 stream의 소유권을 가져가므로, 클로저는 move 없이도 stream을
        // 값으로 캡처합니다(ch13). 그래서 thread::spawn이 요구하는 'static을 만족합니다.
        handles.push(thread::spawn(|| handle_connection(stream)));
    }
    // 예제를 깔끔하게 끝내려고 만든 스레드를 모두 기다립니다. The Book의 Listing 21-11은 기다리지 않습니다.
    for handle in handles {
        handle.join().unwrap();
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let status_line = match &request_line[..] {
        "GET / HTTP/1.1" => "HTTP/1.1 200 OK",
        "GET /sleep HTTP/1.1" => {
            // 오래 걸리는 처리(큰 파일 읽기, 느린 데이터베이스 등)를 흉내 냅니다.
            thread::sleep(SLOW);
            "HTTP/1.1 200 OK"
        }
        _ => "HTTP/1.1 404 NOT FOUND",
    };

    let response = format!("{status_line}\r\nContent-Length: 0\r\n\r\n");
    stream.write_all(response.as_bytes()).unwrap();
}

/// 시각을 0.1초 단위로 반올림합니다. 밀리초 단위의 작은 차이를 감춰 출력을 읽기 쉽게 합니다.
fn round_to_tenth(duration: Duration) -> f64 {
    (duration.as_secs_f64() * 10.0).round() / 10.0
}
