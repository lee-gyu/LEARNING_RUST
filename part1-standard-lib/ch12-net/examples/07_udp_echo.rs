// UDP 에코 서버와 클라이언트: 전달이 보장되지 않으므로 클라이언트가 제한 시간과 재전송을 맡습니다.
//
//   1) 서버는 소켓 하나로 모든 클라이언트를 상대합니다. recv_from이 알려 준 주소로 send_to해서 돌려줍니다.
//      TCP 서버가 연결마다 TcpStream을 따로 받던 것(02 예제)과 비교해 보세요.
//   2) 클라이언트는 connect로 서버를 고정하고, 읽기 제한 시간 안에 답이 없으면 같은 메시지를 다시 보냅니다.
//   3) 손실을 흉내 내려고 서버는 두 번째로 받은 데이터그램을 일부러 버립니다.
//
// 서버는 데이터그램 4개(버린 것 포함)를 처리하면 끝납니다. 출력은 모두 main 스레드가 합니다.
//
// 실행: cargo run -p p1-ch12-net --example 07_udp_echo

use std::io::{self, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

/// 클라이언트가 답을 기다리는 시간입니다.
const REPLY_TIMEOUT: Duration = Duration::from_millis(200);
/// 클라이언트가 같은 메시지를 보내 보는 최대 횟수입니다.
const MAX_TRIES: u32 = 3;
/// 서버가 처리하고 끝낼 데이터그램 수입니다.
const DATAGRAMS: usize = 4;
/// 서버가 일부러 버릴 데이터그램의 순번(0부터 셈)입니다.
const DROP_INDEX: usize = 1;

struct ServerLog {
    from: SocketAddr,
    text: String,
    dropped: bool,
}

fn main() -> io::Result<()> {
    let server_socket = UdpSocket::bind("127.0.0.1:0")?;
    let server_addr = server_socket.local_addr()?;
    // 서버도 영원히 기다리지 않도록 제한 시간을 둡니다. 2초 동안 아무것도 오지 않으면 에러로 끝납니다.
    server_socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    let server = thread::spawn(move || -> io::Result<Vec<ServerLog>> {
        let mut log = Vec::new();
        let mut buf = [0u8; 2048];
        for i in 0..DATAGRAMS {
            let (n, from) = server_socket.recv_from(&mut buf)?;
            let dropped = i == DROP_INDEX;
            if !dropped {
                server_socket.send_to(&buf[..n], from)?; // 보낸 사람에게 그대로 돌려줍니다.
            }
            log.push(ServerLog {
                from,
                text: text(&buf[..n]),
                dropped,
            });
        }
        Ok(log)
    });

    let client_a = client(server_addr)?;
    let client_b = client(server_addr)?;

    println!("1. 클라이언트 A");
    for message in ["안녕", "두 번째 메시지"] {
        let (reply, tries) = request(&client_a, message)?;
        println!("    {message:?} → 답 {reply:?} (보낸 횟수: {tries})");
    }
    // 출력: "안녕" → 답 "안녕" (보낸 횟수: 1)
    // 출력: "두 번째 메시지" → 답 "두 번째 메시지" (보낸 횟수: 2)  ← 첫 번째는 서버가 버렸습니다.

    println!("2. 클라이언트 B");
    let (reply, tries) = request(&client_b, "나는 B")?;
    println!("    \"나는 B\" → 답 {reply:?} (보낸 횟수: {tries})"); // 출력: (보낸 횟수: 1)

    let log = server
        .join()
        .expect("서버 스레드는 패닉하지 않아야 합니다")?;
    let a_addr = client_a.local_addr()?;
    let b_addr = client_b.local_addr()?;
    println!("3. 서버 쪽 기록 (소켓 하나로 두 클라이언트를 상대함)");
    for entry in &log {
        let who = if entry.from == a_addr {
            "A"
        } else if entry.from == b_addr {
            "B"
        } else {
            "알 수 없는 주소"
        };
        let action = if entry.dropped {
            "일부러 버림"
        } else {
            "돌려줌"
        };
        println!("    {who}에게서 {:?} → {action}", entry.text);
    }
    // 출력: A에게서 "안녕" → 돌려줌 / A에게서 "두 번째 메시지" → 일부러 버림
    //       A에게서 "두 번째 메시지" → 돌려줌 / B에게서 "나는 B" → 돌려줌
    Ok(())
}

/// 서버를 상대로 고정하고 답을 기다릴 제한 시간을 건 클라이언트 소켓을 만듭니다.
fn client(server: SocketAddr) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect(server)?;
    socket.set_read_timeout(Some(REPLY_TIMEOUT))?;
    Ok(socket)
}

/// 메시지를 보내고 답을 기다립니다. 제한 시간 안에 답이 없으면 MAX_TRIES번까지 다시 보냅니다.
/// 돌려주는 값은 (받은 답, 보낸 횟수)입니다.
fn request(socket: &UdpSocket, message: &str) -> io::Result<(String, u32)> {
    let mut buf = [0u8; 2048];
    for attempt in 1..=MAX_TRIES {
        socket.send(message.as_bytes())?;
        match socket.recv(&mut buf) {
            Ok(n) => return Ok((text(&buf[..n]), attempt)),
            Err(e) if is_timeout(&e) => {} // 답이 없습니다. 반복을 이어 가 다시 보냅니다.
            Err(e) => return Err(e),
        }
    }
    let msg = format!("{MAX_TRIES}번 보냈지만 답이 없습니다");
    Err(io::Error::new(ErrorKind::TimedOut, msg))
}

/// 제한 시간 초과는 플랫폼에 따라 WouldBlock(유닉스 계열) 또는 TimedOut(윈도우)으로 옵니다.
fn is_timeout(e: &io::Error) -> bool {
    matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
