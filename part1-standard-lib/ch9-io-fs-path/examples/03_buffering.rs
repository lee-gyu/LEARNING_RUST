// 버퍼링이 왜 필요한지를 "안쪽 read/write가 불린 횟수"를 세어 눈으로 확인합니다.
//
// 파일이나 소켓의 read/write는 한 번 부를 때마다 운영체제에 시스템 호출을 합니다.
// 시스템 호출은 함수 호출보다 훨씬 비싸므로, 작은 조각을 여러 번 보내는 대신
// 메모리에 모아 두었다가 큰 덩어리로 한 번에 보내는 것이 버퍼링입니다.
// 이 예제는 파일 대신 호출 횟수를 세는 래퍼 타입을 써서, 그 횟수가 어떻게 줄어드는지 봅니다.
//
//   1) BufWriter: 작은 쓰기를 모아 큰 쓰기로 바꿉니다.
//   2) BufReader: 큰 읽기 한 번으로 받아 두고 조금씩 내줍니다.
//   3) BufWriter를 flush 없이 버리면 쓰기 에러를 알 수 없습니다.
//   4) LineWriter: 줄바꿈을 만날 때마다 비웁니다(표준 출력의 줄 버퍼링).
//   5) 표준 출력도 lock()으로 한 번 잠그고 BufWriter로 감싸면 안쪽 호출이 줄어듭니다.
//
// 실행: cargo run -p p1-ch9-io-fs-path --example 03_buffering
//
// 호출 횟수는 BufReader/BufWriter의 기본 버퍼 크기(문서: "현재 8 KiB, 바뀔 수 있음")와
// 이 툴체인의 구현에 따른 값입니다.

use std::io::{self, BufRead, BufReader, BufWriter, LineWriter, Read, Write};

// ── 호출 횟수를 세는 래퍼 ──────────────────────────────────────────────
// 안쪽 라이터/리더에 그대로 넘기면서 몇 번 불렸는지만 셉니다.
// 진짜 파일이라면 이 횟수가 곧 시스템 호출 횟수입니다.
struct CountingWriter<W> {
    inner: W,
    writes: usize,
}

impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct CountingReader<R> {
    inner: R,
    reads: usize,
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reads += 1;
        self.inner.read(buf)
    }
}

