// TCP 에코 서버와 클라이언트: 서버는 받은 바이트를 그대로 돌려줍니다.
//
//   1) 서버는 포트 0으로 bind합니다. 운영체제가 빈 포트를 고르고, local_addr()로 알아냅니다.
//   2) 서버는 accept()로 연결을 하나씩 받습니다. 연결(TcpStream)과 상대의 소켓 주소가 함께 옵니다.
//   3) 클라이언트는 connect → write_all → shutdown(Write)로 "더 보낼 것이 없다"를 알린 뒤
//      read_to_string으로 응답을 끝까지 읽습니다.
//   4) 서버의 read가 0을 돌려주면 EOF입니다. 반복을 끝내고 stream을 버리면(drop) 연결이 닫히고,
//      그제서야 클라이언트의 read_to_string이 끝납니다.
//
// 서버는 스레드 하나에서 연결 2개를 차례로 처리하고 끝납니다. 출력은 모두 main 스레드가 합니다.
// 포트 번호는 실행할 때마다 달라서 출력하지 않고, "0이 아닌가", "같은가" 같은 확인 결과만 출력합니다.
//
// 실행: cargo run -p p1-ch12-net --example 02_tcp_echo

use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::thread;

/// 서버가 연결 하나를 처리하고 남기는 기록입니다.
struct EchoRecord {
    peer: SocketAddr,
    bytes: usize,
}

fn main() -> io::Result<()> {
    // ── 1. 서버 준비: 포트 0으로 bind ──────────────────────────────
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let server_addr = listener.local_addr()?;
    println!("1. 서버 준비");
    println!("    서버 IP: {}", server_addr.ip()); // 출력: 서버 IP: 127.0.0.1
    let port_chosen = server_addr.port() != 0;
    println!("    운영체제가 고른 포트가 0이 아닌가: {port_chosen}"); // 출력: true

    // listener를 서버 스레드로 옮깁니다. 스레드가 끝나면 listener가 버려져 더는 연결을 받지 않습니다.
    let server = thread::spawn(move || -> io::Result<Vec<EchoRecord>> {
        let mut records = Vec::new();
        for _ in 0..2 {
            // accept는 연결이 들어올 때까지 이 스레드를 멈춰 세웁니다(블로킹).
            let (stream, peer) = listener.accept()?;
            let bytes = echo(stream)?;
            records.push(EchoRecord { peer, bytes });
        }
        Ok(records)
    });

    // ── 2·3. 클라이언트 두 개: 인자 모양만 다르고 하는 일은 같습니다 ──
    println!("2. 클라이언트 A: SocketAddr 값으로 connect");
    let a = run_client(server_addr, "안녕, 에코 서버!")?;
    // 출력: 상대 주소가 서버 주소와 같은가: true / 클라이언트 포트가 서버 포트와 다른가: true
    println!(
        "    상대 주소가 서버 주소와 같은가: {}",
        a.peer == server_addr
    );
    let different = a.local.port() != server_addr.port();
    println!("    클라이언트 포트가 서버 포트와 다른가: {different}");

    println!("3. 클라이언트 B: (Ipv4Addr, u16) 튜플로 connect");
    let b = run_client((Ipv4Addr::LOCALHOST, server_addr.port()), "두 번째 연결")?;
    let different = a.local.port() != b.local.port();
    println!("    A와 B의 클라이언트 포트가 다른가: {different}"); // 출력: true

    // ── 4. 서버 쪽 기록과 맞춰 보기 ────────────────────────────────
    // 서버의 accept가 알려 준 상대 주소 == 클라이언트 쪽 stream.local_addr()
    let records = server
        .join()
        .expect("서버 스레드는 패닉하지 않아야 합니다")?;
    println!("4. 서버 쪽 기록");
    for (record, client) in records.iter().zip([&a, &b]) {
        println!(
            "    {}바이트를 돌려줌, accept가 준 상대 주소 == 클라이언트의 local_addr: {}",
            record.bytes,
            record.peer == client.local
        );
    }
    // 출력:
    //     22바이트를 돌려줌, accept가 준 상대 주소 == 클라이언트의 local_addr: true
    //     17바이트를 돌려줌, accept가 준 상대 주소 == 클라이언트의 local_addr: true
    Ok(())
}

/// 서버 쪽: EOF를 만날 때까지 읽은 만큼 그대로 써서 돌려줍니다.
fn echo(mut stream: TcpStream) -> io::Result<usize> {
    let mut buf = [0u8; 1024];
    let mut total = 0;
    loop {
        // read는 "지금 와 있는 만큼"(최대 buf 길이) 읽고 그 바이트 수를 돌려줍니다.
        let n = stream.read(&mut buf)?;
        if n == 0 {
            // 0은 에러가 아니라 "상대가 더 보내지 않는다(EOF)"는 뜻입니다.
            return Ok(total);
        }
        stream.write_all(&buf[..n])?;
        total += n;
    }
    // 함수가 끝나면서 stream이 버려지고 연결이 닫힙니다.
}

/// 클라이언트 쪽 결과입니다.
struct ClientInfo {
    local: SocketAddr,
    peer: SocketAddr,
}

/// 클라이언트 쪽: 메시지를 보내고, 쓰기 방향을 닫고, 응답을 끝까지 읽습니다.
/// connect와 같은 바운드(A: ToSocketAddrs)를 받으므로 주소를 어떤 모양으로든 넘길 수 있습니다.
fn run_client<A: ToSocketAddrs>(addr: A, message: &str) -> io::Result<ClientInfo> {
    let mut stream = TcpStream::connect(addr)?;
    let local = stream.local_addr()?; // 운영체제가 클라이언트 쪽에 고른 임시 포트
    let peer = stream.peer_addr()?; // 연결한 상대(서버)의 주소

    stream.write_all(message.as_bytes())?;
    // 쓰기 방향만 닫습니다. 서버의 read가 0을 돌려받아 반복을 끝낼 수 있게 됩니다.
    // 이 줄이 없으면 서버는 다음 바이트를, 클라이언트는 응답의 끝을 서로 기다리며 멈춥니다.
    stream.shutdown(Shutdown::Write)?;

    let mut reply = String::new();
    stream.read_to_string(&mut reply)?; // 서버가 연결을 닫아 EOF가 올 때까지 읽습니다.
    println!("    보냄: {message:?} ({}바이트)", message.len());
    println!("    받음: {reply:?}");
    // 출력(A): 보냄: "안녕, 에코 서버!" (22바이트) / 받음: "안녕, 에코 서버!"
    Ok(ClientInfo { local, peer })
}
