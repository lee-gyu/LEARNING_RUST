// io::Error를 자세히 들여다보고, "어느 파일에서" 실패했는지 알려 주도록 감쌉니다.
//
//   1) 운영체제가 알려 준 실패: kind()로 종류를, raw_os_error()로 운영체제의 에러 번호를 봅니다.
//   2) 직접 만드는 io::Error: io::Error::new(종류, 내용), io::Error::other(내용), ErrorKind에서 변환
//   3) io::Error의 메시지에는 경로가 없습니다. 경로를 함께 담은 에러 타입으로 감싸 보고합니다.
//
// 실행: cargo run -p p1-ch9-io-fs-path --example 06_io_errors
//
// std::env::temp_dir() 아래에 이 예제 전용 디렉터리(p1-ch9-06-<프로세스 ID>)를 만들고, 끝날 때 지웁니다.
// 에러 메시지와 에러 번호는 macOS에서 확인한 값입니다. 운영체제마다 다릅니다
// (예: DirectoryNotEmpty의 번호는 macOS 66, Linux 39). 3번 구역의 출력에는 임시 디렉터리 경로가 나옵니다.

use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, Read};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

// ── 2번 구역: 내용이 잘못됐을 때 io::Error를 직접 만듭니다 ─────────────────
// 반환 타입을 io::Result 하나로 유지하고 싶을 때 쓰는 방법입니다.
fn parse_port(mut input: impl BufRead) -> io::Result<u16> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        // 내용이 아예 없음: 종류와 메시지(&str)로 만듭니다.
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "포트 줄이 없습니다",
        ));
    }
    // 숫자가 아님: 원래 에러(ParseIntError)를 그대로 안에 담습니다.
    line.trim()
        .parse::<u16>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

// ── 3번 구역: 어떤 파일에서 무엇을 하다 실패했는지 담는 에러 타입 ──────────
// 원인 io::Error는 그대로 품어 source()로 내줍니다(ch8의 감싸기).
#[derive(Debug)]
struct FileError {
    action: &'static str, // "읽지", "만들지" 같은 동작
    path: PathBuf,
    source: io::Error,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Path는 Display가 없으므로 display()로 출력합니다(08 예제).
        write!(
            f,
            "파일을 {} 못했습니다: {}",
            self.action,
            self.path.display()
        )
    }
}

impl Error for FileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

// fs 함수를 부르는 자리에서 경로를 붙입니다. 경로는 여기서만 알 수 있기 때문입니다.
fn read_text(path: &Path) -> Result<String, FileError> {
    fs::read_to_string(path).map_err(|source| FileError {
        action: "읽지",
        path: path.to_path_buf(),
        source,
    })
}

// ch8의 보고 함수와 같은 모양입니다. source 체인을 따라 내려갑니다.
fn report(err: &dyn Error) {
    println!("    에러: {err}");
    let mut current = err.source();
    while let Some(cause) = current {
        println!("      원인: {cause}");
        current = cause.source();
    }
}