fn main() -> io::Result<()> {
    println!("1. 2000줄을 쓸 때 안쪽 write가 불린 횟수");
    // 버퍼 없이: writeln!은 서식의 조각("줄 ", 숫자, "\n")마다 write_all을 부르므로 한 줄에 여러 번입니다.
    let mut direct = CountingWriter {
        inner: Vec::new(),
        writes: 0,
    };
    for i in 0..2000 {
        writeln!(direct, "줄 {i}")?;
    }
    println!(
        "    버퍼 없이  : {}바이트, write {}번",
        direct.inner.len(),
        direct.writes
    );
    // BufWriter로 감싸면: 8 KiB 버퍼가 찰 때와 flush할 때만 안쪽 write를 부릅니다.
    let mut buffered = BufWriter::new(CountingWriter {
        inner: Vec::new(),
        writes: 0,
    });
    for i in 0..2000 {
        writeln!(buffered, "줄 {i}")?;
    }
    buffered.flush()?; // 남은 내용을 안쪽으로 내보냅니다.
    let inner = buffered.get_ref(); // 감싼 값을 빌려 봅니다.
    println!(
        "    BufWriter  : {}바이트, write {}번",
        inner.inner.len(),
        inner.writes
    );
    // 출력:
    //     버퍼 없이  : 16890바이트, write 6000번
    //     BufWriter  : 16890바이트, write 3번

    println!("2. 10000바이트를 한 바이트씩 읽을 때 안쪽 read가 불린 횟수");
    let data = vec![b'x'; 10_000];
    // bytes()는 1바이트짜리 read를 반복합니다. 버퍼가 없으면 바이트마다 안쪽 read가 불립니다.
    let mut direct = CountingReader {
        inner: &data[..],
        reads: 0,
    };
    // clippy는 버퍼 없는 리더에 bytes()를 부르는 코드를 unbuffered_bytes로 경고합니다.
    // 여기서는 그 비용을 보여 주는 것이 목적이므로 경고를 예상된 것으로 표시합니다.
    #[expect(
        clippy::unbuffered_bytes,
        reason = "버퍼 없는 읽기의 호출 횟수를 보여 주기 위함"
    )]
    let count = direct.by_ref().bytes().count(); // by_ref: 소비하지 않고 빌려서 반복합니다.
    println!(
        "    버퍼 없이  : {count}바이트, read {}번 (마지막 1번은 EOF 확인)",
        direct.reads
    );
    let mut buffered = BufReader::new(CountingReader {
        inner: &data[..],
        reads: 0,
    });
    let count = buffered.by_ref().bytes().count();
    println!(
        "    BufReader  : {count}바이트, read {}번",
        buffered.get_ref().reads
    );
    // lines()도 BufReader가 채워 둔 버퍼 안에서 줄을 잘라 냅니다.
    let text = "가\n".repeat(5_000); // 4바이트 × 5000줄 = 20000바이트
    let mut buffered = BufReader::new(CountingReader {
        inner: text.as_bytes(),
        reads: 0,
    });
    let lines = buffered.by_ref().lines().count();
    println!(
        "    BufReader  : {lines}줄 lines(), read {}번",
        buffered.get_ref().reads
    );
    // 출력:
    //     버퍼 없이  : 10000바이트, read 10001번 (마지막 1번은 EOF 확인)
    //     BufReader  : 10000바이트, read 3번
    //     BufReader  : 5000줄 lines(), read 4번

    println!("3. BufWriter는 flush해야 쓰기 에러를 알 수 있습니다");
    // 8바이트짜리 고정 버퍼(&mut [u8])에 15바이트를 씁니다. 쓰기는 BufWriter 안에 모일 때까지는 성공합니다.
    let mut storage = [b'.'; 8];
    {
        let mut writer = BufWriter::new(&mut storage[..]);
        writer.write_all(b"hello, buffered")?; // 아직 BufWriter의 버퍼에만 있습니다.
        println!(
            "    write_all 뒤 BufWriter에 대기 중: {}바이트",
            writer.buffer().len()
        );
        // flush 없이 스코프를 벗어납니다. drop이 flush를 시도하지만, 실패해도 에러는 버려집니다.
    }
    println!(
        "    flush 없이 버림 → 결과 {:?} (에러를 알 방법이 없음)",
        String::from_utf8_lossy(&storage)
    );
    let mut storage = [b'.'; 8];
    let mut writer = BufWriter::new(&mut storage[..]);
    writer.write_all(b"hello, buffered")?;
    match writer.flush() {
        Ok(()) => println!("    flush → 성공"),
        Err(e) => println!("    flush → 에러 {:?}: {e}", e.kind()),
    }
    // 출력:
    //     write_all 뒤 BufWriter에 대기 중: 15바이트
    //     flush 없이 버림 → 결과 "hello, b" (에러를 알 방법이 없음)
    //     flush → 에러 WriteZero: failed to write the buffered data

    println!("4. LineWriter는 줄바꿈을 만날 때마다 안쪽으로 내보냅니다");
    let mut line_writer = LineWriter::new(CountingWriter {
        inner: Vec::new(),
        writes: 0,
    });
    for word in ["하나", "둘", "셋"] {
        writeln!(line_writer, "{word}")?;
    }
    println!(
        "    3줄 쓴 뒤      : write {}번",
        line_writer.get_ref().writes
    );
    write!(line_writer, "줄바꿈 없음")?; // 줄바꿈이 없으니 아직 버퍼에 남습니다.
    println!(
        "    줄바꿈 없이 쓴 뒤: write {}번",
        line_writer.get_ref().writes
    );
    line_writer.flush()?;
    println!(
        "    flush 뒤       : write {}번",
        line_writer.get_ref().writes
    );
    // 출력:
    //     3줄 쓴 뒤      : write 3번
    //     줄바꿈 없이 쓴 뒤: write 3번
    //     flush 뒤       : write 4번

    println!("5. 표준 출력을 한 번 잠그고(lock) BufWriter로 감싸기");
    // io::stdout()은 프로세스 전체가 함께 쓰는 핸들이라 쓸 때마다 잠금을 잡습니다.
    // lock()으로 한 번 잡아 두면 그동안 다른 곳의 출력이 끼어들지 않고 잠금 비용도 한 번입니다.
    let mut out = CountingWriter {
        inner: io::stdout().lock(),
        writes: 0,
    };
    for i in 1..=3 {
        writeln!(out, "    (잠근 표준 출력) {i}번째 줄")?;
    }
    let direct_writes = out.writes;
    // 다 쓴 잠금은 버려서 풉니다. 잠금을 쥐고 있는 동안에는 다른 스레드가 표준 출력에 쓸 수 없습니다.
    drop(out);
    let mut out = BufWriter::new(CountingWriter {
        inner: io::stdout().lock(),
        writes: 0,
    });
    for i in 1..=3 {
        writeln!(out, "    (BufWriter + 잠근 표준 출력) {i}번째 줄")?;
    }
    out.flush()?;
    let buffered_writes = out.get_ref().writes;
    drop(out);
    println!(
        "    StdoutLock에 직접: write {direct_writes}번, BufWriter를 거치면: write {buffered_writes}번"
    );
    // 출력:
    //     (잠근 표준 출력) 1번째 줄
    //     (잠근 표준 출력) 2번째 줄
    //     (잠근 표준 출력) 3번째 줄
    //     (BufWriter + 잠근 표준 출력) 1번째 줄
    //     (BufWriter + 잠근 표준 출력) 2번째 줄
    //     (BufWriter + 잠근 표준 출력) 3번째 줄
    //     StdoutLock에 직접: write 9번, BufWriter를 거치면: write 1번

    Ok(())
}
