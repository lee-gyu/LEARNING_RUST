// UDP 소켓: 연결 없이 데이터그램(한 덩어리의 메시지)을 주고받습니다.
//
//   1) bind한 소켓 하나로 send_to(받을 주소 지정)와 recv_from(보낸 주소 확인)을 합니다. accept도 connect도 없습니다.
//   2) 데이터그램 하나는 recv 한 번에 통째로 옵니다. 경계가 보존됩니다(05 예제의 TCP와 비교).
//   3) 버퍼보다 큰 데이터그램은 잘리고, 잘린 나머지는 버려집니다.
//   4) 받을 사람이 없어도 send_to는 성공합니다. 전달은 보장되지 않습니다.
//   5) connect로 상대를 고정하면 send/recv를 쓰고, 다른 주소에서 온 데이터그램은 걸러집니다.
//
// 루프백에서는 데이터그램이 사라질 일이 거의 없지만, 예제가 영원히 멈추지 않도록 받는 쪽에 제한 시간을 겁니다.
//
// 실행: cargo run -p p1-ch12-net --example 06_udp_datagrams

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

fn main() -> io::Result<()> {
    // ── 1. send_to와 recv_from ─────────────────────────────────────
    println!("1. send_to와 recv_from");
    let a = bound_socket()?;
    let b = bound_socket()?;
    let a_addr = a.local_addr()?;
    let b_addr = b.local_addr()?;
    let sent = a.send_to("안녕".as_bytes(), b_addr)?;
    let mut buf = [0u8; 64];
    let (n, from) = b.recv_from(&mut buf)?;
    println!("    a → b: send_to가 보낸 바이트: {sent}"); // 출력: 6
    println!(
        "    b가 받은 것: {:?}, 보낸 주소 == a의 주소: {}",
        text(&buf[..n]),
        from == a_addr
    );
    // 출력: b가 받은 것: "안녕", 보낸 주소 == a의 주소: true

    // ── 2. 경계가 보존됨 ───────────────────────────────────────────
    println!("2. send_to 세 번 → recv_from 세 번");
    for piece in ["안녕", "하세요", "!"] {
        a.send_to(piece.as_bytes(), b_addr)?;
    }
    for _ in 0..3 {
        let (n, _) = b.recv_from(&mut buf)?;
        println!("    {n}바이트 {:?}", text(&buf[..n]));
    }
    // 출력: 6바이트 "안녕" / 9바이트 "하세요" / 1바이트 "!"  ← TCP와 달리 붙지 않습니다.

    // ── 3. 버퍼보다 큰 데이터그램은 잘림 ───────────────────────────
    println!("3. 10바이트 데이터그램을 4바이트 버퍼로 받기");
    a.send_to(b"0123456789", b_addr)?;
    a.send_to("다음".as_bytes(), b_addr)?;
    let mut small = [0u8; 4];
    let (n, _) = b.recv_from(&mut small)?;
    println!("    첫 recv_from → {n}바이트 {:?}", text(&small[..n])); // 출력: 4바이트 "0123"
    let (n, _) = b.recv_from(&mut buf)?;
    // 남은 "456789"가 아니라 다음 데이터그램이 옵니다. 잘린 부분은 사라졌습니다.
    println!("    다음 recv_from → {n}바이트 {:?}", text(&buf[..n])); // 출력: 6바이트 "다음"

    // ── 4. 받을 사람이 없어도 보내기는 성공 ────────────────────────
    println!("4. 아무도 bind하지 않은 주소로 보내기");
    let nobody = closed_udp_addr()?;
    // 연결을 맺지 않으므로 상대가 있는지 확인할 단계가 없습니다. 데이터그램은 조용히 사라집니다.
    let result = a.send_to("여보세요?".as_bytes(), nobody);
    println!("    send_to → {}", show(&result)); // 출력: Ok(13)
    // connect한 소켓은 운영체제가 받은 "도착할 수 없음" 알림을 다음 recv의 에러로 받을 수 있습니다.
    // 상대 쪽 운영체제가 알림을 보내지 않거나 중간에 사라지면 오지 않으므로 기대서는 안 됩니다.
    let probe = bound_socket()?;
    probe.connect(nobody)?;
    println!("    connect 후 send → {}", show(&probe.send(b"hi"))); // 출력: Ok(2)
    println!("    이어서 recv     → {}", show(&probe.recv(&mut buf)));
    // 출력(이 macOS 환경): 이어서 recv     → Err(ConnectionRefused)

    // ── 5. connect로 상대 고정하기 ─────────────────────────────────
    println!("5. connect로 상대를 고정한 소켓");
    let c = bound_socket()?;
    let stranger = bound_socket()?;
    c.connect(a_addr)?; // 운영체제에 "a하고만 주고받는다"고 등록합니다. 패킷은 하나도 오가지 않습니다.
    // 출력: c.peer_addr() == a의 주소: true / connect하지 않은 a.peer_addr() → Err(NotConnected)
    println!(
        "    c.peer_addr() == a의 주소: {}",
        c.peer_addr()? == a_addr
    );
    println!(
        "    connect하지 않은 a.peer_addr() → {}",
        show(&a.peer_addr())
    );
    let c_addr = c.local_addr()?;
    stranger.send_to("모르는 사람".as_bytes(), c_addr)?; // 먼저 보냈지만 걸러집니다.
    a.send_to("아는 사람".as_bytes(), c_addr)?;
    let n = c.recv(&mut buf)?; // recv는 보낸 주소를 돌려주지 않습니다. 상대가 정해져 있기 때문입니다.
    println!("    c.recv → {:?}", text(&buf[..n])); // 출력: "아는 사람"
    c.set_read_timeout(Some(Duration::from_millis(100)))?;
    println!("    한 번 더 c.recv → {}", show(&c.recv(&mut buf))); // 출력(macOS): Err(WouldBlock)
    c.send("답장".as_bytes())?; // 받을 주소를 적지 않아도 a에게 갑니다.
    let (n, from) = a.recv_from(&mut buf)?;
    println!(
        "    a가 받은 답장: {:?}, 보낸 주소 == c의 주소: {}",
        text(&buf[..n]),
        from == c_addr
    );
    Ok(())
}

/// 127.0.0.1의 빈 포트에 bind하고, 받기에 1초 제한 시간을 건 소켓을 만듭니다.
fn bound_socket() -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;
    Ok(socket)
}

/// 잠깐 bind했다가 닫아서, 지금은 아무도 쓰지 않는 UDP 주소를 얻습니다.
fn closed_udp_addr() -> io::Result<SocketAddr> {
    UdpSocket::bind("127.0.0.1:0")?.local_addr()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn show<T: std::fmt::Debug>(result: &io::Result<T>) -> String {
    match result {
        Ok(v) => format!("Ok({v:?})"),
        Err(e) => format!("Err({:?})", e.kind()),
    }
}
