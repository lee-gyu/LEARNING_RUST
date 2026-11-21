// 제한 시간과 에러 종류: 네트워크 에러는 io::Error의 ErrorKind로 구분합니다.
//
//   1) 아무도 기다리지 않는 포트에 connect → ConnectionRefused. connect_timeout도 곧바로 같은 에러입니다.
//   2) 이미 쓰이는 주소에 bind → AddrInUse. 같은 포트 번호라도 TCP와 UDP는 따로 셉니다.
//   3) 제한 시간에 0을 주면 InvalidInput입니다. "제한 없음"은 None으로 나타냅니다.
//   4) 읽기 제한 시간: shutdown(Write)를 잊은 클라이언트가 응답의 끝(EOF)을 기다리다 시간 초과합니다.
//   5) 논블로킹 모드: 기다리지 않고 곧바로 WouldBlock을 돌려줍니다.
//
// 에러 메시지 문구와 os error 번호는 운영체제마다 달라서 ErrorKind만 출력합니다.
// 걸린 시간도 실행마다 조금씩 달라서 "기준보다 길었는가/짧았는가"만 출력합니다.
//
// 실행: cargo run -p p1-ch12-net --example 04_timeouts_errors

use std::io::{self, ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::thread;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_millis(200);

fn main() -> io::Result<()> {
    // ── 1. 연결 거부 ───────────────────────────────────────────────
    println!("1. 아무도 기다리지 않는 포트에 연결");
    let closed = closed_port()?;
    println!(
        "    connect         → {}",
        show(&TcpStream::connect(closed))
    );
    let start = Instant::now();
    let result = TcpStream::connect_timeout(&closed, TIMEOUT);
    let fast = start.elapsed() < TIMEOUT;
    println!(
        "    connect_timeout → {}, 제한 시간보다 빨리 돌아왔는가: {fast}",
        show(&result)
    );
    // 출력: connect         → Err(ConnectionRefused)
    // 출력: connect_timeout → Err(ConnectionRefused), 제한 시간보다 빨리 돌아왔는가: true

    // ── 2. 이미 쓰이는 주소 ────────────────────────────────────────
    println!("2. 이미 쓰이는 주소에 bind");
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let busy = listener.local_addr()?;
    println!(
        "    같은 주소에 TcpListener::bind → {}",
        show(&TcpListener::bind(busy))
    );
    println!(
        "    같은 주소에 UdpSocket::bind   → {}",
        show(&UdpSocket::bind(busy))
    );
    // 출력: TcpListener::bind → Err(AddrInUse), UdpSocket::bind → Ok

    // ── 3. 제한 시간 0 ─────────────────────────────────────────────
    println!("3. 제한 시간에 0을 주면");
    let zero = Duration::ZERO;
    println!(
        "    connect_timeout(.., 0)       → {}",
        show(&TcpStream::connect_timeout(&busy, zero))
    );
    let stream = TcpStream::connect(busy)?;
    println!(
        "    set_read_timeout(Some(0))    → {}",
        show(&stream.set_read_timeout(Some(zero)))
    );
    println!(
        "    set_read_timeout(None)       → {}",
        show(&stream.set_read_timeout(None))
    );
    // 출력: Err(InvalidInput), Err(InvalidInput), Ok
    drop(stream);
    drop(listener);

    // ── 4. 읽기 제한 시간: shutdown을 잊은 클라이언트 ──────────────
    println!(
        "4. 읽기 제한 시간 {}ms: shutdown(Write)를 잊으면",
        TIMEOUT.as_millis()
    );
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    // 02와 같은 에코 서버입니다. 클라이언트가 쓰기를 닫아야(EOF) 반복을 끝내고 연결을 닫습니다.
    let server = thread::spawn(move || -> io::Result<usize> {
        let (mut stream, _) = listener.accept()?;
        let mut buf = [0u8; 1024];
        let mut total = 0;
        loop {
            let n = stream.read(&mut buf)?;
            if n == 0 {
                return Ok(total);
            }
            stream.write_all(&buf[..n])?;
            total += n;
        }
    });

    let mut client = TcpStream::connect(addr)?;
    client.set_read_timeout(Some(TIMEOUT))?;
    client.write_all("안녕".as_bytes())?;
    // shutdown(Write)를 빠뜨렸습니다. 서버는 다음 바이트를, 우리는 EOF를 기다립니다.
    let mut reply = Vec::new();
    let start = Instant::now();
    let result = client.read_to_end(&mut reply);
    let waited = start.elapsed() >= TIMEOUT;
    println!(
        "    read_to_end → {}, 제한 시간만큼 기다렸는가: {waited}",
        show(&result)
    );
    if let Err(e) = &result {
        println!("    제한 시간 초과로 볼 것인가: {}", is_timeout(e));
    }
    // read_to_end는 에러로 끝나도 그때까지 읽은 바이트를 buf에 남겨 둡니다.
    println!(
        "    그사이 받아 둔 것: {:?}",
        String::from_utf8_lossy(&reply)
    );
    // 출력(macOS): read_to_end → Err(WouldBlock), 제한 시간만큼 기다렸는가: true
    // 출력:        제한 시간 초과로 볼 것인가: true
    // 출력:        그사이 받아 둔 것: "안녕"

    client.shutdown(Shutdown::Write)?; // 이제 서버가 EOF를 받고 연결을 닫습니다.
    let more = client.read_to_end(&mut reply)?;
    println!("    shutdown(Write) 뒤 다시 read_to_end → Ok({more}바이트 더), EOF에 도달"); // 출력: Ok(0바이트 더)
    let echoed = server
        .join()
        .expect("서버 스레드는 패닉하지 않아야 합니다")?;
    println!("    서버가 돌려준 바이트: {echoed}"); // 출력: 6

    // ── 5. 논블로킹 모드 ───────────────────────────────────────────
    println!("5. 논블로킹 모드");
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    // 기다리는 연결이 없으므로, 블로킹 모드라면 여기서 영원히 멈췄을 것입니다.
    println!("    연결이 없을 때 accept → {}", show(&listener.accept()));
    // 연결은 운영체제가 받아 큐에 넣어 두므로, 서버가 accept하지 않아도 connect는 성공합니다.
    let client = TcpStream::connect(listener.local_addr()?)?;
    client.set_nonblocking(true)?;
    let start = Instant::now();
    let mut buf = [0u8; 16];
    let result = (&client).read(&mut buf);
    let instant = start.elapsed() < Duration::from_millis(50);
    println!(
        "    받은 데이터가 없을 때 read → {}, 곧바로 돌아왔는가: {instant}",
        show(&result)
    );
    // 출력: accept → Err(WouldBlock) / read → Err(WouldBlock), 곧바로 돌아왔는가: true
    Ok(())
}

/// 잠깐 bind했다가 닫아서, 지금은 아무도 기다리지 않는 주소를 얻습니다.
/// (그 짧은 사이에 다른 프로그램이 같은 포트를 차지할 가능성은 아주 낮습니다.)
fn closed_port() -> io::Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.local_addr()
} // listener가 여기서 버려져 포트가 닫힙니다.

/// 제한 시간 초과는 플랫폼에 따라 WouldBlock(유닉스 계열) 또는 TimedOut(윈도우)으로 옵니다.
fn is_timeout(e: &io::Error) -> bool {
    matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

/// 결과를 "Ok" 또는 "Err(ErrorKind)"로 짧게 보여 줍니다.
fn show<T>(result: &io::Result<T>) -> String {
    match result {
        Ok(_) => "Ok".to_string(),
        Err(e) => format!("Err({:?})", e.kind()),
    }
}
