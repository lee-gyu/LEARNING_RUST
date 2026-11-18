// 읽기와 쓰기를 나누기: try_clone과 &TcpStream
//
//   1) try_clone은 같은 소켓을 가리키는 핸들을 하나 더 만듭니다. 설정과 연결 상태를 함께 씁니다.
//   2) 한 스레드에서 "다 쓰고 나서 읽기"를 하면, 데이터가 클 때 양쪽이 서로를 기다리며 멈춥니다.
//      영원히 멈추지 않도록 쓰기 제한 시간(300ms)을 걸어 멈춘 것을 확인합니다.
//   3) try_clone한 핸들을 쓰기 스레드로 옮기고 main은 읽기만 하면 끝까지 주고받습니다.
//   4) &TcpStream도 Read와 Write를 구현하므로, 스코프 스레드에서 같은 stream을 빌려 나눠 써도 됩니다.
//
// 64 MiB를 주고받으므로 메모리를 수백 MiB 쓰고, 2번 구역의 제한 시간 때문에 1초 안팎이 걸립니다.
//
// 실행: cargo run -p p1-ch12-net --example 03_split_read_write

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

const SIZE: usize = 64 * 1024 * 1024;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    // 서버: 연결 4개(구역 1~4)를 차례로 받아 에코하고, 연결마다 결과를 남깁니다.
    let server = thread::spawn(move || -> io::Result<Vec<io::Result<u64>>> {
        let mut results = Vec::new();
        for _ in 0..4 {
            let (stream, _peer) = listener.accept()?;
            results.push(echo(stream));
        }
        Ok(results)
    });

    // ── 1. try_clone: 같은 소켓의 핸들 두 개 ──────────────────────
    println!("1. try_clone: 같은 소켓을 가리키는 두 핸들");
    let stream = TcpStream::connect(addr)?;
    let clone = stream.try_clone()?;
    let same = stream.local_addr()? == clone.local_addr()?;
    println!("    local_addr가 같은가: {same}"); // 출력: true
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;
    // 출력: 원본에 설정한 읽기 제한 시간 → 복제본의 read_timeout(): Some(500ms)
    println!(
        "    원본에 설정한 읽기 제한 시간 → 복제본의 read_timeout(): {:?}",
        clone.read_timeout()?
    );
    // shutdown은 핸들이 아니라 소켓에 작용합니다. 복제본으로 불러도 연결의 쓰기 방향이 닫힙니다.
    clone.shutdown(Shutdown::Write)?;
    let mut rest = Vec::new();
    let n = (&stream).read_to_end(&mut rest)?;
    println!("    복제본으로 shutdown(Write) → 원본의 read_to_end: {n}바이트 후 EOF"); // 출력: 0바이트 후 EOF
    drop(clone);
    drop(stream);

    // 64 MiB짜리 데이터. slice::repeat은 표준 라이브러리 안에서 복사하므로 디버그 빌드에서도 빠릅니다.
    let pattern: Vec<u8> = (0..=250).collect();
    let mut payload = pattern.repeat(SIZE / pattern.len() + 1);
    payload.truncate(SIZE);

    // ── 2. 한 스레드에서 다 쓰고 나서 읽기 → 멈춤 ──────────────────
    println!("2. 한 스레드에서 다 쓰고 나서 읽기 ({} MiB)", SIZE >> 20);
    let mut stream = TcpStream::connect(addr)?;
    // 제한 시간이 없으면 write_all이 영원히 돌아오지 않습니다(교착 상태).
    stream.set_write_timeout(Some(Duration::from_millis(300)))?;
    match stream.write_all(&payload) {
        Ok(()) => println!("    다 썼습니다(운영체제의 소켓 버퍼가 64 MiB보다 큰 환경)"),
        // macOS·리눅스는 WouldBlock, 윈도우는 TimedOut으로 알려 줍니다(TcpStream 문서).
        Err(e) => println!(
            "    쓰기 결과: Err({:?}) — 다 쓰기 전에 멈췄습니다",
            e.kind()
        ),
    }
    // 출력(macOS): 쓰기 결과: Err(WouldBlock) — 다 쓰기 전에 멈췄습니다
    drop(stream); // 받은 데이터를 읽지 않은 채 닫습니다. 서버의 쓰기도 에러로 끝납니다.

    // ── 3. try_clone으로 나누기: 쓰기 스레드 + 읽는 main ──────────
    println!("3. try_clone: 쓰기는 다른 스레드, 읽기는 main");
    let mut reader = TcpStream::connect(addr)?;
    let mut writer = reader.try_clone()?; // thread::spawn에 옮길 수 있는, 따로 소유한 핸들
    let to_send = payload.clone();
    let handle = thread::spawn(move || -> io::Result<()> {
        writer.write_all(&to_send)?;
        writer.shutdown(Shutdown::Write) // 다 보냈음을 알려 서버가 EOF를 받게 합니다.
    });
    let mut received = Vec::with_capacity(SIZE);
    reader.read_to_end(&mut received)?; // 쓰는 동안에도 계속 읽어 주므로 서버가 멈추지 않습니다.
    handle
        .join()
        .expect("쓰기 스레드는 패닉하지 않아야 합니다")?;
    report(&payload, &received);
    // 출력: 보낸 바이트: 67108864, 돌려받은 바이트: 67108864, 내용이 같은가: true

    // ── 4. &TcpStream + 스코프 스레드 ──────────────────────────────
    // 스코프 스레드(thread::scope)는 스코프가 끝나기 전에 반드시 join되므로 지역 변수를 빌릴 수 있습니다.
    println!("4. &TcpStream + 스코프 스레드: stream 하나를 빌려서 나눠 쓰기");
    let stream = TcpStream::connect(addr)?;
    let received = thread::scope(|s| -> io::Result<Vec<u8>> {
        let writer = s.spawn(|| -> io::Result<()> {
            // (&stream)은 &TcpStream입니다. Write가 &TcpStream에도 구현되어 있어 mut 없이 씁니다.
            (&stream).write_all(&payload)?;
            stream.shutdown(Shutdown::Write)
        });
        let mut received = Vec::with_capacity(SIZE);
        (&stream).read_to_end(&mut received)?;
        writer
            .join()
            .expect("쓰기 스레드는 패닉하지 않아야 합니다")?;
        Ok(received)
    })?;
    report(&payload, &received);

    // ── 5. 서버 쪽 기록 ────────────────────────────────────────────
    let results = server
        .join()
        .expect("서버 스레드는 패닉하지 않아야 합니다")?;
    println!("5. 서버 쪽 기록 (연결별 에코 결과)");
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(n) => println!("    연결 {}: {n}바이트를 돌려줌", i + 1),
            // 상대가 읽지 않고 닫았을 때의 ErrorKind(BrokenPipe, ConnectionReset 등)는
            // 운영체제와 시점에 따라 달라 출력하지 않습니다.
            Err(_) => println!(
                "    연결 {}: 에러로 끝남 (클라이언트가 읽지 않고 닫음)",
                i + 1
            ),
        }
    }
    // 출력: 연결 1: 0바이트 / 연결 2: 에러로 끝남 / 연결 3·4: 67108864바이트를 돌려줌
    Ok(())
}

/// 에코 서버의 본체입니다. io::copy는 EOF까지 읽은 것을 모두 써 주고 그 바이트 수를 돌려줍니다.
/// &TcpStream이 Read와 Write를 모두 구현하므로, 같은 stream을 읽는 쪽과 쓰는 쪽으로 동시에 빌립니다.
fn echo(stream: TcpStream) -> io::Result<u64> {
    io::copy(&mut &stream, &mut &stream)
}

fn report(sent: &[u8], received: &[u8]) {
    println!(
        "    보낸 바이트: {}, 돌려받은 바이트: {}, 내용이 같은가: {}",
        sent.len(),
        received.len(),
        sent == received
    );
}