fn show_os_error<T>(label: &str, result: io::Result<T>) {
    match result {
        Ok(_) => println!("    {label} → 성공"),
        Err(e) => {
            let code = e
                .raw_os_error()
                .map_or("없음".to_string(), |n| n.to_string());
            println!("    {label} → {:?}, OS 번호 {code}, \"{e}\"", e.kind());
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let dir = std::env::temp_dir().join(format!("p1-ch9-06-{}", std::process::id()));
    fs::create_dir_all(dir.join("sub"))?;
    let file = dir.join("data.txt");
    fs::write(&file, "1234")?;
    fs::write(dir.join("latin1.txt"), b"caf\xe9")?; // UTF-8이 아닌 바이트(Latin-1의 é)

    println!("1. 실패를 일으켜 kind()와 raw_os_error()를 봅니다");
    show_os_error("없는 파일 열기", File::open(dir.join("missing.txt")));
    show_os_error("있는 파일 create_new", File::create_new(&file));
    show_os_error("파일 아래 경로 열기", File::open(file.join("child")));
    show_os_error("비지 않은 디렉터리 삭제", fs::remove_dir(&dir));
    show_os_error(
        "디렉터리를 파일로 읽기",
        fs::read_to_string(dir.join("sub")),
    );
    // 아래 두 가지는 운영체제가 아니라 Rust 표준 라이브러리가 판단한 실패라서 OS 번호가 없습니다.
    show_os_error(
        "UTF-8 아닌 파일 읽기",
        fs::read_to_string(dir.join("latin1.txt")),
    );
    let mut eight = [0u8; 8];
    show_os_error(
        "4바이트 파일에 read_exact(8)",
        File::open(&file).and_then(|mut f| f.read_exact(&mut eight)),
    );
    // 출력:
    //     없는 파일 열기 → NotFound, OS 번호 2, "No such file or directory (os error 2)"
    //     있는 파일 create_new → AlreadyExists, OS 번호 17, "File exists (os error 17)"
    //     파일 아래 경로 열기 → NotADirectory, OS 번호 20, "Not a directory (os error 20)"
    //     비지 않은 디렉터리 삭제 → DirectoryNotEmpty, OS 번호 66, "Directory not empty (os error 66)"
    //     디렉터리를 파일로 읽기 → IsADirectory, OS 번호 21, "Is a directory (os error 21)"
    //     UTF-8 아닌 파일 읽기 → InvalidData, OS 번호 없음, "stream did not contain valid UTF-8"
    //     4바이트 파일에 read_exact(8) → UnexpectedEof, OS 번호 없음, "failed to fill whole buffer"

    println!("2. io::Error를 직접 만듭니다");
    for input in ["8080\n", "80x\n", ""] {
        match parse_port(input.as_bytes()) {
            Ok(port) => println!("    {input:?} → 포트 {port}"),
            Err(e) => println!("    {input:?} → {:?}: {e}", e.kind()),
        }
    }
    // io::Error::new로 담은 원래 에러는 get_ref()로 빌려 보거나 downcast()로 되찾을 수 있습니다.
    let err = parse_port("80x".as_bytes()).unwrap_err();
    println!("    get_ref() → {:?}", err.get_ref());
    println!("    raw_os_error() → {:?}", err.raw_os_error());
    let original: Result<ParseIntError, io::Error> = err.downcast();
    println!("    downcast::<ParseIntError>() → {original:?}");
    // 그 밖의 만드는 방법
    let other = io::Error::other("설정 형식이 잘못됐습니다"); // 종류가 Other인 new의 줄임
    let bare = io::Error::from(io::ErrorKind::NotFound); // 메시지 없이 종류만 (할당 없음)
    let os = io::Error::from_raw_os_error(2); // 운영체제 에러 번호에서
    println!("    other(..)              → {:?}: {other}", other.kind());
    println!("    from(ErrorKind)        → {:?}: {bare}", bare.kind());
    println!("    from_raw_os_error(2)   → {:?}: {os}", os.kind());
    // 출력:
    //     "8080\n" → 포트 8080
    //     "80x\n" → InvalidData: invalid digit found in string
    //     "" → UnexpectedEof: 포트 줄이 없습니다
    //     get_ref() → Some(ParseIntError { kind: InvalidDigit })
    //     raw_os_error() → None
    //     downcast::<ParseIntError>() → Ok(ParseIntError { kind: InvalidDigit })
    //     other(..)              → Other: 설정 형식이 잘못됐습니다
    //     from(ErrorKind)        → NotFound: entity not found
    //     from_raw_os_error(2)   → NotFound: No such file or directory (os error 2)

    println!("3. 어느 파일에서 실패했나요?");
    // (가) 그냥 ?로 올려 보내면 메시지에 경로가 없어 어느 파일인지 알 수 없습니다.
    let plain = fs::read_to_string(dir.join("app.conf")).unwrap_err();
    println!("  (가) io::Error 그대로");
    println!("    에러: {plain}");
    // (나) 경로를 담은 에러 타입으로 감싸면 보고서에 경로와 원인이 모두 남습니다.
    println!("  (나) FileError로 감싸기");
    let wrapped = read_text(&dir.join("app.conf")).unwrap_err();
    report(&wrapped);
    // 감싸도 원인의 종류는 그대로 물어볼 수 있습니다.
    println!(
        "    원인의 kind() = {:?}, 파일이 없는 경우인가 {}",
        wrapped.source.kind(),
        wrapped.source.kind() == io::ErrorKind::NotFound
    );
    // (다) 타입을 io::Error로 유지하며 메시지에만 경로를 붙이는 방법. 종류는 남지만 OS 번호는 사라집니다.
    println!("  (다) 메시지에 경로를 붙인 새 io::Error");
    let path = dir.join("sub");
    let rebuilt = fs::read_to_string(&path)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
        .unwrap_err();
    println!("    에러: {rebuilt}");
    println!(
        "    kind() = {:?}, raw_os_error() = {:?}",
        rebuilt.kind(),
        rebuilt.raw_os_error()
    );
    // 출력 (<임시 디렉터리>는 환경마다 다른 경로입니다):
    //   (가) io::Error 그대로
    //     에러: No such file or directory (os error 2)
    //   (나) FileError로 감싸기
    //     에러: 파일을 읽지 못했습니다: <임시 디렉터리>/p1-ch9-06-<프로세스 ID>/app.conf
    //       원인: No such file or directory (os error 2)
    //     원인의 kind() = NotFound, 파일이 없는 경우인가 true
    //   (다) 메시지에 경로를 붙인 새 io::Error
    //     에러: <임시 디렉터리>/p1-ch9-06-<프로세스 ID>/sub: Is a directory (os error 21)
    //     kind() = IsADirectory, raw_os_error() = None

    fs::remove_dir_all(&dir)?;
    Ok(())
}
