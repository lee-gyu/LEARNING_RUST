// TCP는 바이트 스트림입니다: 메시지의 경계는 직접 나눠야 합니다(프레이밍).
//
//   1) 여러 번 쓴 것이 한 번의 read로 붙어서 올 수 있습니다.
//   2) 한 번 쓴 것이 여러 번의 read로 나뉘어 올 수 있습니다. UTF-8 글자의 중간에서도 잘립니다.
//   3) 줄 단위: 메시지 끝에 '\n'을 붙이고, 받는 쪽은 BufReader의 lines()로 줄을 모읍니다.
//   4) 길이 접두사: 메시지 앞에 바이트 수(u32 빅 엔디언 4바이트)를 붙이고, 받는 쪽은 read_exact로 읽습니다.
//
// 1·2번 구역은 관찰을 위해, 클라이언트가 다 쓴 뒤 데이터가 도착할 시간을 조금 준 다음 읽습니다.
//
// 실행: cargo run -p p1-ch12-net --example 05_message_framing

use std::io::{self, BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// 데이터가 받는 쪽 소켓 버퍼에 도착할 때까지 기다리는 시간(관찰용)입니다.
const SETTLE: Duration = Duration::from_millis(100);
/// 길이 접두사 방식에서 받아들일 메시지의 최대 길이입니다.
const MAX_FRAME: u32 = 1024 * 1024;

fn main() -> io::Result<()> {
    // ── 1. 여러 번 쓴 것이 한 번에 읽힘 ────────────────────────────
    println!("1. 여러 번 쓴 것이 한 번의 read로");
    let pieces = ["안녕", "하세요", "!"];
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let (addr, server) = serve_once(move |mut stream| {
        done_rx
            .recv()
            .expect("main은 신호를 보낸 뒤에 송신자를 버립니다");
        thread::sleep(SETTLE);
        let mut buf = [0u8; 64];
        let n = stream.read(&mut buf)?; // read 한 번
        Ok(buf[..n].to_vec())
    })?;
    let mut client = TcpStream::connect(addr)?;
    for piece in pieces {
        client.write_all(piece.as_bytes())?; // write_all 세 번
    }
    done_tx
        .send(())
        .expect("서버 스레드는 신호를 기다리고 있습니다");
    let got = join(server)?;
    println!("    write_all {}번: {pieces:?}", pieces.len());
    println!(
        "    read 1번 → {}바이트 {:?}",
        got.len(),
        String::from_utf8_lossy(&got)
    );
    // 출력: read 1번 → 16바이트 "안녕하세요!"

    // ── 2. 한 번 쓴 것이 여러 번에 나뉘어 읽힘 ─────────────────────
    println!("2. 한 번 쓴 것이 여러 번의 read로 (5바이트 버퍼)");
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let (addr, server) = serve_once(move |mut stream| {
        done_rx
            .recv()
            .expect("main은 신호를 보낸 뒤에 송신자를 버립니다");
        thread::sleep(SETTLE);
        let mut chunks = Vec::new();
        let mut buf = [0u8; 5];
        loop {
            let n = stream.read(&mut buf)?;
            if n == 0 {
                return Ok(chunks);
            }
            chunks.push(buf[..n].to_vec());
        }
    })?;
    let mut client = TcpStream::connect(addr)?;
    client.write_all("안녕하세요!".as_bytes())?; // 16바이트를 한 번에
    client.shutdown(Shutdown::Write)?;
    done_tx
        .send(())
        .expect("서버 스레드는 신호를 기다리고 있습니다");
    let chunks = join(server)?;
    let sizes: Vec<usize> = chunks.iter().map(Vec::len).collect();
    let decoded: Vec<_> = chunks.iter().map(|c| String::from_utf8_lossy(c)).collect();
    println!("    read {}번, 바이트 수: {sizes:?}", chunks.len()); // 출력: read 4번, 바이트 수: [5, 5, 5, 1]
    println!("    조각마다 따로 UTF-8로 풀면: {decoded:?}");
    // 출력: ["안�", "�하�", "��요", "!"]  ← 한글 한 글자(3바이트)가 조각 경계에서 잘렸습니다.

    // ── 3. 줄 단위로 나누기 ────────────────────────────────────────
    println!("3. 줄 단위로 나누기 (BufReader + lines)");
    let (addr, server) = serve_once(|stream| {
        // BufReader가 읽은 바이트를 모아 두었다가 '\n'까지를 한 줄로 내줍니다.
        let reader = BufReader::new(&stream);
        let mut writer = &stream; // &TcpStream도 Write입니다(03 예제).
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line = line?;
            writer.write_all(format!("에코: {line}\n").as_bytes())?;
            lines.push(line);
        }
        Ok(lines)
    })?;
    let mut client = TcpStream::connect(addr)?;
    // 줄의 중간에서 일부러 끊어 보냅니다. 조각이 따로 도착하도록 사이사이 잠깐 쉽니다.
    let cut = ["첫째 줄\n둘", "째 줄\n셋째 ", "줄\n"];
    for piece in cut {
        client.write_all(piece.as_bytes())?;
        thread::sleep(Duration::from_millis(20));
    }
    client.shutdown(Shutdown::Write)?;
    let replies: Vec<String> = BufReader::new(&client).lines().collect::<io::Result<_>>()?;
    let lines = join(server)?;
    println!("    보낸 조각: {cut:?}");
    println!("    서버가 읽은 줄: {lines:?}");
    println!("    클라이언트가 받은 줄: {replies:?}");
    // 출력: 서버가 읽은 줄: ["첫째 줄", "둘째 줄", "셋째 줄"]

    // ── 4. 길이 접두사로 나누기 ────────────────────────────────────
    println!("4. 길이 접두사로 나누기 (u32 빅 엔디언 4바이트 + 내용)");
    let mut frame = Vec::new(); // Vec<u8>도 Write라서 프레임을 메모리에 만들어 볼 수 있습니다.
    write_frame(&mut frame, "안녕".as_bytes())?;
    println!("    \"안녕\"의 프레임: {frame:?}"); // 출력: [0, 0, 0, 6, 236, 149, 136, 235, 133, 149]

    let (addr, server) = serve_once(|stream| {
        let mut messages = Vec::new();
        while let Some(payload) = read_frame(&stream)? {
            write_frame(&stream, &payload)?;
            messages.push(String::from_utf8_lossy(&payload).into_owned());
        }
        Ok(messages)
    })?;
    let client = TcpStream::connect(addr)?;
    // 내용에 '\n'이 있어도, 내용이 비어 있어도 경계가 정확히 지켜집니다.
    for message in ["안녕", "줄바꿈이\n들어 있어도 괜찮습니다", ""] {
        write_frame(&client, message.as_bytes())?;
    }
    client.shutdown(Shutdown::Write)?;
    let mut replies = Vec::new();
    while let Some(payload) = read_frame(&client)? {
        replies.push(String::from_utf8_lossy(&payload).into_owned());
    }
    let messages = join(server)?;
    println!("    서버가 읽은 메시지 {}개: {messages:?}", messages.len());
    println!(
        "    클라이언트가 받은 메시지 {}개: {replies:?}",
        replies.len()
    );
    // 출력: 서버가 읽은 메시지 3개: ["안녕", "줄바꿈이\n들어 있어도 괜찮습니다", ""]

    // &[u8]도 Read라서 네트워크 없이 잘못된 입력을 시험할 수 있습니다.
    let huge: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF];
    let cut_short: &[u8] = &[0, 0, 0, 5, b'a', b'b'];
    println!("    길이가 상한을 넘는 헤더 → {}", show(&read_frame(huge)));
    println!(
        "    내용이 덜 온 채 끝남   → {}",
        show(&read_frame(cut_short))
    );
    // 출력: Err(InvalidData: 메시지 길이 4294967295바이트가 상한 1048576바이트를 넘습니다)
    // 출력: Err(UnexpectedEof: failed to fill whole buffer)
    Ok(())
}

