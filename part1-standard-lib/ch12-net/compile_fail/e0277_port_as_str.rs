// 기대하는 에러: E0277 — the trait bound `(&str, &str): ToSocketAddrs` is not satisfied
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_port_as_str.rs --out-dir /tmp/rust-p1-ch12
//
// connect의 인자는 A: ToSocketAddrs입니다. 튜플 모양으로 주려면 (호스트, 포트)의 포트가 u16이어야 합니다.
// 설정 파일이나 명령줄에서 읽은 포트는 문자열이기 쉬운데, ToSocketAddrs는 (&str, &str)을 받지 않습니다.
// 포트가 0~65535 범위의 숫자인지 검사하는 일을 connect가 대신해 주지 않는다는 뜻입니다.
// 해결: 포트를 먼저 u16으로 파싱합니다.
//       let port: u16 = port.parse().expect("포트는 0~65535 사이의 숫자여야 합니다");
//       TcpStream::connect(("127.0.0.1", port))
//       또는 "127.0.0.1:8080"처럼 "호스트:포트" 문자열 하나로 넘깁니다.

use std::net::TcpStream;

fn main() {
    let port = "8080"; // 예: 설정 파일에서 읽은 값
    match TcpStream::connect(("127.0.0.1", port)) {
        Ok(_) => println!("연결했습니다"),
        Err(e) => println!("연결하지 못했습니다: {e}"),
    }
}
