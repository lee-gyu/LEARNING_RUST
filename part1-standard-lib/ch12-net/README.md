# ch12. 네트워크: 주소 타입, TCP·UDP 소켓

[Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md)에서는 `TcpListener`로 연결을 받아 HTTP 응답을 돌려주는 웹 서버를 만들었습니다. 그때 "TCP·UDP 소켓 자세히 — 클라이언트 쪽 `TcpStream`, 타임아웃, UDP 송수신"을 이 챕터로 미뤄 두었습니다. 이 챕터는 HTTP와 스레드 풀이 아니라 그 아래의 `std::net` 자체를 다룹니다.

- **주소**: IP 주소와 소켓 주소를 값으로 만들고, 문자열에서 파싱하고, 종류를 판별합니다. `bind`와 `connect`가 여러 모양의 인자를 받는 이유(`ToSocketAddrs`)도 봅니다.
- **TCP**: 서버와 클라이언트를 직접 짜고, 연결을 닫는 방법(EOF와 `shutdown`), 읽기와 쓰기를 나누는 방법, 제한 시간과 에러 종류를 익힙니다. TCP가 메시지의 경계를 지켜 주지 않는다는 것을 확인하고, 경계를 나누는 두 가지 방법을 구현합니다.
- **UDP**: 연결 없이 데이터그램을 주고받고, TCP와 무엇이 다른지, 언제 무엇을 고를지 봅니다.

에코 서버(받은 것을 그대로 돌려주는 서버)를 TCP와 UDP로 각각 완성하는 것이 목표입니다.

