// Read, Write, Seek 트레이트의 기본 동작을 메모리 버퍼로 확인합니다.
//
//   1) Read::read는 버퍼를 "최대" buf.len()바이트까지 채우고, 실제로 채운 개수를 돌려줍니다.
//      요청보다 적게 줄 수 있고, 0은 "더 읽을 것이 없음(EOF)"입니다.
//   2) read_exact / read_to_end / read_to_string은 read를 여러 번 불러 주는 제공 메서드입니다.
//   3) Write::write도 일부만 쓸 수 있습니다. write_all이 끝까지 반복해 줍니다.
//   4) write!/writeln!은 io::Write(바이트를 받는 곳)와 fmt::Write(문자열을 받는 곳)에 모두 쓰입니다.
//   5) Seek: Cursor는 메모리 버퍼에 "읽기·쓰기 위치"를 붙여 파일처럼 다루게 해 줍니다.
//
// 실행: cargo run -p p1-ch9-io-fs-path --example 01_read_write_traits
//
// 파일을 쓰지 않습니다. &[u8], Vec<u8>, &mut [u8], Cursor처럼 메모리에 있는 값도
// Read/Write를 구현하므로, 파일이나 네트워크 없이 입출력 코드를 시험할 수 있습니다.

use std::fmt::Write as _; // String에 write!를 쓰기 위한 fmt::Write (이름은 io::Write와 겹치므로 _로 가져옴)
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

// ── 직접 만든 리더: 한 번의 read에서 최대 max바이트만 내줍니다 ─────────────
// 파이프나 네트워크에서는 데이터가 조금씩 도착하므로 read가 요청보다 적게 주는 일이 흔합니다.
// 그 상황을 메모리 안에서 재현하려고 안쪽 리더의 read를 max바이트로 잘라서 부릅니다.
// Read를 구현할 때 안정 버전에서 직접 쓰는 메서드는 read 하나이고, 나머지는 모두 이것을 바탕으로 동작합니다.
struct Trickle<R> {
    inner: R,
    max: usize,
    calls: usize, // read가 몇 번 불렸는지 셉니다.
}

impl<R: Read> Read for Trickle<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.calls += 1;
        let limit = buf.len().min(self.max);
        self.inner.read(&mut buf[..limit])
    }
}

// ── 직접 만든 라이터: 한 번의 write에서 최대 max바이트만 받습니다 ──────────
// Write의 필수 메서드는 write와 flush 두 개입니다.
struct Narrow {
    written: Vec<u8>,
    max: usize,
    calls: usize,
}

impl Write for Narrow {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.calls += 1;
        let n = buf.len().min(self.max);
        self.written.extend_from_slice(&buf[..n]);
        Ok(n) // 받은 만큼만 "썼다"고 알립니다. 나머지를 다시 보내는 것은 호출한 쪽의 몫입니다.
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(()) // 따로 모아 둔 것이 없으므로 할 일이 없습니다.
    }
}

