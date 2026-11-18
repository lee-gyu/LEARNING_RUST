// BufRead로 줄 단위(또는 구분자 단위)로 읽고, 읽는 함수를 제네릭으로 만들어 메모리 버퍼로 시험합니다.
//
//   1) lines(): 줄마다 io::Result<String>을 내주고, 줄 끝의 \n이나 \r\n을 떼어 냅니다.
//   2) read_line(): 줄 끝 문자까지 포함해 String 뒤에 덧붙이고, 읽은 바이트 수를 돌려줍니다(0이면 EOF).
//   3) split()/read_until(): 줄바꿈이 아닌 다른 바이트로 나눕니다. 결과는 Vec<u8>입니다.
//   4) UTF-8이 아닌 줄을 만나면 lines()는 그 줄에서 InvalidData 에러를 냅니다.
//   5) impl BufRead를 받는 함수는 &[u8], Cursor, BufReader<File>, 표준 입력에 모두 쓸 수 있습니다.
//
// 실행: cargo run -p p1-ch9-io-fs-path --example 02_bufread_lines
// 테스트: cargo test -p p1-ch9-io-fs-path --example 02_bufread_lines
//
// 5번 구역은 std::env::temp_dir() 아래에 이 예제 전용 디렉터리를 만들고, 끝날 때 지웁니다.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Cursor};

// ── 5번 구역의 함수: 가장 긴 줄의 번호(1부터)와 내용을 찾습니다 ─────────────
// 매개변수가 impl BufRead라서 어디서 읽는지 모릅니다. 그 덕분에 파일 없이 바이트 슬라이스로 시험할 수 있습니다.
// 길이는 바이트가 아니라 글자(char) 수로 잽니다.
fn longest_line(input: impl BufRead) -> io::Result<Option<(usize, String)>> {
    let mut best: Option<(usize, String)> = None;
    for (index, line) in input.lines().enumerate() {
        let line = line?; // 읽다가 실패하면(UTF-8이 아님 등) 그대로 돌려보냅니다.
        let longer = match &best {
            Some((_, b)) => line.chars().count() > b.chars().count(),
            None => true,
        };
        if longer {
            best = Some((index + 1, line));
        }
    }
    Ok(best)
}