- 패키지: `p1-ch12-net`
- 기준 문서: [`std::net`](https://doc.rust-lang.org/std/net/index.html)
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (Edition 2024), macOS (에러의 종류와 메시지, 소켓 버퍼 크기, UDP 데이터그램의 최대 크기는 운영체제마다 다를 수 있습니다)
- 네트워크: 모든 예제는 이 컴퓨터 자신을 가리키는 `127.0.0.1`에서만 소켓을 열고, 포트 번호는 운영체제가 고르게 합니다(2.4절). 서버와 클라이언트를 한 프로세스 안의 스레드로 띄우므로 터미널 하나로 실행하면 몇 초 안에 스스로 끝납니다. 외부 호스트에 연결하지 않으며, 이름 해석은 01 예제의 `localhost` 하나뿐입니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. `IpAddr`·`Ipv4Addr`·`Ipv6Addr`·`SocketAddr`를 생성자와 `parse`로 만들고 `is_loopback`·`is_private` 등으로 판별할 수 있으며, `ToSocketAddrs`가 받는 인자의 모양과 `parse`가 이름을 해석하지 않는 이유를 설명할 수 있다.
2. 포트 0과 `local_addr`로 서버를 띄우고 `accept`·`connect`로 TCP 에코 서버와 클라이언트를 작성할 수 있으며, `shutdown(Shutdown::Write)`과 `read`가 돌려주는 0(EOF)의 관계를 설명할 수 있다.
3. 한 스레드에서 "다 쓰고 나서 읽기"가 큰 데이터에서 멈추는 이유를 설명하고, `try_clone`이나 `&TcpStream`과 스코프 스레드로 읽기와 쓰기를 나눌 수 있다.
4. `connect_timeout`, `set_read_timeout`/`set_write_timeout`, `set_nonblocking`을 쓰고, 실패를 `ErrorKind`(`ConnectionRefused`, `AddrInUse`, `InvalidInput`, `WouldBlock`/`TimedOut`)로 구분해 처리할 수 있다.
5. TCP는 경계 없는 바이트 스트림이고 UDP는 경계가 보존되지만 전달이 보장되지 않는 데이터그램이라는 차이를 예제로 보일 수 있고, 줄 단위·길이 접두사 프레이밍과 제한 시간·재전송을 구현해 상황에 맞게 둘 중 하나를 고를 수 있다.

## 2. 핵심 개념

### 2.1 `std::net`의 지도

**소켓(socket)** 은 운영체제가 관리하는 통신의 끝점입니다. `TcpStream`이나 `UdpSocket` 같은 Rust 값은 그 소켓을 가리키는 핸들이고, 읽기와 쓰기는 결국 운영체제가 소켓마다 둔 버퍼와 바이트를 주고받는 일입니다. 파일처럼 **값이 버려지면(drop) 소켓이 닫힙니다.**

| 타입 | 역할 | 절 |
| --- | --- | --- |
| `IpAddr` (`Ipv4Addr`, `Ipv6Addr`) | IP 주소 | 2.2 |
| `SocketAddr` (`SocketAddrV4`, `SocketAddrV6`) | IP 주소 + 포트 | 2.2 |
| `AddrParseError` | 주소 문자열 파싱 실패 | 2.2 |
| `ToSocketAddrs` | `bind`·`connect`의 인자를 소켓 주소 목록으로 바꾸는 트레이트 | 2.3 |
| `TcpListener` | 연결을 기다리는 TCP 소켓 (리스너) | 2.4 |
| `TcpStream` | TCP 연결 하나. 바이트 스트림을 읽고 씀 | 2.4~2.8 |
| `Shutdown` | 연결의 한쪽 방향만 닫을 때 넘기는 값 | 2.5 |
| `UdpSocket` | 데이터그램을 보내고 받는 UDP 소켓 | 2.9 |

앞 챕터에서 배운 것이 이렇게 쓰입니다.

| 쓰이는 곳 | 배운 곳 |
| --- | --- |
| `TcpStream`의 `read`/`write_all`/`read_to_end`, `BufReader`, `io::Error`와 `ErrorKind` | `ch9-io-fs-path` |
| 서버를 스레드로 띄우고 `join`으로 결과 돌려받기, `move` 클로저 | [Part 0 ch15](../../part0-getting-started/ch15-concurrency/README.md) |
| 스코프 스레드로 지역 변수를 빌려 쓰기 | `ch11-threads-sync` |
| 제한 시간에 쓰는 `Duration`, 걸린 시간을 재는 `Instant` | `ch10-env-process-time` |
| `"127.0.0.1:8080".parse::<SocketAddr>()` | [ch3](../ch3-strings-fmt/README.md)의 `FromStr` |
| `u32::try_from`, `to_be_bytes`/`from_be_bytes` | [ch6](../ch6-core-traits/README.md), [ch1](../ch1-std-overview/README.md) |

### 2.2 IP 주소와 소켓 주소

**IP 주소** 는 네트워크에서 컴퓨터(정확히는 네트워크 인터페이스)를 가리키고, **포트(port)** 는 그 컴퓨터 안에서 프로그램을 가리키는 `u16` 번호입니다. 둘을 합친 것이 **소켓 주소(socket address)** 이고, 연결이나 데이터그램의 양 끝은 각각 소켓 주소 하나로 나타냅니다.

IP 주소에는 4바이트짜리 IPv4와 16바이트짜리 IPv6가 있습니다. 표준 라이브러리는 버전별 타입(`Ipv4Addr`, `Ipv6Addr`)과 "둘 중 하나"를 담는 열거형(`IpAddr`)을 함께 제공합니다. 소켓 주소도 같은 구조입니다(`SocketAddrV4`, `SocketAddrV6`, 열거형 `SocketAddr`). 대부분의 API가 열거형을 받으므로 코드 하나로 두 버전을 다룰 수 있습니다.

```rust
let v4 = Ipv4Addr::new(127, 0, 0, 1);
let ip: IpAddr = IpAddr::from([192, 168, 0, 10]);       // 배열에서
let a = SocketAddr::new(IpAddr::V4(v4), 8080);
let b = SocketAddr::from(([127, 0, 0, 1], 8080));        // (IP, 포트) 튜플에서
let c = SocketAddr::from((Ipv6Addr::LOCALHOST, 8080));   // [::1]:8080
```

주소 값을 만드는 것은 네트워크와 아무 관계가 없습니다. 정수나 문자열처럼 복사하고(`Copy`), 비교하고, 해시 맵의 키로 쓸 수 있는 평범한 값입니다.

**문자열에서 파싱하기.** 주소 타입은 모두 `FromStr`을 구현하므로 `parse`로 만들 수 있고, 실패하면 `AddrParseError`를 돌려줍니다([`01_addresses.rs`](examples/01_addresses.rs)의 2번 구역).

| 문자열 | `SocketAddr`로 파싱 | 이유 |
| --- | --- | --- |
| `"127.0.0.1:8080"` | `Ok` | |
| `"[::1]:8080"` | `Ok` | IPv6 주소에는 `:`가 들어 있어서 포트와 구분하려고 대괄호로 감쌉니다. |
| `"127.0.0.1"` | `Err` | 포트가 없습니다. 이 문자열은 `IpAddr`로는 파싱됩니다. |
| `"localhost:8080"` | `Err` | **`parse`는 이름을 해석하지 않습니다.** 숫자로 적힌 주소만 받습니다. |
| `"::1:8080"` | `Err` | 대괄호가 없으면 어디까지가 주소인지 알 수 없습니다. |
| `"127.0.0.1:70000"` | `Err` | 포트는 `u16`(0~65535)입니다. |

에러 메시지는 모두 `invalid socket address syntax`입니다. 무엇이 틀렸는지까지는 알려 주지 않으므로, 사용자 입력이라면 어떤 모양을 기대하는지 메시지에 덧붙여 주는 편이 좋습니다. 출력할 때는 표준 표기로 바뀝니다. `"0:0:0:0:0:0:0:1"`을 `IpAddr`로 파싱해 출력하면 `::1`입니다.

**판별 메서드.** 주소가 어떤 종류인지 물어보는 메서드가 있습니다(01 예제의 3번 구역).

| 메서드 | 참이 되는 주소 | 있는 곳 |
| --- | --- | --- |
| `is_loopback` | `127.0.0.0/8`, `::1` — 이 컴퓨터 자신(루프백) | `IpAddr`, `Ipv4Addr`, `Ipv6Addr` |
| `is_unspecified` | `0.0.0.0`, `::` — "특정 주소 아님" | 세 타입 모두 |
| `is_multicast` | `224.0.0.0/4`, `ff00::/8` | 세 타입 모두 |
| `is_private` | `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` — 사설 주소 | `Ipv4Addr`만 |
| `is_unique_local` | `fc00::/7` — IPv6에서 사설 주소에 해당하는 범위 (1.84 안정화) | `Ipv6Addr`만 |
| `is_documentation` | `192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24` — 문서 예시용 | `Ipv4Addr`만 |

`IpAddr`에는 두 버전에 공통인 판별만 있으므로, 버전별 판별은 `match`로 꺼내서 부릅니다. "인터넷에서 닿을 수 있는 주소인가"를 묻는 `is_global`은 1.100.0에서 nightly 전용 실험 API라 쓸 수 없습니다.

```rust
let private = match ip {
    IpAddr::V4(v4) => v4.is_private(),
    IpAddr::V6(v6) => v6.is_unique_local(),
};
```

**루프백과 `0.0.0.0`.** `127.0.0.1`로 보낸 데이터는 컴퓨터 밖으로 나가지 않습니다. 이 챕터의 예제가 모두 루프백만 쓰는 이유입니다. 반대로 서버를 `0.0.0.0`(미지정 주소)에 bind하면 "이 컴퓨터의 모든 네트워크 인터페이스"에서 연결을 받으므로 같은 네트워크의 다른 컴퓨터도 접속할 수 있게 됩니다. 연습용 서버는 `127.0.0.1`에 bind하는 습관을 들이세요.

### 2.3 `ToSocketAddrs`: `bind`와 `connect`의 인자가 유연한 이유

[Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md)에서는 `bind("127.0.0.1:7878")`처럼 문자열을 넘겼습니다. 시그니처를 보면 인자가 `&str`가 아닙니다.

```rust
pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream>
```

`ToSocketAddrs`는 "소켓 주소 **목록**으로 바뀔 수 있는 것"을 뜻하는 트레이트이고, `to_socket_addrs()`가 `SocketAddr`를 내놓는 반복자를 돌려줍니다. 1.100.0 문서의 구현 목록은 다음과 같습니다.

| 인자의 모양 | 바뀌는 방법 |
| --- | --- |
| `SocketAddr`, `SocketAddrV4`, `SocketAddrV6` | 그대로 하나 |
| `(IpAddr, u16)`, `(Ipv4Addr, u16)`, `(Ipv6Addr, u16)` | 그대로 하나 |
| `(&str, u16)`, `(String, u16)` | 문자열이 IP 주소면 그대로, 아니면 호스트 이름으로 보고 해석 |
| `&str`, `String` | `"호스트:포트"` 모양. 소켓 주소로 파싱되면 그대로, 아니면 이름 해석 |
| `&[SocketAddr]` | 슬라이스의 주소 전부 |
| `&T` (`T: ToSocketAddrs`) | `T`와 같음 |

`TcpListener::bind`, `TcpStream::connect`, `UdpSocket::bind`/`connect`/`send_to`가 모두 이 바운드를 씁니다. 그래서 설정 파일에서 읽은 문자열도, 프로그램이 계산한 `SocketAddr`도 변환 없이 바로 넘길 수 있습니다. 문서는 이 트레이트가 있는 이유를 "간단한 경우에는 `"localhost:12345"` 같은 문자열이 편하지만, `SocketAddr`가 이미 있는데 문자열로 바꿨다가 다시 파싱하는 것은 무의미하기 때문"이라고 설명합니다. 01 예제의 `resolve` 함수는 같은 바운드(`A: ToSocketAddrs`)를 가진 제네릭 함수로, 각 모양이 무엇으로 바뀌는지 출력합니다.

**왜 목록인가.** 이름 하나가 주소 여러 개로 해석될 수 있기 때문입니다. 이 macOS 환경에서 `"localhost:8080"`은 `[::1]:8080`과 `127.0.0.1:8080` 두 개로 해석되었습니다(환경마다 개수와 순서가 다를 수 있습니다). `connect`와 `bind`는 목록의 주소를 차례로 시도해 처음 성공한 것을 쓰고, 모두 실패하면 마지막 시도의 에러를 돌려줍니다. `UdpSocket::send_to`는 첫 번째 주소로만 보냅니다.

**`parse`와 이름 해석.** 문자열은 소켓 주소로 파싱되면 그대로 쓰이고, 파싱되지 않을 때 운영체제에 이름 해석(name resolution)을 부탁합니다. 문서에 따르면 이름 해석은 **현재 스레드를 멈추게 할 수 있고**(DNS 서버에 묻는 동안), 네트워크 상태에 따라 실패할 수 있습니다. 반면 `parse::<SocketAddr>()`는 문자열만 보고 즉시 끝나며 이름은 받지 않습니다. 운영체제의 해석기는 `parse`보다 너그러워서 결과가 다를 수도 있습니다. 이 환경에서 `"127.1:80"`은 `parse`로는 실패했지만 `to_socket_addrs()`로는 `127.0.0.1:80`이 되었습니다(해석기가 옛 축약 표기를 받아 줌). 숫자 주소만 받을 곳이라면 `parse`로 먼저 검사하는 것이 안전합니다.

`connect_timeout`만은 `&SocketAddr` 하나를 받습니다. 제한 시간은 주소 하나에 대해 재는 것이라서, 여러 주소로 해석될 수 있는 인자는 받지 않습니다(4.5절).

### 2.4 TCP 서버와 클라이언트

TCP는 두 프로그램 사이에 **연결(connection)** 을 맺고, 그 연결로 바이트를 빠짐없이 보낸 순서대로 전달합니다. [`02_tcp_echo.rs`](examples/02_tcp_echo.rs)가 서버와 클라이언트의 기본 모양입니다.

**서버.** `TcpListener::bind`로 주소와 포트를 차지하고 연결을 기다립니다(리스너). 포트에 **0** 을 주면 운영체제가 비어 있는 포트를 골라 주고, 고른 포트는 `local_addr()`로 알아냅니다. 고정 포트를 쓰면 다른 프로그램이나 이전에 띄운 서버와 충돌할 수 있으므로(4.6절의 `AddrInUse`), 테스트와 예제에서는 포트 0이 기본입니다.

```rust
let listener = TcpListener::bind("127.0.0.1:0")?;
let server_addr = listener.local_addr()?;   // 예: 127.0.0.1:52713 (포트는 실행마다 다름)

let (stream, peer) = listener.accept()?;    // 연결이 올 때까지 기다림(블로킹)
```

`accept`는 연결 하나를 꺼내 그 연결의 `TcpStream`과 **상대의 소켓 주소**를 함께 돌려줍니다. Part 0 ch20에서 쓴 `incoming()`은 `accept`를 반복하는 반복자로, 상대 주소는 주지 않고 `None`을 돌려주지 않아 끝나지 않습니다. 서버가 정해진 수의 연결만 처리하고 끝나야 하는 이 챕터의 예제는 `accept`를 정해진 횟수만큼 부릅니다.

**클라이언트.** `TcpStream::connect`가 연결을 맺어 `TcpStream`을 돌려줍니다. 클라이언트도 소켓 주소가 필요한데, 따로 bind하지 않으면 운영체제가 **임시 포트(ephemeral port)** 를 골라 줍니다.

**주소의 짝.** 연결 하나에는 끝이 둘 있고, 각 `TcpStream`은 자기 쪽(`local_addr`)과 상대 쪽(`peer_addr`) 주소를 압니다. 02 예제는 포트 번호를 출력하지 않는 대신 짝이 맞는지 확인합니다.

| 서버 쪽에서 본 값 | | 클라이언트 쪽에서 본 값 |
| --- | --- | --- |
| `listener.local_addr()` | == | `stream.peer_addr()` |
| `accept`가 준 상대 주소 | == | `stream.local_addr()` (임시 포트) |

**서버가 `accept`하기 전에도 `connect`는 성공합니다.** 연결을 맺는 절차(핸드셰이크)는 운영체제가 처리하고 맺어진 연결을 대기열에 넣어 두며, `accept`는 그 대기열에서 하나를 꺼낼 뿐입니다. 04 예제의 5번 구역은 `accept`를 부르지 않는 리스너에 `connect`가 성공하는 것을 보여 줍니다.

### 2.5 연결 닫기: EOF와 `shutdown`

에코 서버의 본체는 "읽은 만큼 써서 돌려주기"를 반복합니다.

```rust
fn echo(mut stream: TcpStream) -> io::Result<usize> {
    let mut buf = [0u8; 1024];
    let mut total = 0;
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            return Ok(total);              // EOF: 상대가 더 보내지 않음
        }
        stream.write_all(&buf[..n])?;
        total += n;
    }
}                                          // stream이 버려지며 연결이 닫힘
```

TCP 연결은 **방향이 둘인 바이트 스트림** 입니다. 소켓에서 `read`가 `Ok(0)`을 돌려주는 것은 에러가 아니라 **EOF**, 즉 "상대가 자기 쪽 쓰기 방향을 닫았다(더 보낼 것이 없다)"는 뜻입니다. 데이터가 아직 오지 않았을 때는 0을 돌려주지 않고 올 때까지 기다리므로, 0은 반복을 끝낼 확실한 신호입니다.

그렇다면 클라이언트는 "다 보냈다"를 어떻게 알릴까요? 연결을 버리면(drop) 응답을 받을 수 없게 됩니다. `shutdown`은 연결의 **한쪽 방향만** 닫습니다.

```rust
stream.write_all(message.as_bytes())?;
stream.shutdown(Shutdown::Write)?;   // 쓰기 방향만 닫음 → 서버의 read가 0을 받음
let mut reply = String::new();
stream.read_to_string(&mut reply)?;  // 읽기 방향은 열려 있음. 서버가 닫으면 EOF
```

| `Shutdown` | 동작 (문서) |
| --- | --- |
| `Write` | 쓰기 방향을 닫습니다. 상대의 `read`는 남은 데이터를 다 읽은 뒤 0을 받습니다. 이후 이쪽의 쓰기는 에러입니다(이 환경에서 `BrokenPipe`). |
| `Read` | 읽기 방향을 닫습니다. 이후 이쪽의 읽기는 `Ok(0)`입니다. |
| `Both` | 둘 다 닫습니다. |

`shutdown`을 빠뜨리면 서버는 다음 바이트를, 클라이언트는 응답의 끝을 서로 기다리며 멈춥니다. 04 예제의 4번 구역이 이 상황을 제한 시간으로 끊어서 보여 주고, Part 0 ch20의 2.3절에서 `read_to_string`으로 요청을 읽으려던 서버가 멈춘 것도 같은 원인입니다. HTTP는 "빈 줄"과 `Content-Length`로 메시지의 끝을 알려서 이 문제를 피합니다. 이렇게 메시지의 끝을 정하는 방법은 2.8절에서 다룹니다.

알아 둘 점이 두 가지 더 있습니다.

- 상대가 이미 닫은 연결에 쓰면 에러가 납니다. 이 환경에서는 상대가 연결을 버린 뒤 첫 `write`는 성공하고 두 번째 `write`가 `BrokenPipe`로 실패했습니다. 유닉스에서 이런 쓰기는 프로세스를 끝내는 `SIGPIPE` 시그널을 일으킬 수 있는데, `TcpStream` 문서에 따르면 Rust는 이를 막는 플래그(`MSG_NOSIGNAL`)를 붙여 쓰므로 에러 값으로만 받습니다.
- 문서에 따르면 `shutdown`을 같은 방향으로 두 번 부를 때의 동작은 운영체제마다 다릅니다. 이 macOS 환경에서 두 번째 호출은 `NotConnected` 에러였습니다(리눅스는 `Ok(())`).

### 2.6 읽기와 쓰기 나누기: `try_clone`과 `&TcpStream`

02의 클라이언트는 "다 쓰고, 쓰기를 닫고, 다 읽는" 순서로 동작합니다. 메시지가 작으면 문제가 없지만, 크면 멈춥니다. [`03_split_read_write.rs`](examples/03_split_read_write.rs)의 2번 구역은 64 MiB를 이 방식으로 에코 서버에 보냅니다.

```text
클라이언트 ─쓰기→ [클라 송신 버퍼] → [서버 수신 버퍼] ─읽기→ 서버
클라이언트 ←읽기─ [클라 수신 버퍼] ← [서버 송신 버퍼] ←쓰기─ 서버
```

클라이언트가 읽지 않으니 아래 줄의 버퍼가 가득 차고, 서버의 쓰기가 멈춥니다. 쓰기에서 멈춘 서버는 더 읽지 않으니 위 줄의 버퍼도 가득 차고, 클라이언트의 쓰기도 멈춥니다. 서로 상대가 움직이기를 기다리는 **교착 상태(deadlock)** 입니다. 예제는 쓰기 제한 시간(300ms)을 걸어 `write_all`이 `Err(WouldBlock)`으로 돌아오게 해서 이를 확인합니다. 운영체제는 버퍼 크기를 상황에 따라 늘리므로 멈추기 전까지 쓴 양은 실행마다 달랐습니다(이 환경에서 수 MiB에서 13 MiB 정도). 64 MiB는 그보다 확실히 크게 고른 값입니다.

해결책은 **읽기와 쓰기를 동시에** 하는 것이고, 그러려면 같은 연결을 두 스레드가 써야 합니다. 방법은 두 가지입니다.

**① `try_clone`: 같은 소켓의 핸들을 하나 더 만듭니다.** 돌려받은 `TcpStream`은 따로 소유한 값이라 `thread::spawn`에 옮길 수 있습니다(3번 구역).

```rust
let mut reader = TcpStream::connect(addr)?;
let mut writer = reader.try_clone()?;
let handle = thread::spawn(move || -> io::Result<()> {
    writer.write_all(&to_send)?;
    writer.shutdown(Shutdown::Write)
});
reader.read_to_end(&mut received)?;   // 쓰는 동안에도 계속 읽어 줌
handle.join().expect("쓰기 스레드는 패닉하지 않아야 합니다")?;
```

두 핸들은 같은 소켓을 가리키므로 한쪽에서 바꾼 설정(제한 시간 등)이 다른 쪽에도 보이고, `shutdown`은 어느 핸들로 불러도 연결에 작용합니다(1번 구역). 반면 핸들 하나를 버려도 연결은 닫히지 않습니다. 이 환경에서 원본을 버린 뒤에도 복제본으로 계속 읽고 쓸 수 있었고, 핸들을 모두 버리자 상대의 `read`가 0을 받았습니다. 이름이 `clone`이 아니라 `try_clone`인 것은 운영체제에 핸들을 하나 더 만들어 달라고 부탁하는 일이라 실패할 수 있기 때문입니다(`io::Result`).

**② `&TcpStream`: 빌린 참조로 읽고 씁니다.** `Read`와 `Write`는 `TcpStream`뿐 아니라 `&TcpStream`에도 구현되어 있습니다. 그래서 `mut` 없이 `(&stream).write_all(..)`로 쓸 수 있고, `TcpStream`은 `Sync`이므로 여러 스레드가 `&TcpStream`을 함께 빌릴 수 있습니다. 스코프 스레드(`thread::scope`, 스코프가 끝나기 전에 모든 스레드를 `join`하므로 지역 변수를 빌릴 수 있는 스레드 — `ch11-threads-sync`)와 함께 쓰면 핸들을 늘리지 않고 나눌 수 있습니다(4번 구역).

```rust
let received = thread::scope(|s| -> io::Result<Vec<u8>> {
    let writer = s.spawn(|| -> io::Result<()> {
        (&stream).write_all(&payload)?;
        stream.shutdown(Shutdown::Write)
    });
    let mut received = Vec::new();
    (&stream).read_to_end(&mut received)?;
    writer.join().expect("쓰기 스레드는 패닉하지 않아야 합니다")?;
    Ok(received)
})?;
```

공유 참조로 "쓰기"를 해도 되는 이유는 연결의 상태가 Rust 값이 아니라 운영체제 안에 있기 때문입니다. 읽기와 쓰기는 각각 운영체제 호출 하나이고, 동시에 와도 운영체제가 질서를 지켜 줍니다(`&File`이 `Read`/`Write`인 것과 같은 이유입니다). 다만 **두 스레드가 동시에 쓰면 바이트가 섞일 수 있으므로** 메시지 단위의 질서는 지켜 주지 않습니다. 보통 쓰는 스레드 하나, 읽는 스레드 하나로 나눕니다.

이 성질 덕분에 03 예제의 에코 서버는 한 줄입니다. `io::copy`는 EOF까지 읽은 것을 모두 써 주고 바이트 수를 돌려주는데, 같은 `stream`을 읽는 쪽과 쓰는 쪽으로 동시에 빌립니다.

```rust
fn echo(stream: TcpStream) -> io::Result<u64> {
    io::copy(&mut &stream, &mut &stream)
}
```

| 방법 | 쓰는 때 |
| --- | --- |
| `try_clone` | 핸들을 `thread::spawn`처럼 `'static`을 요구하는 곳에 옮겨야 할 때, 읽는 쪽과 쓰는 쪽을 서로 다른 구조체에 나눠 담을 때 |
| `&TcpStream` | 스코프 스레드나 한 함수 안에서 잠깐 나눠 쓸 때, `BufReader::new(&stream)`으로 읽으면서 같은 `stream`에 쓸 때(05 예제) |

### 2.7 제한 시간과 에러 종류

소켓의 `accept`, `connect`, `read`, `write`는 기본적으로 **블로킹** 입니다. 할 수 있을 때까지 스레드를 멈추고 기다립니다. 상대가 응답하지 않으면 영원히 기다릴 수 있으므로, 실제 프로그램에서는 제한 시간(timeout)을 거는 것이 좋습니다.

| 메서드 | 하는 일 |
| --- | --- |
| `TcpStream::connect_timeout(&addr, dur)` | 연결을 `dur`까지만 기다립니다. 주소는 `&SocketAddr` 하나입니다. |
| `set_read_timeout(Some(dur))` / `set_write_timeout(..)` | 이후의 읽기·쓰기가 `dur`보다 오래 걸리면 에러를 돌려줍니다. `None`은 "제한 없음"입니다. |
| `read_timeout()` / `write_timeout()` | 지금 설정을 돌려줍니다. |

제한 시간에 `Duration::ZERO`를 주면 `InvalidInput` 에러입니다. "기다리지 않음"과 "제한 없음"을 헷갈리지 않도록 0을 거부하는 것입니다. 제한 시간 초과의 `ErrorKind`는 **플랫폼마다 다릅니다.** 문서는 유닉스 계열이 보통 `WouldBlock`, 윈도우는 `TimedOut`을 돌려줄 수 있다고 적고 있고, 이 macOS 환경에서는 `WouldBlock`이었습니다. 그래서 예제는 둘을 함께 검사하는 도우미를 씁니다.

```rust
fn is_timeout(e: &io::Error) -> bool {
    matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}
```

제한 시간이 지나도 연결은 그대로 쓸 수 있습니다. 04 예제의 4번 구역은 시간 초과 뒤 `shutdown`하고 다시 읽어 EOF까지 정상적으로 받습니다. `read_to_end`는 에러로 끝나도 그때까지 읽은 바이트를 버퍼에 남겨 둔다는 점(문서에 명시)도 확인합니다.

네트워크 에러는 메시지 문구가 운영체제마다 다르므로 `ErrorKind`로 구분합니다. 이 챕터에서 실제로 만난 것은 다음과 같습니다(괄호 안은 macOS의 메시지).

| `ErrorKind` | 언제 | 예제 |
| --- | --- | --- |
| `ConnectionRefused` | 아무도 기다리지 않는 포트에 `connect` (`Connection refused (os error 61)`) | 04 |
| `AddrInUse` | 이미 쓰이는 주소에 `bind` (`Address already in use (os error 48)`) | 04 |
| `InvalidInput` | 제한 시간 0, 포트 없는 `"127.0.0.1".to_socket_addrs()` | 01, 04 |
| `WouldBlock` | 제한 시간 초과(이 환경), 논블로킹 모드에서 할 일이 없음 (`Resource temporarily unavailable (os error 35)`) | 03, 04, 06 |
| `NotConnected` | `connect`하지 않은 UDP 소켓의 `peer_addr` | 06 |
| `UnexpectedEof` | `read_exact`가 버퍼를 다 채우기 전에 EOF | 05 |
| `BrokenPipe` | 닫힌 연결에 쓰기 (`Broken pipe (os error 32)`) | 03(종류는 출력하지 않음) |

`connect_timeout`은 "응답이 없을 때" 기다림을 끊는 것이지 거부까지 기다리게 하지는 않습니다. 닫힌 포트에는 운영체제가 곧바로 거부를 알려 주므로 `connect_timeout`도 제한 시간 전에 `ConnectionRefused`로 돌아옵니다(04의 1번 구역). 진짜로 시간이 초과되는 상황(패킷이 사라지는 먼 호스트)은 루프백만으로 재현할 수 없어 이 챕터의 예제에서는 다루지 않습니다.

**논블로킹 모드.** `set_nonblocking(true)`를 부르면 기다리지 않고 곧바로 돌아옵니다. 할 일이 없으면 `WouldBlock` 에러인데, 이것은 실패가 아니라 "지금은 없으니 나중에 다시 시도하라"는 뜻입니다(04의 5번 구역). 소켓 여러 개를 논블로킹으로 두고 준비된 것만 골라 처리하는 것이 **이벤트 루프**이고, 비동기 런타임이 안에서 하는 일입니다. 표준 라이브러리에는 "준비될 때까지 기다리는" 기능(운영체제의 `epoll`·`kqueue` 등)이 없어서 이벤트 루프는 보통 크레이트로 만듭니다. 퓨처와 실행기의 원리는 `ch13-async-primitives`에서, 비동기 런타임은 [Part 0 ch16](../../part0-getting-started/ch16-async/README.md)에서 맛봤습니다.

### 2.8 TCP는 바이트 스트림: 메시지 경계 나누기

TCP가 보장하는 것은 "보낸 바이트가 빠짐없이 순서대로 도착한다"까지입니다. **`write` 한 번이 `read` 한 번으로 온다는 보장은 없습니다.** [`05_message_framing.rs`](examples/05_message_framing.rs)의 1·2번 구역이 두 방향을 모두 보여 줍니다.

- `write_all`을 세 번(`"안녕"`, `"하세요"`, `"!"`) 한 것이 `read` 한 번에 16바이트로 붙어서 왔습니다.
- `write_all`을 한 번(16바이트) 한 것을 5바이트 버퍼로 읽자 `read`가 네 번(5, 5, 5, 1바이트) 필요했고, 한글 한 글자(3바이트)가 조각 경계에서 잘려 조각을 따로 UTF-8로 풀면 깨졌습니다.

`read`는 "지금 와 있는 만큼, 최대 버퍼 길이까지" 돌려줄 뿐입니다. 어디까지가 메시지 하나인지는 **주고받는 쪽이 약속한 규칙** 으로 정해야 하고, 이것을 메시지 경계 나누기(프레이밍, framing)라고 합니다. 흔한 방법은 두 가지입니다.

**① 구분자: 줄 단위.** 메시지 끝에 `'\n'`을 붙이고, 받는 쪽은 `BufReader`의 `read_line`이나 `lines()`로 줄 끝까지 모읍니다(3번 구역). `BufReader`는 `read`가 돌려준 조각을 버퍼에 모아 두었다가 `'\n'`이 나올 때까지 이어 붙이므로, 보낸 쪽이 줄 중간에서 끊어 보내도 줄 단위로 정확히 나옵니다. 사람이 읽을 수 있고 디버깅이 쉬워서 텍스트 프로토콜(HTTP의 요청 줄과 헤더 등)이 많이 씁니다. 대신 메시지 안에 `'\n'`이 들어가면 안 되고, 상대가 줄 끝을 보내지 않으면 줄이 한없이 길어질 수 있습니다.

```rust
let reader = BufReader::new(&stream);   // &TcpStream으로 감싸서
let mut writer = &stream;               // 같은 stream에 계속 쓸 수 있게
for line in reader.lines() {
    let line = line?;
    writer.write_all(format!("에코: {line}\n").as_bytes())?;
}
```

**② 길이 접두사.** 메시지 앞에 내용의 바이트 수를 고정 크기 정수로 붙입니다(4번 구역). 받는 쪽은 길이 4바이트를 읽고, 그만큼을 읽습니다. 내용에 무엇이 들어 있어도(줄바꿈, 바이너리, 빈 메시지) 경계가 정확합니다.

```rust
fn write_frame(mut w: impl Write, payload: &[u8]) -> io::Result<()> {
    let len = u32::try_from(payload.len()).map_err(|_| /* InvalidInput */)?;
    w.write_all(&len.to_be_bytes())?;          // 길이: u32 빅 엔디언 4바이트
    w.write_all(payload)
}

fn read_frame(mut r: impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; 4];
    match r.read_exact(&mut header) {          // 조각나서 와도 4바이트를 모음
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_be_bytes(header);
    if len > MAX_FRAME { /* InvalidData 에러 */ }
    let mut payload = vec![0u8; len as usize];
    r.read_exact(&mut payload)?;
    Ok(Some(payload))
}
```

- `read_exact`는 버퍼를 다 채울 때까지 `read`를 되풀이합니다. 그전에 EOF를 만나면 `UnexpectedEof`입니다. 예제의 `read_frame`은 헤더에서 만난 EOF를 "메시지 사이에서 연결이 끝남"으로 보고 `None`을 돌려주는데, 헤더 중간에서 끊긴 경우와 구분하지 않는 단순화입니다(연습 문제 2).
- 정수를 바이트로 보낼 때는 바이트 순서(엔디언)를 양쪽이 약속해야 합니다. 네트워크 프로토콜은 관례적으로 빅 엔디언을 씁니다.
- **상대가 보낸 길이를 그대로 믿고 할당하지 않습니다.** 헤더 하나가 `0xFFFFFFFF`면 4 GiB를 할당하려 합니다. 예제는 상한(1 MiB)을 넘으면 `InvalidData` 에러를 돌려줍니다.
- `impl Write`/`impl Read`로 받으므로 `&TcpStream`도, `Vec<u8>`(프레임을 메모리에 만들어 보기)도, `&[u8]`(잘못된 입력 시험하기)도 넘길 수 있습니다.

Part 0 ch20의 HTTP는 두 방법을 섞어 씁니다. 요청 줄과 헤더는 줄 단위로 읽고, 본문의 길이는 `Content-Length` 헤더로 알립니다.

### 2.9 UDP: 데이터그램

UDP는 연결을 맺지 않습니다. bind한 소켓 하나로 아무 주소에나 **데이터그램(datagram)**, 즉 한 덩어리의 메시지를 보내고, 아무 주소에서 온 데이터그램이든 받습니다. [`06_udp_datagrams.rs`](examples/06_udp_datagrams.rs)가 성질을 하나씩 확인합니다.

```rust
let socket = UdpSocket::bind("127.0.0.1:0")?;
socket.send_to(b"hello", peer_addr)?;            // 받을 주소를 매번 적음
let (n, from) = socket.recv_from(&mut buf)?;     // 받은 바이트 수와 보낸 주소
```

- **경계가 보존됩니다.** `send_to` 한 번이 `recv_from` 한 번에 통째로 옵니다. 05 예제에서 TCP가 붙여 버린 `"안녕"`, `"하세요"`, `"!"`가 UDP에서는 6, 9, 1바이트로 따로 옵니다.
- **버퍼보다 큰 데이터그램은 잘립니다.** 10바이트를 4바이트 버퍼로 받으면 앞의 4바이트만 받고 나머지는 버려지며, 다음 `recv_from`은 다음 데이터그램을 돌려줍니다. 잘렸다는 알림도 없으므로 버퍼는 받을 수 있는 가장 큰 데이터그램보다 크게 잡습니다.
- **연결이 없으니 상대가 있는지 모릅니다.** 아무도 bind하지 않은 주소로 `send_to`해도 `Ok(13)`이 돌아오고, 데이터그램은 조용히 사라집니다.
- **전달이 보장되지 않습니다.** 문서가 인용하는 UDP 명세(RFC 768)는 UDP를 순서도 신뢰성도 보장하지 않는 프로토콜이라고 설명합니다. 루프백에서는 거의 사라지지 않지만, 실제 네트워크에서는 데이터그램이 사라지거나 보낸 순서와 다르게 도착할 수 있습니다.
- **크기에 한계가 있습니다.** 문서는 운영체제가 65507바이트보다 큰 버퍼를 거부할 수 있다고 적고 있는데, 이 macOS 환경의 기본 한계는 그보다 훨씬 작은 9216바이트였습니다(`sysctl net.inet.udp.maxdgram`). 9217바이트를 `send_to`하자 `Message too long (os error 40)`으로 실패했습니다.

**UDP의 `connect`.** `UdpSocket::connect`는 패킷을 하나도 주고받지 않고, 운영체제에 "이 소켓은 이 주소하고만 주고받는다"고 등록할 뿐입니다. 등록한 뒤에는 주소를 적지 않는 `send`/`recv`를 쓸 수 있고, 다른 주소에서 온 데이터그램은 걸러집니다(5번 구역에서 먼저 보낸 "모르는 사람"의 데이터그램이 오지 않습니다). `connect`하지 않은 소켓의 `peer_addr`는 `NotConnected`이고, 상대를 모르는 `send`도 에러입니다. 이 환경에서는 `connect`한 소켓으로 아무도 없는 포트에 보내자 다음 `recv`가 `ConnectionRefused`를 돌려줬습니다. 운영체제가 받은 "도착할 수 없음" 알림을 넘겨준 것인데, 상대 쪽이 알림을 보내지 않거나 알림이 사라지면 오지 않으므로 이것에 기대서는 안 됩니다.

`UdpSocket`의 메서드는 모두 `&self`를 받습니다. `mut` 없이 쓸 수 있고, 한 소켓을 여러 스레드가 빌려 함께 써도 됩니다.

### 2.10 TCP와 UDP 고르기

UDP에서 신뢰성이 필요하면 프로그램이 직접 챙겨야 합니다. [`07_udp_echo.rs`](examples/07_udp_echo.rs)의 클라이언트는 읽기 제한 시간 안에 답이 없으면 같은 메시지를 다시 보내고(재전송), 서버는 손실을 흉내 내려고 두 번째 데이터그램을 일부러 버립니다. 그래도 남는 문제가 있습니다.

- 요청이 아니라 **답이 사라진** 경우, 재전송된 요청을 서버가 두 번 처리합니다.
- 답이 사라진 것이 아니라 **늦게 온** 경우, 늦은 답이 다음 요청의 답으로 잘못 읽힙니다. 요청마다 번호를 붙여 맞춰 봐야 합니다(연습 문제 3).

TCP는 번호 매기기, 확인 응답, 재전송, 순서 맞추기, 상대가 감당할 만큼만 보내는 흐름 제어를 운영체제 안에서 모두 해 줍니다. 그 대가로 연결을 맺는 왕복이 필요하고, 패킷 하나가 사라지면 뒤의 데이터가 모두 그것을 기다립니다.

| | TCP | UDP |
| --- | --- | --- |
| 연결 | 맺고 닫음 (`connect`/`accept`, `shutdown`) | 없음 (`connect`는 상대 등록일 뿐) |
| 데이터 단위 | 경계 없는 바이트 스트림 → 프레이밍 필요 | 데이터그램, 경계 보존 |
| 순서·전달 | 보장 (안 되면 에러로 알림) | 보장 없음 |
| 서버 구조 | 연결마다 `TcpStream` 하나 (02) | 소켓 하나로 모든 상대 (07) |
| 표준 타입 | `TcpListener`, `TcpStream` | `UdpSocket` |
| 어울리는 곳 | 웹, 파일 전송, 데이터베이스 등 빠짐없이 순서대로 받아야 하는 것 | 짧은 질의·응답, 늦게 온 데이터는 쓸모없는 실시간 데이터(게임 상태, 음성), 브로드캐스트 |

특별한 이유가 없으면 TCP를 고릅니다. UDP는 "조금 잃어도 빠른 것이 낫다"거나 "메시지가 아주 짧아 연결을 맺는 비용이 아깝다"는 이유가 분명할 때 고르고, 필요한 신뢰성은 직접 설계합니다.

### 2.11 이 챕터에서 다루지 않는 것

| 주제 | 한 줄 소개 | 다루는 곳 |
| --- | --- | --- |
| 이벤트 루프, 비동기 네트워킹 | 논블로킹 소켓 여럿을 준비된 순서로 처리. 표준 라이브러리에는 `set_nonblocking`과 `WouldBlock`까지만 있음 | `ch13-async-primitives`(퓨처와 실행기), [Part 0 ch16](../../part0-getting-started/ch16-async/README.md)(비동기 런타임) |
| TLS(암호화 연결), HTTP 클라이언트 | 표준 라이브러리에 없어서 crates.io의 크레이트를 씀 | 이 저장소에서는 다루지 않음 |
| 유닉스 도메인 소켓 | 같은 컴퓨터의 프로세스끼리 파일 경로로 통신. `std::os::unix::net`에 `UnixStream`, `UnixListener`, `UnixDatagram`이 있음 | 문서 참고 |
| 소켓 옵션 | `set_nodelay`(작은 쓰기를 모으지 않음), `set_ttl`, UDP 브로드캐스트·멀티캐스트 | 문서 참고. `set_linger`, `set_keepalive`, `std::net::hostname`은 1.100.0에서 nightly 전용 |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 학습 목표 |
| --- | --- | --- |
| [`examples/01_addresses.rs`](examples/01_addresses.rs) | 주소 값 만들기, 파싱 규칙, 판별 메서드, `ToSocketAddrs`가 받는 모양 | 1 |
| [`examples/02_tcp_echo.rs`](examples/02_tcp_echo.rs) | 포트 0, `accept`/`connect`, 주소의 짝, `shutdown(Write)`과 EOF로 끝나는 에코 | 2 |
| [`examples/03_split_read_write.rs`](examples/03_split_read_write.rs) | 큰 데이터의 교착 상태, `try_clone`, `&TcpStream`과 스코프 스레드, `io::copy` 에코 | 3 |
| [`examples/04_timeouts_errors.rs`](examples/04_timeouts_errors.rs) | `ConnectionRefused`, `AddrInUse`, 제한 시간 0, 읽기 제한 시간, 논블로킹 | 4 |
| [`examples/05_message_framing.rs`](examples/05_message_framing.rs) | 경계 없는 바이트 스트림 관찰, 줄 단위·길이 접두사 에코 서버 | 5 |
| [`examples/06_udp_datagrams.rs`](examples/06_udp_datagrams.rs) | `send_to`/`recv_from`, 경계 보존, 잘림, 받을 사람 없음, UDP의 `connect` | 5 |
| [`examples/07_udp_echo.rs`](examples/07_udp_echo.rs) | 소켓 하나로 여러 클라이언트를 상대하는 서버, 제한 시간과 재전송 | 5 |
| [`compile_fail/`](compile_fail) | 소켓 API에서 자주 만나는 컴파일 에러 5개 (4절) | 1~3 |

모든 명령은 저장소 루트에서 실행합니다. 모든 예제는 종료 코드 0으로 끝나고 패닉하지 않습니다. 운영체제가 고른 포트 번호는 실행마다 다르므로 출력하지 않고, "0이 아닌가", "같은가" 같은 확인 결과만 출력합니다. 서버 스레드는 출력하지 않고 결과를 돌려주며, `main`이 모아서 출력하므로 줄 순서는 항상 같습니다.

### 01_addresses — 주소 타입

```sh
cargo run -p p1-ch12-net --example 01_addresses
```

```text
1. 주소 값 만들기
    Ipv4Addr::new(127, 0, 0, 1) → 127.0.0.1
    Ipv4Addr::LOCALHOST         → 127.0.0.1
    Ipv6Addr::LOCALHOST         → ::1
    IpAddr::from([192, 168, 0, 10]) → 192.168.0.10
    SocketAddr::new(..)         → 127.0.0.1:8080
    SocketAddr::from((v6, 8080)) → [::1]:8080  (IPv6는 대괄호로 감싸 표시)
    네 가지 방법으로 만든 127.0.0.1:8080이 모두 같은가: true
    c.ip() = ::1, c.port() = 8080, c.is_ipv6() = true
2. 문자열에서 파싱하기
    SocketAddr 127.0.0.1:8080    → Ok(127.0.0.1:8080)
    SocketAddr [::1]:8080        → Ok([::1]:8080)
    SocketAddr 127.0.0.1         → Err(invalid socket address syntax)
    SocketAddr localhost:8080    → Err(invalid socket address syntax)
    SocketAddr ::1:8080          → Err(invalid socket address syntax)
    SocketAddr 127.0.0.1:70000   → Err(invalid socket address syntax)
    IpAddr     0:0:0:0:0:0:0:1   → Ok(::1)
    IpAddr     127.0.0.1:8080    → Err(invalid IP address syntax)
3. 판별 메서드
    주소            루프백  사설    미지정  멀티캐스트
    127.0.0.1       true    false   false   false
    192.168.0.10    false   true    false   false
    172.31.255.1    false   true    false   false
    172.32.0.1      false   false   false   false
    203.0.113.7     false   false   false   false
    0.0.0.0         false   false   true    false
    224.0.0.1       false   false   false   true
    ::1             true    false   false   false
    fd00::1         false   true    false   false
    ::              false   false   true    false
4. ToSocketAddrs로 바꾼 결과
    a (SocketAddr)               → Ok([127.0.0.1:8080])
    "127.0.0.1:8080"             → Ok([127.0.0.1:8080])
    ("127.0.0.1", 8080)          → Ok([127.0.0.1:8080])
    (Ipv4Addr::LOCALHOST, 8080)  → Ok([127.0.0.1:8080])
    &pair[..]                    → Ok([127.0.0.1:8080, [::1]:8080])
    "127.0.0.1"                  → Err(InvalidInput: invalid socket address)
    "localhost:8080"             → Ok([[::1]:8080, 127.0.0.1:8080])
```

- 소켓을 하나도 열지 않습니다. 마지막 줄만 운영체제에 이름 해석을 부탁하며, 그 결과는 이 macOS 환경의 값입니다. 다른 환경에서는 `127.0.0.1:8080` 하나만 나오거나 순서가 다를 수 있습니다.
- 3번 구역의 "사설" 열은 IPv4는 `is_private`, IPv6는 `is_unique_local`의 결과입니다. `172.31.255.1`은 사설 범위 `172.16.0.0/12` 안이고, `172.32.0.1`은 그 바로 바깥입니다.
- 4번 구역의 `"127.0.0.1"`은 `parse`(2번 구역)와 달리 `io::Error`(`InvalidInput`)로 실패합니다. 이름 해석까지 포함한 변환은 입출력 작업으로 취급되기 때문입니다.

### 02_tcp_echo — TCP 에코 서버와 클라이언트

```sh
cargo run -p p1-ch12-net --example 02_tcp_echo
```

```text
1. 서버 준비
    서버 IP: 127.0.0.1
    운영체제가 고른 포트가 0이 아닌가: true
2. 클라이언트 A: SocketAddr 값으로 connect
    보냄: "안녕, 에코 서버!" (22바이트)
    받음: "안녕, 에코 서버!"
    상대 주소가 서버 주소와 같은가: true
    클라이언트 포트가 서버 포트와 다른가: true
3. 클라이언트 B: (Ipv4Addr, u16) 튜플로 connect
    보냄: "두 번째 연결" (17바이트)
    받음: "두 번째 연결"
    A와 B의 클라이언트 포트가 다른가: true
4. 서버 쪽 기록
    22바이트를 돌려줌, accept가 준 상대 주소 == 클라이언트의 local_addr: true
    17바이트를 돌려줌, accept가 준 상대 주소 == 클라이언트의 local_addr: true
```

- 서버 스레드는 `accept`를 두 번 불러 연결 두 개를 차례로 처리하고 끝납니다. 클라이언트 함수 `run_client`는 `connect`와 같은 바운드(`A: ToSocketAddrs`)를 받아, A는 `SocketAddr` 값으로, B는 `(Ipv4Addr::LOCALHOST, 포트)` 튜플로 연결합니다.
- 클라이언트마다 운영체제가 서로 다른 임시 포트를 골랐고, 서버의 `accept`가 알려 준 상대 주소가 바로 그 주소입니다(2.4절의 표).
- `run_client`에서 `shutdown(Shutdown::Write)` 줄을 지우면 이 예제는 끝나지 않습니다. 같은 상황을 제한 시간으로 끊어 본 것이 04 예제의 4번 구역입니다.

### 03_split_read_write — 읽기와 쓰기 나누기

```sh
cargo run -p p1-ch12-net --example 03_split_read_write
```

```text
1. try_clone: 같은 소켓을 가리키는 두 핸들
    local_addr가 같은가: true
    원본에 설정한 읽기 제한 시간 → 복제본의 read_timeout(): Some(500ms)
    복제본으로 shutdown(Write) → 원본의 read_to_end: 0바이트 후 EOF
2. 한 스레드에서 다 쓰고 나서 읽기 (64 MiB)
    쓰기 결과: Err(WouldBlock) — 다 쓰기 전에 멈췄습니다
3. try_clone: 쓰기는 다른 스레드, 읽기는 main
    보낸 바이트: 67108864, 돌려받은 바이트: 67108864, 내용이 같은가: true
4. &TcpStream + 스코프 스레드: stream 하나를 빌려서 나눠 쓰기
    보낸 바이트: 67108864, 돌려받은 바이트: 67108864, 내용이 같은가: true
5. 서버 쪽 기록 (연결별 에코 결과)
    연결 1: 0바이트를 돌려줌
    연결 2: 에러로 끝남 (클라이언트가 읽지 않고 닫음)
    연결 3: 67108864바이트를 돌려줌
    연결 4: 67108864바이트를 돌려줌
```

- 64 MiB를 주고받으므로 메모리를 수백 MiB 쓰고, 2번 구역의 쓰기 제한 시간 때문에 이 환경에서 1초 안팎이 걸렸습니다. 2번 구역의 `WouldBlock`은 macOS 기준이며 윈도우에서는 `TimedOut`일 수 있습니다.
- 2번 구역의 클라이언트가 받은 데이터를 읽지 않은 채 연결을 버리자 서버의 쓰기도 에러로 끝났습니다(5번 구역의 연결 2). 이때의 `ErrorKind`는 운영체제와 시점에 따라 `BrokenPipe`나 `ConnectionReset` 등으로 달라질 수 있어 출력하지 않습니다.
- 서버의 에코는 `io::copy(&mut &stream, &mut &stream)` 한 줄입니다(2.6절).

### 04_timeouts_errors — 제한 시간과 에러 종류

```sh
cargo run -p p1-ch12-net --example 04_timeouts_errors
```

```text
1. 아무도 기다리지 않는 포트에 연결
    connect         → Err(ConnectionRefused)
    connect_timeout → Err(ConnectionRefused), 제한 시간보다 빨리 돌아왔는가: true
2. 이미 쓰이는 주소에 bind
    같은 주소에 TcpListener::bind → Err(AddrInUse)
    같은 주소에 UdpSocket::bind   → Ok
3. 제한 시간에 0을 주면
    connect_timeout(.., 0)       → Err(InvalidInput)
    set_read_timeout(Some(0))    → Err(InvalidInput)
    set_read_timeout(None)       → Ok
4. 읽기 제한 시간 200ms: shutdown(Write)를 잊으면
    read_to_end → Err(WouldBlock), 제한 시간만큼 기다렸는가: true
    제한 시간 초과로 볼 것인가: true
    그사이 받아 둔 것: "안녕"
    shutdown(Write) 뒤 다시 read_to_end → Ok(0바이트 더), EOF에 도달
    서버가 돌려준 바이트: 6
5. 논블로킹 모드
    연결이 없을 때 accept → Err(WouldBlock)
    받은 데이터가 없을 때 read → Err(WouldBlock), 곧바로 돌아왔는가: true
```

- 1번 구역의 "닫힌 포트"는 포트 0으로 bind했다가 곧바로 닫아서 얻습니다. 그 짧은 사이에 다른 프로그램이 같은 포트를 차지하면 결과가 달라질 수 있지만, 가능성은 아주 낮습니다.
- 2번 구역: 같은 포트 번호라도 TCP와 UDP는 따로 셉니다.
- 4번 구역: 에코는 받았지만 EOF가 오지 않아 `read_to_end`가 제한 시간(200ms)에 걸렸습니다. 이 환경에서 기다린 시간은 매번 200ms를 조금 넘었습니다. 그사이 받은 `"안녕"`은 버퍼에 남아 있고, `shutdown(Write)` 뒤에는 서버가 연결을 닫아 정상적으로 EOF를 받습니다.
- 5번 구역의 클라이언트는 `accept`하지 않는 리스너에 연결했는데도 `connect`가 성공했습니다(2.4절). 블로킹 모드였다면 두 줄 모두 영원히 기다렸을 것입니다.

### 05_message_framing — 바이트 스트림과 프레이밍

```sh
cargo run -p p1-ch12-net --example 05_message_framing
```

```text
1. 여러 번 쓴 것이 한 번의 read로
    write_all 3번: ["안녕", "하세요", "!"]
    read 1번 → 16바이트 "안녕하세요!"
2. 한 번 쓴 것이 여러 번의 read로 (5바이트 버퍼)
    read 4번, 바이트 수: [5, 5, 5, 1]
    조각마다 따로 UTF-8로 풀면: ["안�", "�하�", "��요", "!"]
3. 줄 단위로 나누기 (BufReader + lines)
    보낸 조각: ["첫째 줄\n둘", "째 줄\n셋째 ", "줄\n"]
    서버가 읽은 줄: ["첫째 줄", "둘째 줄", "셋째 줄"]
    클라이언트가 받은 줄: ["에코: 첫째 줄", "에코: 둘째 줄", "에코: 셋째 줄"]
4. 길이 접두사로 나누기 (u32 빅 엔디언 4바이트 + 내용)
    "안녕"의 프레임: [0, 0, 0, 6, 236, 149, 136, 235, 133, 149]
    서버가 읽은 메시지 3개: ["안녕", "줄바꿈이\n들어 있어도 괜찮습니다", ""]
    클라이언트가 받은 메시지 3개: ["안녕", "줄바꿈이\n들어 있어도 괜찮습니다", ""]
    길이가 상한을 넘는 헤더 → Err(InvalidData: 메시지 길이 4294967295바이트가 상한 1048576바이트를 넘습니다)
    내용이 덜 온 채 끝남   → Err(UnexpectedEof: failed to fill whole buffer)
```

- 1·2번 구역은 관찰을 위해, 클라이언트가 다 쓴 뒤 채널로 신호를 보내면 서버가 0.1초 더 기다렸다가 읽습니다. 데이터가 모두 도착한 상태에서 읽으므로 이 환경에서는 매번 위와 같았습니다. 기다리지 않고 바로 읽으면 몇 바이트씩 읽힐지는 그때그때 다릅니다. 바로 그 "알 수 없음"이 프레이밍이 필요한 이유입니다.
- 2번 구역의 `�`는 `String::from_utf8_lossy`가 잘린 글자 자리에 넣은 대체 문자입니다.
- 3·4번 구역은 각각 완성된 에코 서버입니다. 줄 단위 서버는 `BufReader::new(&stream)`으로 읽으면서 같은 `stream`에 쓰고, 길이 접두사 서버는 `read_frame(&stream)`/`write_frame(&stream, ..)`에 `&TcpStream`을 넘깁니다.
- 4번 구역의 마지막 두 줄은 네트워크 없이 `&[u8]`을 `read_frame`에 넘겨 시험한 것입니다.

### 06_udp_datagrams — UDP의 성질

```sh
cargo run -p p1-ch12-net --example 06_udp_datagrams
```

```text
1. send_to와 recv_from
    a → b: send_to가 보낸 바이트: 6
    b가 받은 것: "안녕", 보낸 주소 == a의 주소: true
2. send_to 세 번 → recv_from 세 번
    6바이트 "안녕"
    9바이트 "하세요"
    1바이트 "!"
3. 10바이트 데이터그램을 4바이트 버퍼로 받기
    첫 recv_from → 4바이트 "0123"
    다음 recv_from → 6바이트 "다음"
4. 아무도 bind하지 않은 주소로 보내기
    send_to → Ok(13)
    connect 후 send → Ok(2)
    이어서 recv     → Err(ConnectionRefused)
5. connect로 상대를 고정한 소켓
    c.peer_addr() == a의 주소: true
    connect하지 않은 a.peer_addr() → Err(NotConnected)
    c.recv → "아는 사람"
    한 번 더 c.recv → Err(WouldBlock)
    a가 받은 답장: "답장", 보낸 주소 == c의 주소: true
```

- 받는 소켓에는 모두 1초짜리 읽기 제한 시간을 걸어, 데이터그램이 오지 않아도 예제가 멈추지 않게 했습니다. 5번 구역의 `Err(WouldBlock)`은 걸러진 데이터그램을 기다리다 100ms 제한 시간에 걸린 것입니다(macOS 기준).
- 4번 구역의 `ConnectionRefused`는 이 macOS 환경에서 매번 나왔지만 보장되는 동작은 아닙니다(2.9절).

### 07_udp_echo — UDP 에코 서버와 클라이언트

```sh
cargo run -p p1-ch12-net --example 07_udp_echo
```

```text
1. 클라이언트 A
    "안녕" → 답 "안녕" (보낸 횟수: 1)
    "두 번째 메시지" → 답 "두 번째 메시지" (보낸 횟수: 2)
2. 클라이언트 B
    "나는 B" → 답 "나는 B" (보낸 횟수: 1)
3. 서버 쪽 기록 (소켓 하나로 두 클라이언트를 상대함)
    A에게서 "안녕" → 돌려줌
    A에게서 "두 번째 메시지" → 일부러 버림
    A에게서 "두 번째 메시지" → 돌려줌
    B에게서 "나는 B" → 돌려줌
```

- 서버는 `recv_from`이 알려 준 주소로 `send_to`해 돌려주므로, 소켓 하나로 A와 B를 모두 상대합니다. 02의 TCP 서버가 연결마다 `TcpStream`을 따로 받은 것과 비교해 보세요.
- A의 두 번째 메시지는 서버가 버려서 답이 오지 않았고, 클라이언트는 200ms 제한 시간 뒤 다시 보내 답을 받았습니다. 클라이언트들이 차례로 보내므로 "두 번째로 받은 데이터그램"은 항상 A의 두 번째 메시지입니다.
- 서버도 2초 읽기 제한 시간을 걸어, 데이터그램이 오지 않으면 영원히 기다리지 않고 에러로 끝나게 했습니다.

## 4. 자주 하는 실수

`compile_fail/`의 파일은 일부러 컴파일되지 않게 만든 코드입니다. 챕터 디렉터리에서 `rustc`로 직접 컴파일해 에러를 확인합니다. 고쳐서 컴파일에 성공했을 때 실행 파일이 챕터 디렉터리에 생기지 않도록 `--out-dir`로 임시 디렉터리를 지정합니다.

```sh
cd part1-standard-lib/ch12-net
rustc --edition 2024 compile_fail/e0277_port_as_str.rs --out-dir /tmp/rust-p1-ch12
```

각 파일 맨 위의 주석에 고치는 방법을 적어 두었고, 그대로 고친 코드가 이 환경에서 경고 없이 컴파일되는 것을 확인했습니다. 아래 인용에서 표준 라이브러리 소스 위치(`/rustc/0ef1bc38.../library/...`) 줄은 생략했습니다.

### 4.1 포트를 문자열로 넘김 — E0277

파일: [`compile_fail/e0277_port_as_str.rs`](compile_fail/e0277_port_as_str.rs)

```text
error[E0277]: the trait bound `(&str, &str): ToSocketAddrs` is not satisfied
  --> compile_fail/e0277_port_as_str.rs:16:30
   |
16 |     match TcpStream::connect(("127.0.0.1", port)) {
   |           ------------------ ^^^^^^^^^^^^^^^^^^^ the trait `ToSocketAddrs` is not implemented for `(&str, &str)`
   |           |
   |           required by a bound introduced by this call
   |
   = help: the following other types implement trait `ToSocketAddrs`:
             (&str, u16)
             (IpAddr, u16)
             (Ipv4Addr, u16)
             (Ipv6Addr, u16)
             (String, u16)
```

- **읽는 법**: `connect`의 바운드 `ToSocketAddrs`를 `(&str, &str)`이 만족하지 않는다는 뜻입니다. `help`가 튜플 모양의 구현 목록을 보여 주는데, 모두 두 번째 원소가 `u16`입니다.
- **원인**: 설정 파일이나 명령줄에서 읽은 포트는 문자열이기 쉽습니다. `"8080"`이 올바른 포트인지(숫자인지, 65535 이하인지) 검사하는 일을 `connect`가 대신해 주지 않습니다.
- **해결**: `let port: u16 = port.parse()?;`처럼 먼저 파싱해서 `("127.0.0.1", port)`로 넘깁니다. 또는 `format!("{host}:{port}")`로 만든 `"호스트:포트"` 문자열 하나를 넘깁니다.

### 4.2 `TcpStream`에 `read_line` — E0599

파일: [`compile_fail/e0599_read_line_on_tcpstream.rs`](compile_fail/e0599_read_line_on_tcpstream.rs)

```text
error[E0599]: no method named `read_line` found for struct `TcpStream` in the current scope
  --> compile_fail/e0599_read_line_on_tcpstream.rs:18:12
   |
18 |     stream.read_line(&mut line)?;
   |            ^^^^^^^^^ method not found in `TcpStream`
```

- **읽는 법**: `TcpStream`에 `read_line`이라는 메서드가 없다는 뜻입니다. 이 파일은 `use std::io::BufRead;`를 써 두었는데도 같은 에러입니다.
- **원인**: `read_line`과 `lines`는 `BufRead`의 메서드이고, `BufRead`는 안에 버퍼가 있어 줄 끝을 찾을 수 있는 타입만 구현합니다. `TcpStream`은 읽을 때마다 운영체제에 바로 부탁하는 타입이라 버퍼가 없습니다. 바이트 스트림에서 줄을 찾으려면 읽은 조각을 모아 둘 곳이 필요하다는 2.8절의 이야기가 타입으로 드러난 것입니다.
- **해결**: `let mut reader = BufReader::new(&stream);`으로 감싸서 `reader.read_line(&mut line)`을 부릅니다. `&stream`으로 감싸면 같은 `stream`에 계속 쓸 수 있습니다(05 예제의 3번 구역).

### 4.3 `mut` 없는 `stream`에 쓰기 — E0596

파일: [`compile_fail/e0596_stream_not_mut.rs`](compile_fail/e0596_stream_not_mut.rs)

```text
error[E0596]: cannot borrow `stream` as mutable, as it is not declared as mutable
  --> compile_fail/e0596_stream_not_mut.rs:15:5
   |
15 |     stream.write_all(b"hello\n")?;
   |     ^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
14 |     let mut stream = TcpStream::connect("127.0.0.1:7878")?;
   |         +++
```

- **읽는 법**: `write_all`이 `&mut self`를 받으므로 `stream`을 가변으로 빌려야 하는데, `mut` 없이 선언했다는 뜻입니다.
- **해결 1**: `help`대로 `let mut stream`으로 선언합니다.
- **해결 2**: `&TcpStream`에도 `Write`가 구현되어 있으므로 `(&stream).write_all(b"hello\n")?;`으로 씁니다. 이때 가변으로 빌리는 대상은 `stream`이 아니라 임시로 만든 참조 `&stream`입니다. 여러 곳에서 같은 `stream`을 빌려 써야 할 때 이 방법이 쓸모 있습니다(2.6절).

### 4.4 스레드로 옮긴 `stream`을 다시 씀 — E0382

파일: [`compile_fail/e0382_stream_moved_into_thread.rs`](compile_fail/e0382_stream_moved_into_thread.rs)

```text
error[E0382]: borrow of moved value: `stream`
  --> compile_fail/e0382_stream_moved_into_thread.rs:19:5
   |
16 |     let mut stream = TcpStream::connect("127.0.0.1:7878")?;
   |         ---------- move occurs because `stream` has type `TcpStream`, which does not implement the `Copy` trait
17 |     let writer = thread::spawn(move || stream.write_all(b"hello\n"));
   |                                ------- ------ variable moved due to use in closure
   |                                |
   |                                value moved into closure here
18 |     let mut reply = String::new();
19 |     stream.read_to_string(&mut reply)?;
   |     ^^^^^^ value borrowed here after move
```

- **읽는 법**: 17번 줄의 `move` 클로저가 `stream`을 가져갔는데, 19번 줄에서 다시 쓰려 했다는 뜻입니다.
- **원인**: 쓰기는 다른 스레드에, 읽기는 `main`에 맡기려는 의도는 옳습니다(2.6절). 하지만 연결 하나를 두 곳이 쓰려면 핸들이 두 개 있거나 둘 다 빌려야 합니다. `TcpStream`은 `Clone`을 구현하지 않습니다.
- **해결 1**: `let mut writer = stream.try_clone()?;`로 핸들을 하나 더 만들어 그것을 스레드로 옮깁니다(03 예제의 3번 구역).
- **해결 2**: `thread::scope` 안에서 스레드가 `&stream`을 빌려 `(&stream).write_all(..)`로 씁니다(03 예제의 4번 구역).

### 4.5 `connect_timeout`에 문자열을 넘김 — E0308

파일: [`compile_fail/e0308_connect_timeout_str.rs`](compile_fail/e0308_connect_timeout_str.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_connect_timeout_str.rs:15:46
   |
15 |     let _stream = TcpStream::connect_timeout("127.0.0.1:7878", Duration::from_millis(500))?;
   |                   -------------------------- ^^^^^^^^^^^^^^^^ expected `&SocketAddr`, found `&str`
   |                   |
   |                   arguments to this function are incorrect
   |
   = note: expected reference `&std::net::SocketAddr`
              found reference `&'static str`
```

- **읽는 법**: 첫 번째 인자로 `&SocketAddr`를 기대했는데 `&str`가 왔다는 뜻입니다.
- **원인**: `connect`가 문자열을 받으니 `connect_timeout`도 받으리라 생각하기 쉽습니다. 하지만 `connect_timeout`은 `ToSocketAddrs`가 아니라 `&SocketAddr` 하나만 받습니다(2.3절).
- **해결**: `let addr: SocketAddr = "127.0.0.1:7878".parse()?;`로 파싱한 뒤 `connect_timeout(&addr, ..)`로 넘깁니다. 이름을 써야 한다면 `to_socket_addrs()`로 얻은 주소마다 `connect_timeout`을 시도합니다.

### 4.6 그 밖의 실수

| 상황 | 결과 | 해결 |
| --- | --- | --- |
| `stream.write(&buf)?;`처럼 `write`의 반환값을 버림 | clippy가 기본으로 거부(`clippy::unused_io_amount`, deny): `written amount is not handled`. `write`는 일부만 쓸 수 있습니다. `read`도 같습니다. | 다 써야 하면 `write_all`, 정확히 채워 읽어야 하면 `read_exact`를 씁니다. `read`는 돌려준 바이트 수만큼만 씁니다. |
| `let mut buf = Vec::new();`나 `Vec::with_capacity(64)`를 `read`의 버퍼로 넘김 | 길이가 0인 슬라이스라 `read`가 늘 `Ok(0)`을 돌려주고, EOF로 착각합니다. 기본 설정의 clippy는 경고하지 않습니다(`clippy::read_zero_byte_vec`는 기본으로 꺼져 있음). | `[0u8; 1024]`나 `vec![0u8; 1024]`처럼 길이가 있는 버퍼를 씁니다. 끝까지 모을 때는 `read_to_end(&mut vec)`를 씁니다. |
| 요청을 보낸 뒤 `shutdown(Write)` 없이 "EOF까지" 응답을 읽음 | 양쪽이 서로를 기다리며 멈춤 | `shutdown(Shutdown::Write)`로 다 보냈음을 알리거나, 프레이밍으로 메시지의 끝을 알립니다(04의 4번 구역, 2.8절). |
| `"localhost:8080".parse::<SocketAddr>()` | `Err(invalid socket address syntax)` | `parse`는 이름을 해석하지 않습니다. `connect("localhost:8080")`처럼 `ToSocketAddrs`를 받는 곳에 넘기거나 `to_socket_addrs()`를 씁니다(01). |
| `socket.recv_from(&mut String::new())` | E0308 `` expected `&mut [u8]`, found `&mut String` `` | 바이트 버퍼(`[0u8; N]`)로 받고 `String::from_utf8_lossy(&buf[..n])` 등으로 바꿉니다. |
| UDP 받는 버퍼를 작게 잡음 | 데이터그램이 알림 없이 잘림(06의 3번 구역) | 받을 수 있는 가장 큰 데이터그램보다 크게 잡습니다. |
| `read`로 받은 조각마다 `String::from_utf8` | 글자가 조각 경계에서 잘려 실패하거나 깨짐(05의 2번 구역) | 메시지 단위로 모은 뒤 바꿉니다. 줄 단위면 `BufReader::read_line`이 알아서 모읍니다. |
| 제한 시간 초과를 `ErrorKind::TimedOut`만으로 검사 | 이 macOS 환경에서는 `WouldBlock`이 와서 검사를 빠져나감 | `WouldBlock`과 `TimedOut`을 함께 검사합니다(2.7절의 `is_timeout`). |
| 연습용 서버를 `0.0.0.0`에 bind | 같은 네트워크의 다른 컴퓨터도 접속할 수 있음 | 이 컴퓨터 안에서만 쓸 서버는 `127.0.0.1`에 bind합니다(2.2절). |

## 5. 연습 문제

### 연습 1. 여러 클라이언트를 동시에 받는 줄 단위 에코 서버

05 예제의 줄 단위 서버를 바꿔, 연결 3개를 **동시에** 처리하게 하세요. `main`은 클라이언트 3개를 각각 스레드로 띄우고, 클라이언트마다 줄 두 개(`"{이름}: 하나"`, `"{이름}: 둘"`)를 보낸 뒤 받은 줄을 돌려받습니다. 서버는 연결 3개를 처리하면 끝나고, 출력은 클라이언트 이름 순서로 항상 같아야 합니다.

<details>
<summary>힌트</summary>

- 서버 스레드 안에서 `thread::scope(|s| { for _ in 0..3 { let (stream, _) = listener.accept()?; s.spawn(move || handle(stream)); } ... })`처럼 연결마다 스코프 스레드를 띄웁니다. `handle`은 05 예제의 줄 단위 서버 클로저를 함수로 옮긴 것입니다. `accept`의 `?`를 쓰려면 스코프 클로저가 `io::Result`를 돌려주게 합니다.
- 클라이언트 스레드의 `JoinHandle`을 `Vec`에 모아 두었다가 차례로 `join`하면 출력 순서가 고정됩니다(스레드 안에서 출력하지 않습니다).
- 연결마다 스레드를 만드는 방식의 한계(상한 없음)는 Part 0 ch20의 2.5절을 떠올려 보세요. 여기서는 연결 수가 3개로 정해져 있어 괜찮습니다.

</details>

### 연습 2. 헤더 중간에서 끊긴 연결을 구분하기

05 예제의 `read_frame`은 헤더를 하나도 못 읽은 경우와 헤더 중간에서 끊긴 경우를 모두 `Ok(None)`으로 처리합니다. 앞의 경우만 `Ok(None)`이고 뒤의 경우는 `UnexpectedEof` 에러가 되게 고치세요. `&[u8]`을 넘겨 다음을 확인합니다.

```text
빈 입력          → Ok(None)
[0, 0]           → Err(UnexpectedEof)
[0, 0, 0, 1, 65] → Ok(Some([65]))
```

<details>
<summary>힌트</summary>

- 헤더의 첫 바이트를 `read`로 읽어 0이면 `Ok(None)`을 돌려주고, 나머지 3바이트는 `read_exact(&mut header[1..])`로 읽습니다. 이때 생기는 `UnexpectedEof`는 그대로 `?`로 돌려줍니다.
- `read`는 신호에 끊기면 `ErrorKind::Interrupted`를 돌려줄 수 있습니다. 엄밀하게 하려면 그때 다시 시도하는 반복문으로 감쌉니다(`read_exact`는 안에서 이미 그렇게 합니다).

</details>

### 연습 3. UDP 요청에 번호 붙이기

07 예제의 서버를 바꿔, **첫 번째로 받은 데이터그램의 답을 300ms 늦게** 보내게 하세요(버리지 않고 늦게 보냅니다). 그러면 클라이언트는 제한 시간(200ms) 뒤 재전송하고, 늦게 도착한 첫 답과 재전송의 답이 둘 다 옵니다. 지금의 클라이언트로 메시지 두 개를 보내면 두 번째 메시지의 답 자리에 무엇이 오는지 확인한 뒤, 요청 번호로 이 문제를 고치세요.

<details>
<summary>힌트</summary>

- 늦게 보내기는 서버 스레드에서 `thread::sleep(Duration::from_millis(300))` 뒤 `send_to`하면 됩니다. 서버가 처리할 데이터그램 수(`DATAGRAMS`)도 맞게 고칩니다.
- 클라이언트는 `format!("{id}:{message}")`로 보내고, 답을 `split_once(':')`로 나눠 번호가 `id`와 다르면 버리고 다시 `recv`합니다. 번호가 다른 답을 버리는 동안에도 제한 시간은 계속 적용됩니다.
- 이 문제는 TCP를 쓰면 생기지 않습니다. TCP가 안에서 무엇을 해 주는지 2.10절의 목록과 비교해 보세요.

</details>

### 연습 4. 열린 포트 확인하기

`127.0.0.1`에서 포트 0으로 리스너 2개를 띄우고, 04 예제의 `closed_port` 방법으로 닫힌 포트 2개를 얻으세요. 네 주소를 섞은 목록에 `connect_timeout`(제한 시간 100ms)을 차례로 시도해 `열림`, `닫힘(거부)`, `응답 없음(시간 초과)`, `그 밖의 에러` 중 하나로 분류해 출력합니다. 포트 번호는 출력하지 말고, 목록에서의 순번과 결과만 출력합니다. 반드시 `127.0.0.1`만 대상으로 합니다.

<details>
<summary>힌트</summary>

- `match TcpStream::connect_timeout(&addr, timeout)`에서 `Ok(_)`는 열림, `Err(e) if e.kind() == ErrorKind::ConnectionRefused`는 닫힘, `Err(e) if is_timeout(&e)`는 응답 없음으로 나눕니다.
- 루프백에서는 "응답 없음"이 나오지 않을 것입니다(2.7절). 그래도 분류는 남겨 두세요.
- 리스너는 `accept`하지 않아도 연결이 성공합니다(2.4절). 결과를 다 출력할 때까지 리스너를 버리지 않도록 변수에 담아 둡니다.

</details>

## 6. 참고 자료

- 표준 라이브러리 문서 (기준 문서)
  - [`std::net`](https://doc.rust-lang.org/std/net/index.html): 모듈에 있는 타입의 전체 목록. 자식 프로세스에 소켓이 상속되지 않게 한다는 설명도 있습니다.
  - [`TcpListener`](https://doc.rust-lang.org/std/net/struct.TcpListener.html): `bind`의 포트 0, `accept`가 돌려줄 수 있는 에러 중 서버가 무시하고 계속해도 되는 것, `incoming`이 끝나지 않는다는 설명.
  - [`TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html): `connect`가 주소를 차례로 시도하는 방식, `connect_timeout`, `shutdown`의 플랫폼 차이, 제한 시간 초과의 `ErrorKind`(Platform-specific behavior 절), `set_nonblocking`, `SIGPIPE` 설명. Trait Implementations 절에서 `Read for &TcpStream`, `Write for &TcpStream`을 확인할 수 있습니다.
  - [`UdpSocket`](https://doc.rust-lang.org/std/net/struct.UdpSocket.html): `recv_from`의 잘림, `send_to`의 크기 한계, `connect`가 하는 일과 한계, `peer_addr`의 `NotConnected`.
  - [`SocketAddr`](https://doc.rust-lang.org/std/net/enum.SocketAddr.html), [`IpAddr`](https://doc.rust-lang.org/std/net/enum.IpAddr.html), [`Ipv4Addr`](https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html): 생성자와 `From` 구현, 판별 메서드의 정확한 범위, nightly 전용으로 표시된 메서드(`is_global` 등).
  - [`ToSocketAddrs`](https://doc.rust-lang.org/std/net/trait.ToSocketAddrs.html): 구현 목록, 이 트레이트가 있는 이유, 이름 해석이 스레드를 멈추게 할 수 있다는 설명.
  - [`Shutdown`](https://doc.rust-lang.org/std/net/enum.Shutdown.html): 세 배리언트의 동작.
  - [`std::os::unix::net`](https://doc.rust-lang.org/std/os/unix/net/index.html): 유닉스 도메인 소켓(2.11절).
- Rust By Example과 Rustlings에는 이 챕터에 대응하는 절이나 연습이 없습니다. 소켓을 쓰는 프로그램 전체의 흐름은 [Part 0 ch20](../../part0-getting-started/ch20-web-server-project/README.md)의 웹 서버를 다시 읽어 보세요.
- 에러 설명: `rustc --explain E0277`처럼 에러 코드를 넣어 실행하면 터미널에서 설명과 예제를 볼 수 있습니다.