fn main() -> io::Result<()> {
    println!("1. read는 요청보다 적게 줄 수 있고, 0이면 EOF입니다");
    // &[u8]도 Read를 구현합니다. 읽을 때마다 슬라이스의 앞부분이 잘려 나갑니다.
    let mut reader = Trickle {
        inner: "hello, world!".as_bytes(),
        max: 5,
        calls: 0,
    };
    let mut buf = [0u8; 8];
    loop {
        let n = reader.read(&mut buf)?;
        println!(
            "    read(8바이트 버퍼) → {n}바이트 {:?}",
            String::from_utf8_lossy(&buf[..n])
        );
        if n == 0 {
            break;
        }
    }
    // 출력:
    //     read(8바이트 버퍼) → 5바이트 "hello"
    //     read(8바이트 버퍼) → 5바이트 ", wor"
    //     read(8바이트 버퍼) → 3바이트 "ld!"
    //     read(8바이트 버퍼) → 0바이트 ""

    println!("2. read_exact / read_to_end / read_to_string은 read를 반복해 부릅니다");
    let mut reader = Trickle {
        inner: "0123456789".as_bytes(),
        max: 3,
        calls: 0,
    };
    let mut head = [0u8; 8];
    reader.read_exact(&mut head)?; // 8바이트를 다 채울 때까지 read를 반복합니다.
    println!(
        "    read_exact(8) → {:?}, read 호출 {}번",
        String::from_utf8_lossy(&head),
        reader.calls
    );
    // 남은 것은 2바이트뿐이므로 4바이트를 채울 수 없습니다.
    let mut more = [0u8; 4];
    match reader.read_exact(&mut more) {
        Ok(()) => println!("    read_exact(4) → 성공"),
        Err(e) => println!("    read_exact(4) → 에러 {:?}: {e}", e.kind()),
    }
    // 출력:
    //     read_exact(8) → "01234567", read 호출 3번
    //     read_exact(4) → 에러 UnexpectedEof: failed to fill whole buffer

    let mut reader = Trickle {
        inner: "가나다".as_bytes(), // 한글 한 글자는 UTF-8로 3바이트입니다.
        max: 2,
        calls: 0,
    };
    let mut bytes = Vec::new();
    let n = reader.read_to_end(&mut bytes)?; // EOF(0)가 나올 때까지 Vec 뒤에 덧붙입니다.
    println!(
        "    read_to_end → {n}바이트 {bytes:?}, read 호출 {}번",
        reader.calls
    );
    // 2바이트씩 읽으면 글자가 중간에서 잘리지만, read_to_string은 끝까지 모은 뒤 UTF-8을 검사하므로 괜찮습니다.
    let mut reader = Trickle {
        inner: "가나다".as_bytes(),
        max: 2,
        calls: 0,
    };
    let mut text = String::from("앞: ");
    reader.read_to_string(&mut text)?; // 기존 내용 뒤에 덧붙입니다(read_line과 같음).
    println!("    read_to_string → {text:?}");
    // 출력:
    //     read_to_end → 9바이트 [234, 176, 128, 235, 130, 152, 235, 139, 164], read 호출 6번
    //     read_to_string → "앞: 가나다"

    println!("3. write도 일부만 쓸 수 있고, write_all이 끝까지 반복합니다");
    let mut writer = Narrow {
        written: Vec::new(),
        max: 4,
        calls: 0,
    };
    let n = writer.write(b"hello, world")?; // 12바이트를 넘겼지만 4바이트만 받았습니다.
    println!(
        "    write(12바이트) → Ok({n}), 지금까지 {:?}",
        String::from_utf8_lossy(&writer.written)
    );
    writer.write_all(b" / write_all")?; // 다 쓸 때까지 write를 반복합니다.
    writer.flush()?;
    println!(
        "    write_all 뒤 → {:?}, write 호출 {}번",
        String::from_utf8_lossy(&writer.written),
        writer.calls
    );
    // 출력:
    //     write(12바이트) → Ok(4), 지금까지 "hell"
    //     write_all 뒤 → "hell / write_all", write 호출 4번

    println!("4. write!는 io::Write와 fmt::Write에 모두 쓰입니다");
    // io::Write: Vec<u8>에 바이트로 씁니다. 반환 타입은 io::Result<()>입니다.
    let mut bytes: Vec<u8> = Vec::new();
    writeln!(bytes, "{}+{}={}", 2, 3, 2 + 3)?;
    write!(bytes, "끝")?;
    println!("    Vec<u8>  ← {bytes:?}");
    println!(
        "    String::from_utf8 → {:?}",
        String::from_utf8(bytes).unwrap()
    );
    // fmt::Write: String에 문자로 씁니다. 반환 타입은 fmt::Result이고, String에 쓰기는 실패하지 않습니다.
    let mut text = String::new();
    writeln!(text, "{}+{}={}", 2, 3, 2 + 3).unwrap();
    println!("    String   ← {text:?}");
    // io::Write는 실패할 수 있습니다. 크기가 정해진 &mut [u8]에 넘치게 쓰면 WriteZero 에러가 납니다.
    let mut fixed = [0u8; 6];
    let mut slot: &mut [u8] = &mut fixed;
    let result = write!(slot, "{}", 123_456_789);
    println!(
        "    [u8; 6]  ← {:?}, 결과 {result:?}",
        String::from_utf8_lossy(&fixed)
    );
    // 출력:
    //     Vec<u8>  ← [50, 43, 51, 61, 53, 10, 235, 129, 157]
    //     String::from_utf8 → "2+3=5\n끝"
    //     String   ← "2+3=5\n"
    //     [u8; 6]  ← "123456", 결과 Err(Error { kind: WriteZero, message: "failed to write whole buffer" })

    println!("5. Seek: Cursor로 읽기·쓰기 위치를 옮깁니다");
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_all(b"0123456789")?;
    println!("    10바이트 쓴 뒤 위치 = {}", cursor.stream_position()?);
    let mut three = [0u8; 3];
    cursor.seek(SeekFrom::Start(2))?; // 처음에서 2바이트
    cursor.read_exact(&mut three)?;
    println!(
        "    Start(2)에서 3바이트 → {:?}, 위치 = {}",
        String::from_utf8_lossy(&three),
        cursor.stream_position()?
    );
    let pos = cursor.seek(SeekFrom::Current(2))?; // 지금 위치에서 2바이트 뒤로
    println!("    Current(2) → 위치 = {pos}");
    let pos = cursor.seek(SeekFrom::End(-2))?; // 끝에서 2바이트 앞
    let mut tail = String::new();
    cursor.read_to_string(&mut tail)?;
    println!("    End(-2) → 위치 = {pos}, 끝까지 읽으면 {tail:?}");
    cursor.rewind()?; // seek(SeekFrom::Start(0))과 같습니다.
    cursor.write_all(b"AB")?; // 처음 두 바이트를 덮어씁니다(끼워 넣지 않음).
    println!(
        "    rewind 뒤 \"AB\"를 쓰면 → {:?}",
        String::from_utf8_lossy(cursor.get_ref())
    );
    // 출력:
    //     10바이트 쓴 뒤 위치 = 10
    //     Start(2)에서 3바이트 → "234", 위치 = 5
    //     Current(2) → 위치 = 7
    //     End(-2) → 위치 = 8, 끝까지 읽으면 "89"
    //     rewind 뒤 "AB"를 쓰면 → "AB23456789"

    Ok(())
}