fn main() -> io::Result<()> {
    // CRLF(\r\n) 줄과, 줄바꿈 없이 끝나는 마지막 줄을 일부러 넣었습니다.
    let text = "첫째 줄\n둘째 줄\r\n셋째 줄";

    println!("1. lines()는 줄 끝의 \\n과 \\r\\n을 떼어 냅니다");
    // &[u8]은 BufRead도 구현합니다. 이미 메모리에 있으니 따로 버퍼가 필요 없기 때문입니다.
    for line in text.as_bytes().lines() {
        println!("    {:?}", line?);
    }
    // 출력:
    //     "첫째 줄"
    //     "둘째 줄"
    //     "셋째 줄"

    println!("2. read_line()은 줄 끝 문자까지 덧붙이고, 0을 돌려주면 EOF입니다");
    let mut reader = text.as_bytes();
    let mut line = String::new();
    loop {
        line.clear(); // 비우지 않으면 앞 줄 뒤에 계속 덧붙습니다.
        let n = reader.read_line(&mut line)?;
        println!("    {n:>2}바이트 {line:?}");
        if n == 0 {
            break;
        }
    }
    // 출력:
    //     11바이트 "첫째 줄\n"
    //     12바이트 "둘째 줄\r\n"
    //     10바이트 "셋째 줄"
    //      0바이트 ""

    println!("3. split()과 read_until()은 다른 바이트로 나눕니다");
    // &[u8]에는 같은 이름의 고유 메서드 slice::split(클로저를 받음)이 있어서 그쪽이 먼저 골라집니다.
    // 그래서 BufRead::split을 쓰려면 Cursor로 감싸거나 BufRead::split(슬라이스, b';')로 부릅니다.
    let fields: Vec<Vec<u8>> = Cursor::new("a;bb;;ccc;")
        .split(b';')
        .collect::<io::Result<_>>()?;
    let fields: Vec<String> = fields
        .iter()
        .map(|f| String::from_utf8_lossy(f).into_owned())
        .collect();
    println!("    split(b';') → {fields:?}");
    let mut reader = Cursor::new("key=value");
    let mut key = Vec::new();
    let n = reader.read_until(b'=', &mut key)?; // 구분자까지 포함해 Vec 뒤에 덧붙입니다.
    println!(
        "    read_until(b'=') → {n}바이트 {:?}",
        String::from_utf8_lossy(&key)
    );
    // 출력:
    //     split(b';') → ["a", "bb", "", "ccc"]
    //     read_until(b'=') → 4바이트 "key="

    println!("4. UTF-8이 아닌 줄이 있으면 lines()는 그 줄에서 에러를 냅니다");
    let raw: &[u8] = b"ok\n\xff\xfe\nafter\n"; // 둘째 줄은 UTF-8이 아닌 바이트입니다.
    for line in raw.lines() {
        match line {
            Ok(l) => println!("    Ok({l:?})"),
            Err(e) => println!("    Err({:?}: {e})", e.kind()),
        }
    }
    // 바이트로 받으면 실패하지 않습니다. 고칠 수 없는 바이트는 U+FFFD(�)로 바꿔 보여 줍니다.
    let lossy: Vec<String> = BufRead::split(raw, b'\n')
        .map(|bytes| bytes.map(|b| String::from_utf8_lossy(&b).into_owned()))
        .collect::<io::Result<_>>()?;
    println!("    바이트로 읽고 from_utf8_lossy → {lossy:?}");
    // 출력:
    //     Ok("ok")
    //     Err(InvalidData: stream did not contain valid UTF-8)
    //     Ok("after")
    //     바이트로 읽고 from_utf8_lossy → ["ok", "��", "after"]

    println!("5. impl BufRead를 받는 함수는 어디서 읽든 똑같이 동작합니다");
    let poem = "산\n들판을 건너\n강";
    // (1) 바이트 슬라이스
    println!("    &[u8]           → {:?}", longest_line(poem.as_bytes())?);
    // (2) Cursor<Vec<u8>>: 소유한 버퍼에서 읽습니다.
    println!(
        "    Cursor<Vec<u8>> → {:?}",
        longest_line(Cursor::new(poem.as_bytes().to_vec()))?
    );
    // (3) 파일: File은 BufRead가 아니므로 BufReader로 감싸서 넘깁니다.
    let dir = std::env::temp_dir().join(format!("p1-ch9-02-{}", std::process::id()));
    fs::create_dir_all(&dir)?;
    let path = dir.join("poem.txt");
    fs::write(&path, poem)?;
    let file = File::open(&path)?;
    println!(
        "    BufReader<File> → {:?}",
        longest_line(BufReader::new(file))?
    );
    // (4) 내용이 하나도 없는 리더: io::empty()는 늘 EOF입니다.
    println!("    io::empty()     → {:?}", longest_line(io::empty())?);
    fs::remove_dir_all(&dir)?;
    // 출력:
    //     &[u8]           → Some((2, "들판을 건너"))
    //     Cursor<Vec<u8>> → Some((2, "들판을 건너"))
    //     BufReader<File> → Some((2, "들판을 건너"))
    //     io::empty()     → None

    Ok(())
}

// 파일을 만들지 않고 바이트 슬라이스만으로 읽기 함수를 시험합니다.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_first_longest_line() {
        let result = longest_line("ab\n가나다\nxyz\n".as_bytes()).unwrap();
        // "가나다"와 "xyz"는 글자 수가 같으므로 먼저 나온 2번째 줄을 고릅니다.
        assert_eq!(result, Some((2, "가나다".to_string())));
    }

    #[test]
    fn reports_invalid_utf8() {
        let err = longest_line(&b"ok\n\xff\n"[..]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