/// 메시지 하나를 [길이 4바이트][내용] 모양으로 씁니다.
fn write_frame(mut w: impl Write, payload: &[u8]) -> io::Result<()> {
    let len = u32::try_from(payload.len()).map_err(|_| {
        io::Error::new(
            ErrorKind::InvalidInput,
            "메시지가 u32로 나타낼 수 없을 만큼 깁니다",
        )
    })?;
    // 네트워크로 보내는 정수는 바이트 순서를 정해 둬야 합니다. 관례적으로 빅 엔디언을 씁니다.
    w.write_all(&len.to_be_bytes())?;
    w.write_all(payload)
}

/// 메시지 하나를 읽습니다. 메시지 사이에서 EOF를 만나면 Ok(None)입니다.
fn read_frame(mut r: impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; 4];
    // read_exact는 버퍼를 다 채울 때까지 read를 되풀이합니다. 조각나서 와도 4바이트를 모읍니다.
    match r.read_exact(&mut header) {
        Ok(()) => {}
        // 단순화: 헤더를 하나도 못 읽은 경우와 헤더 중간에서 끊긴 경우를 구분하지 않습니다.
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_be_bytes(header);
    // 상대가 보낸 길이를 그대로 믿고 할당하면, 잘못된 헤더 하나로 4 GiB를 할당할 수 있습니다.
    if len > MAX_FRAME {
        let msg = format!("메시지 길이 {len}바이트가 상한 {MAX_FRAME}바이트를 넘습니다");
        return Err(io::Error::new(ErrorKind::InvalidData, msg));
    }
    let mut payload = vec![0u8; len as usize];
    r.read_exact(&mut payload)?;
    Ok(Some(payload))
}

/// 포트 0으로 리스너를 만들고, 연결 하나를 받아 handler에 넘기는 서버 스레드를 띄웁니다.
fn serve_once<T, F>(handler: F) -> io::Result<(SocketAddr, JoinHandle<io::Result<T>>)>
where
    F: FnOnce(TcpStream) -> io::Result<T> + Send + 'static,
    T: Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let handle = thread::spawn(move || {
        let (stream, _peer) = listener.accept()?;
        handler(stream)
    });
    Ok((addr, handle))
}

fn join<T>(handle: JoinHandle<io::Result<T>>) -> io::Result<T> {
    handle.join().expect("서버 스레드는 패닉하지 않아야 합니다")
}

fn show<T>(result: &io::Result<T>) -> String {
    match result {
        Ok(_) => "Ok".to_string(),
        Err(e) => format!("Err({:?}: {e})", e.kind()),
    }
}
