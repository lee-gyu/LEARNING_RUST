// 기대하는 에러: E0382 — borrow of moved value: `stream`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0382_stream_moved_into_thread.rs --out-dir /tmp/rust-p1-ch12
//
// 쓰기는 다른 스레드에 맡기고 main에서는 응답을 읽으려 했습니다. 그런데 move 클로저가 stream의 소유권을
// 가져갔으므로 main에는 읽을 stream이 남아 있지 않습니다. TcpStream은 Clone을 구현하지 않습니다.
// 해결 1: try_clone으로 같은 소켓의 핸들을 하나 더 만들어 그것을 스레드로 옮깁니다.
//         let mut writer = stream.try_clone()?;
//         let handle = thread::spawn(move || writer.write_all(b"hello\n"));
// 해결 2: thread::scope 안에서 스레드가 &stream을 빌려 (&stream).write_all(..)로 씁니다(03 예제의 4번 구역).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    let writer = thread::spawn(move || stream.write_all(b"hello\n"));
    let mut reply = String::new();
    stream.read_to_string(&mut reply)?;
    writer.join().expect("쓰기 스레드는 패닉하지 않아야 합니다")?;
    println!("받은 응답: {reply:?}");
    Ok(())
}
