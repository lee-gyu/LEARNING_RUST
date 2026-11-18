// 기대하는 에러: E0599 — no method named `read_line` found for struct `TcpStream` in the current scope
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0599_read_line_on_tcpstream.rs --out-dir /tmp/rust-p1-ch12
//
// read_line과 lines는 BufRead 트레이트의 메서드입니다. BufRead는 "안에 버퍼가 있어서 줄 끝을 찾을 수 있는"
// 타입만 구현합니다. TcpStream은 운영체제에 바로 읽기를 부탁하는 타입이라 버퍼가 없으므로 BufRead가 아닙니다.
// 그래서 use std::io::BufRead;를 써 두어도 이 메서드를 찾지 못합니다.
// 해결: BufReader로 감쌉니다. 쓰기도 해야 한다면 &stream을 빌려 감싸서 stream을 계속 쓸 수 있게 합니다.
//       let mut reader = BufReader::new(&stream);
//       reader.read_line(&mut line)?;
//       (&stream).write_all(b"...")?;   // 또는 try_clone한 핸들로 씁니다.

use std::io::BufRead;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    let mut line = String::new();
    stream.read_line(&mut line)?;
    println!("받은 줄: {line:?}");
    Ok(())
}
