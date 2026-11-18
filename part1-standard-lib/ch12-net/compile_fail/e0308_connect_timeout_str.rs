// 기대하는 에러: E0308 — mismatched types: expected `&SocketAddr`, found `&str`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0308_connect_timeout_str.rs --out-dir /tmp/rust-p1-ch12
//
// connect는 A: ToSocketAddrs를 받아 문자열도 받아 주지만, connect_timeout은 &SocketAddr 하나만 받습니다.
// 제한 시간은 주소 하나에 대해 재는 것이라서, 이름 해석으로 주소가 여러 개 나올 수 있는 인자는 받지 않습니다.
// 해결 1: 문자열을 먼저 SocketAddr로 파싱합니다(이름은 해석하지 않음).
//         let addr: SocketAddr = "127.0.0.1:7878".parse().expect("올바른 소켓 주소입니다");
//         TcpStream::connect_timeout(&addr, Duration::from_millis(500))
// 해결 2: 이름을 써야 한다면 to_socket_addrs()로 주소 목록을 얻어 하나씩 connect_timeout을 시도합니다.

use std::net::TcpStream;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let _stream = TcpStream::connect_timeout("127.0.0.1:7878", Duration::from_millis(500))?;
    Ok(())
}
