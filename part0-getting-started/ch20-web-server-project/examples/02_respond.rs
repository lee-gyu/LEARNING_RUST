// 요청 줄을 보고 HTTP 응답(상태 줄 + 헤더 + 본문)을 돌려줍니다. (The Book 21.1, Listing 21-3~21-9 단계)
//
// 실행: cargo run -p p0-ch20-web-server-project --example 02_respond
//
// 클라이언트 스레드가 "/"와 "/nothing"을 차례로 요청하고, 서버가 보낸 응답을 끝까지 받아 돌려줍니다.
// 서버는 연결 두 개를 처리하면 끝나므로 예제는 스스로 종료합니다.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

// 완성판(projects/hello-server)은 HTML을 파일에서 읽습니다. 이 예제는 응답 형식에 집중하려고
// HTML을 문자열 상수로 두었습니다.
const HELLO_HTML: &str = "<!DOCTYPE html>
<html lang=\"ko\"><head><meta charset=\"utf-8\"><title>안녕!</title></head>
<body><h1>안녕하세요!</h1></body></html>
";
const NOT_FOUND_HTML: &str = "<!DOCTYPE html>
<html lang=\"ko\"><head><meta charset=\"utf-8\"><title>없음</title></head>
<body><h1>이런!</h1></body></html>
";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    // 클라이언트: 요청을 하나 보내고, 서버가 연결을 닫을 때까지 응답을 모두 읽습니다.
    // 응답은 출력하지 않고 돌려줍니다. main이 서버 출력이 끝난 뒤에 출력하면 순서가 섞이지 않습니다.
    let client = thread::spawn(move || {
        let mut responses = Vec::new();
        for path in ["/", "/nothing"] {
            let mut stream = TcpStream::connect(addr).unwrap();
            let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n\r\n");
            stream.write_all(request.as_bytes()).unwrap();

            // 서버가 응답을 쓰고 스트림을 drop하면 연결이 닫히고, 그때 read_to_string이 끝납니다.
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            responses.push(response);
        }
        responses
    });

    for stream in listener.incoming().take(2) {
        handle_connection(stream.unwrap());
    }
    // 출력: [서버] GET / HTTP/1.1 → HTTP/1.1 200 OK
    // 출력: [서버] GET /nothing HTTP/1.1 → HTTP/1.1 404 NOT FOUND

    let responses = client.join().unwrap();
    for response in responses {
        // 헤더와 본문은 빈 줄(\r\n\r\n)로 나뉩니다.
        let (head, body) = response.split_once("\r\n\r\n").unwrap();
        println!("--- 클라이언트가 받은 응답 ---");
        println!("헤더 부분: {head:?}");
        print!("본문:\n{body}");
        // Content-Length는 글자 수가 아니라 바이트 수입니다. 한글은 UTF-8에서 한 글자가 3바이트입니다(ch8).
        println!(
            "본문의 바이트 수: {}, 글자 수: {}",
            body.len(),
            body.chars().count()
        );
    }
    // 출력: --- 클라이언트가 받은 응답 ---
    // 출력: 헤더 부분: "HTTP/1.1 200 OK\r\nContent-Length: 136"
    // 출력: 본문:
    // 출력: <!DOCTYPE html>
    // 출력: <html lang="ko"><head><meta charset="utf-8"><title>안녕!</title></head>
    // 출력: <body><h1>안녕하세요!</h1></body></html>
    // 출력: 본문의 바이트 수: 136, 글자 수: 122
    // 출력: --- 클라이언트가 받은 응답 ---
    // 출력: 헤더 부분: "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 126"
    // 출력: 본문:
    // 출력: <!DOCTYPE html>
    // 출력: <html lang="ko"><head><meta charset="utf-8"><title>없음</title></head>
    // 출력: <body><h1>이런!</h1></body></html>
    // 출력: 본문의 바이트 수: 126, 글자 수: 118
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    // 첫 줄(요청 줄)만 있으면 무엇을 돌려줄지 정할 수 있습니다.
    // next()는 Option(줄이 없을 수 있음), 그 안은 Result(읽다가 실패할 수 있음)라서 unwrap이 두 번입니다.
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // if/else(또는 match)는 "무엇이 다른가"만 고르고, 응답을 만드는 코드는 한 번만 씁니다.
    // &request_line[..]은 String 전체를 &str로 빌린 슬라이스입니다(ch4). 문자열 리터럴 패턴과 비교하려면 &str이어야 합니다.
    let (status_line, contents) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", HELLO_HTML),
        _ => ("HTTP/1.1 404 NOT FOUND", NOT_FOUND_HTML),
    };
    println!("[서버] {request_line} → {status_line}");

    // 응답 형식: 상태 줄 CRLF, 헤더 CRLF, 빈 줄(CRLF), 본문
    // Content-Length가 있어야 클라이언트가 본문이 어디서 끝나는지 압니다.
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // write_all은 바이트 슬라이스(&[u8])를 받으므로 as_bytes로 바꿉니다.
    // 쓰기는 스트림의 상태를 바꾸므로 매개변수가 mut stream이어야 합니다.
    stream.write_all(response.as_bytes()).unwrap();
} // 여기서 stream이 drop되며 연결이 닫힙니다.
