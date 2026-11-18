// 주소 타입: IP 주소(IpAddr)와 소켓 주소(SocketAddr), 그리고 ToSocketAddrs.
//
//   1) 주소 값은 생성자, 상수, From 변환으로 만듭니다. 네트워크를 전혀 쓰지 않는 평범한 값입니다.
//   2) 문자열에서는 parse(FromStr)로 만듭니다. parse는 이름(localhost 등)을 해석하지 않습니다.
//   3) is_loopback, is_private 같은 판별 메서드로 주소의 종류를 물어봅니다.
//   4) bind/connect가 받는 ToSocketAddrs는 여러 모양의 인자를 SocketAddr 목록으로 바꿔 줍니다.
//
// 이 예제는 소켓을 하나도 열지 않습니다. 4번 구역의 "localhost"만 운영체제에 이름 해석을 부탁합니다.
//
// 실행: cargo run -p p1-ch12-net --example 01_addresses

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};

fn main() {
    // ── 1. 주소 값 만들기 ────────────────────────────────────────────
    println!("1. 주소 값 만들기");
    let v4 = Ipv4Addr::new(127, 0, 0, 1);
    let v6 = Ipv6Addr::LOCALHOST; // ::1, IPv6의 루프백
    // IpAddr는 "IPv4 또는 IPv6"인 열거형입니다. 배열에서 바로 만들 수도 있습니다.
    let private: IpAddr = IpAddr::from([192, 168, 0, 10]);
    println!("    Ipv4Addr::new(127, 0, 0, 1) → {v4}");
    println!("    Ipv4Addr::LOCALHOST         → {}", Ipv4Addr::LOCALHOST);
    println!("    Ipv6Addr::LOCALHOST         → {v6}");
    println!("    IpAddr::from([192, 168, 0, 10]) → {private}");

    // 소켓 주소 = IP 주소 + 포트(u16). (IP, 포트) 튜플에서 From으로 만들 수 있습니다.
    let a = SocketAddr::new(IpAddr::V4(v4), 8080);
    let b = SocketAddr::from(([127, 0, 0, 1], 8080));
    let c = SocketAddr::from((v6, 8080));
    let d = SocketAddr::V4(SocketAddrV4::new(v4, 8080));
    println!("    SocketAddr::new(..)         → {a}");
    println!("    SocketAddr::from((v6, 8080)) → {c}  (IPv6는 대괄호로 감싸 표시)");
    // 출력: 네 가지 방법으로 만든 127.0.0.1:8080이 모두 같은가: true
    println!(
        "    네 가지 방법으로 만든 127.0.0.1:8080이 모두 같은가: {}",
        a == b && b == d
    );
    println!(
        "    c.ip() = {}, c.port() = {}, c.is_ipv6() = {}",
        c.ip(),
        c.port(),
        c.is_ipv6()
    );

    // ── 2. 문자열에서 파싱하기 ──────────────────────────────────────
    // SocketAddr, IpAddr, Ipv4Addr 등은 FromStr을 구현하므로 parse를 쓸 수 있습니다(Part 1 ch3).
    // 에러 타입은 모두 AddrParseError입니다.
    println!("2. 문자열에서 파싱하기");
    for text in [
        "127.0.0.1:8080",
        "[::1]:8080",
        "127.0.0.1",       // 포트가 없음
        "localhost:8080",  // parse는 이름을 해석하지 않음
        "::1:8080",        // IPv6 + 포트는 대괄호가 필요
        "127.0.0.1:70000", // 포트는 u16(0~65535)
    ] {
        // 출력: 앞의 둘은 Ok, 나머지 넷은 모두 Err(invalid socket address syntax)
        let parsed: Result<SocketAddr, _> = text.parse();
        match parsed {
            Ok(addr) => println!("    SocketAddr {text:<17} → Ok({addr})"),
            Err(e) => println!("    SocketAddr {text:<17} → Err({e})"),
        }
    }
    for text in ["0:0:0:0:0:0:0:1", "127.0.0.1:8080"] {
        match text.parse::<IpAddr>() {
            Ok(ip) => println!("    IpAddr     {text:<17} → Ok({ip})"),
            Err(e) => println!("    IpAddr     {text:<17} → Err({e})"),
        }
    }
    // 출력: IpAddr     0:0:0:0:0:0:0:1   → Ok(::1)   ← 0이 이어지는 부분은 ::로 줄여 표시합니다.

    // ── 3. 판별 메서드 ──────────────────────────────────────────────
    // is_private은 Ipv4Addr에만 있습니다(IPv6에서 비슷한 것은 is_unique_local).
    // IpAddr에는 두 버전에 공통인 판별만 있으므로, 버전별 판별은 match로 꺼내서 부릅니다.
    // 출력: 172.31.255.1은 사설(172.16.0.0/12 안), 172.32.0.1은 그 바로 바깥이라 false입니다.
    //       203.0.113.7은 문서 예시용으로 남겨 둔 주소라 어느 쪽도 아닙니다.
    println!("3. 판별 메서드");
    println!("    주소            루프백  사설    미지정  멀티캐스트");
    for text in [
        "127.0.0.1",
        "192.168.0.10",
        "172.31.255.1",
        "172.32.0.1",
        "203.0.113.7",
        "0.0.0.0",
        "224.0.0.1",
        "::1",
        "fd00::1",
        "::",
    ] {
        let ip: IpAddr = text.parse().expect("표의 주소는 모두 올바른 IP 주소입니다");
        let private = match ip {
            IpAddr::V4(v4) => v4.is_private(),
            IpAddr::V6(v6) => v6.is_unique_local(),
        };
        println!(
            "    {text:<15} {:<7} {:<7} {:<7} {}",
            ip.is_loopback(),
            private,
            ip.is_unspecified(),
            ip.is_multicast()
        );
    }

    // ── 4. ToSocketAddrs: bind/connect가 받아 주는 모양들 ───────────
    // TcpListener::bind, TcpStream::connect, UdpSocket::bind 등은 모두 A: ToSocketAddrs를 받습니다.
    // resolve는 같은 바운드를 가진 제네릭 함수라서, 이 함수가 받는 것은 connect도 받습니다.
    println!("4. ToSocketAddrs로 바꾼 결과");
    let pair = [a, c];
    resolve("a (SocketAddr)", a); // 출력: a (SocketAddr) → Ok([127.0.0.1:8080])
    resolve("\"127.0.0.1:8080\"", "127.0.0.1:8080");
    resolve("(\"127.0.0.1\", 8080)", ("127.0.0.1", 8080));
    resolve("(Ipv4Addr::LOCALHOST, 8080)", (Ipv4Addr::LOCALHOST, 8080));
    resolve("&pair[..]", &pair[..]); // 출력: → Ok([127.0.0.1:8080, [::1]:8080])
    resolve("\"127.0.0.1\"", "127.0.0.1"); // 출력: → Err(InvalidInput: invalid socket address)
    // "localhost"는 IP 주소로 파싱되지 않으므로 운영체제에 이름 해석을 부탁합니다.
    // 결과는 환경(/etc/hosts 등)에 따라 다르고, 개수와 순서도 달라질 수 있습니다.
    resolve("\"localhost:8080\"", "localhost:8080");
    // 출력(이 macOS 환경): "localhost:8080" → Ok([[::1]:8080, 127.0.0.1:8080])
}

fn resolve<A: ToSocketAddrs>(label: &str, addr: A) {
    // to_socket_addrs는 SocketAddr를 내놓는 반복자를 돌려줍니다. 이름 해석이 실패할 수 있어 io::Result입니다.
    match addr.to_socket_addrs() {
        Ok(iter) => {
            let list: Vec<SocketAddr> = iter.collect();
            println!("    {label:<28} → Ok({list:?})");
        }
        Err(e) => println!("    {label:<28} → Err({:?}: {e})", e.kind()),
    }
}
