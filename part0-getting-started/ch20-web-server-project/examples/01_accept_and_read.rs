// TCP 연결을 받아 HTTP 요청을 줄 단위로 읽습니다. (The Book 21.1, Listing 21-1·21-2 단계)
//
// 실행: cargo run -p p0-ch20-web-server-project --example 01_accept_and_read
//
// 브라우저 대신 예제 안의 클라이언트 스레드가 127.0.0.1의 서버에 접속해 요청 두 개를 보냅니다.
// 서버는 연결 두 개를 처리하면 끝나므로 예제는 스스로 종료합니다.
// 포트 번호는 운영체제가 고르므로 실행할 때마다 달라집니다.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    // 포트 0으로 바인드하면 운영체제가 비어 있는 포트를 하나 골라 줍니다.
    // 7878처럼 번호를 정해 두면 그 포트를 이미 쓰는 프로그램이 있을 때 bind가 실패합니다.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    println!("{addr}에서 연결을 기다립니다"); // 출력: 127.0.0.1:<포트>에서 연결을 기다립니다 (포트는 실행마다 다름)

    // 클라이언트가 보낼 요청을 main에서 미리 만들어 출력합니다.
    // 클라이언트 스레드 안에서 출력하면 서버 쪽 출력과 순서가 섞일 수 있기 때문입니다.
    let requests: Vec<String> = ["/", "/favicon.ico"]
        .iter()
        .map(|path| format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nAccept: */*\r\n\r\n"))
        .collect();
    for request in &requests {
        // {:?}로 출력하면 줄 끝의 \r\n이 눈에 보이게 이스케이프됩니다.
        println!("클라이언트가 보낼 요청: {request:?}");
    }
    // 출력: 클라이언트가 보낼 요청: "GET / HTTP/1.1\r\nHost: 127.0.0.1:<포트>\r\nAccept: */*\r\n\r\n"
    // 출력: 클라이언트가 보낼 요청: "GET /favicon.ico HTTP/1.1\r\nHost: 127.0.0.1:<포트>\r\nAccept: */*\r\n\r\n"

    // 클라이언트 역할을 하는 스레드입니다. 연결마다 요청을 쓰고, 스트림이 drop되면 연결이 닫힙니다.
    let client = thread::spawn(move || {
        for request in requests {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(request.as_bytes()).unwrap();
        }
    });

    // incoming()은 연결 시도를 하나씩 내놓는 반복자이고, 절대 None을 돌려주지 않습니다.
    // 그대로 for 문에 넣으면 영원히 기다리므로 take(2)로 두 번째 연결까지만 받습니다.
    for (number, stream) in listener.incoming().take(2).enumerate() {
        // 연결 시도는 실패할 수도 있으므로 각 항목이 Result입니다.
        let stream = stream.unwrap();
        println!("연결 {}번이 성립했습니다", number + 1);
        handle_connection(stream);
    }
    // 출력: 연결 1번이 성립했습니다
    // 출력: Request: [
    // 출력:     "GET / HTTP/1.1",
    // 출력:     "Host: 127.0.0.1:<포트>",
    // 출력:     "Accept: */*",
    // 출력: ]
    // 출력: 연결 2번이 성립했습니다
    // 출력: Request: [
    // 출력:     "GET /favicon.ico HTTP/1.1",
    // 출력:     "Host: 127.0.0.1:<포트>",
    // 출력:     "Accept: */*",
    // 출력: ]

    client.join().unwrap();
    println!("연결 두 개를 처리했으므로 서버를 끝냅니다"); // 출력: 연결 두 개를 처리했으므로 서버를 끝냅니다
}

fn handle_connection(stream: TcpStream) {
    // BufReader는 스트림에서 한 번에 넉넉히 읽어 두고 그 안에서 줄을 잘라 줍니다.
    // &stream을 넘기는 까닭: &TcpStream도 Read를 구현하므로 스트림을 빌려서 읽을 수 있고,
    // 다음 단계(02)에서 같은 스트림에 응답을 써야 하기 때문입니다.
    let buf_reader = BufReader::new(&stream);

    // lines()는 줄마다 Result<String>을 내놓고, 줄 끝의 \r\n(또는 \n)은 떼어 냅니다.
    // HTTP 요청의 헤더는 빈 줄로 끝납니다. 클라이언트는 그 뒤로 응답을 기다리며
    // 연결을 열어 두므로, 끝까지 읽으려 하면(read_to_string 등) 영원히 기다리게 됩니다.
    // 그래서 빈 줄이 나올 때까지만 읽습니다.
    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // {:#?}는 디버그 형식을 여러 줄로 보기 좋게 출력합니다.
    println!("Request: {http_request:#?}");
}
