// 기대하는 에러: E0596 — cannot borrow `stream` as mutable, as it is not declared as mutable
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0596_stream_not_mut.rs --out-dir /tmp/rust-p1-ch12
//
// Write::write_all은 &mut self를 받습니다. stream.write_all(..)은 TcpStream에 구현된 Write를 쓰므로
// &mut TcpStream이 필요하고, mut 없이 만든 바인딩에서는 가변으로 빌릴 수 없습니다.
// 해결 1: let mut stream = ...;  (컴파일러의 help가 제안하는 방법)
// 해결 2: &TcpStream에도 Write가 구현되어 있으므로 (&stream).write_all(..)로 씁니다.
//         여러 스레드나 여러 함수가 같은 stream에 쓰고 읽어야 할 때 이 방법이 쓸모 있습니다(03 예제).

use std::io::Write;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:7878")?;
    stream.write_all(b"hello\n")?;
    Ok(())
}
