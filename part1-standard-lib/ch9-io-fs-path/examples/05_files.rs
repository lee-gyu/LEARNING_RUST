// 파일을 만들고, 쓰고, 읽고, 덧붙이고, 복사·이름 바꾸기·삭제를 합니다.
//
//   1) fs::write / fs::read_to_string / fs::read: 파일 전체를 한 번에 다루는 편의 함수
//   2) File::create + writeln!, File::open + BufReader: 조금씩 쓰고 줄 단위로 읽기
//   3) OpenOptions: 뒤에 덧붙이기(append), 없을 때만 새로 만들기(create_new)
//   4) 열린 File에는 읽기·쓰기 위치가 하나 있어서, 끝까지 읽은 뒤 다시 읽으려면 rewind가 필요합니다.
//   5) 메타데이터: 크기, 종류, 수정 시각, 읽기 전용 여부
//   6) fs::copy / fs::rename / fs::remove_file
//
// 실행: cargo run -p p1-ch9-io-fs-path --example 05_files
//
// std::env::temp_dir() 아래에 이 예제 전용 디렉터리(p1-ch9-05-<프로세스 ID>)를 만들고, 끝날 때 지웁니다.
// 임시 디렉터리의 위치는 환경마다 다르므로 출력에는 파일 이름만 씁니다.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

// 경로 대신 파일 이름만 보여 주는 도우미입니다(경로 다루기는 08 예제).
fn name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn main() -> io::Result<()> {
    let dir = std::env::temp_dir().join(format!("p1-ch9-05-{}", std::process::id()));
    fs::create_dir_all(&dir)?;

    println!("1. 편의 함수: 파일 전체를 한 번에 쓰고 읽습니다");
    let memo = dir.join("memo.txt");
    fs::write(&memo, "첫 줄\n둘째 줄\n")?; // 없으면 만들고, 있으면 내용을 지우고 씁니다.
    let text = fs::read_to_string(&memo)?; // 내용 전체를 String으로 (UTF-8이 아니면 에러)
    let bytes = fs::read(&memo)?; // 내용 전체를 Vec<u8>로 (어떤 바이트든 괜찮음)
    println!("    read_to_string → {text:?}");
    println!("    read           → {}바이트", bytes.len());
    // 출력:
    //     read_to_string → "첫 줄\n둘째 줄\n"
    //     read           → 19바이트

    println!("2. File::create로 조금씩 쓰고, File::open + BufReader로 한 줄씩 읽습니다");
    let scores = dir.join("scores.txt");
    {
        let mut file = File::create(&scores)?; // 쓰기 전용으로 엽니다. 있던 내용은 지워집니다.
        for (who, score) in [("민수", 82), ("서연", 95), ("지훈", 77)] {
            writeln!(file, "{who} {score}")?; // File은 io::Write를 구현합니다.
        }
    } // file이 버려지면서 파일이 닫힙니다. File에는 따로 버퍼가 없어 writeln!마다 바로 씁니다.
    let reader = BufReader::new(File::open(&scores)?); // 읽기 전용으로 엽니다.
    let mut total = 0;
    for line in reader.lines() {
        let line = line?;
        let score: u32 = line.rsplit(' ').next().unwrap_or("0").parse().unwrap_or(0);
        total += score;
        println!("    {line}");
    }
    println!("    합계 {total}");
    // 출력:
    //     민수 82
    //     서연 95
    //     지훈 77
    //     합계 254

    println!("3. OpenOptions: 덧붙이기와 새로 만들기 전용");
    let log = dir.join("app.log");
    for message in ["시작", "작업 1 완료", "종료"] {
        // append(true): 쓰기가 늘 파일 끝에 붙습니다. create(true): 없으면 만듭니다.
        let mut file = OpenOptions::new().append(true).create(true).open(&log)?;
        writeln!(file, "[log] {message}")?;
    }
    for line in fs::read_to_string(&log)?.lines() {
        println!("    {line}");
    }
    // create_new(true): 파일이 이미 있으면 덮어쓰지 않고 AlreadyExists로 실패합니다.
    // "확인하고 나서 만들기"를 운영체제가 한 번에 해 주므로, 그 사이에 다른 프로그램이 끼어들 틈이 없습니다.
    match File::options().write(true).create_new(true).open(&log) {
        Ok(_) => println!("    create_new → 새로 만들었습니다"),
        Err(e) => println!("    create_new → {:?}: {e}", e.kind()),
    }
    // 출력:
    //     [log] 시작
    //     [log] 작업 1 완료
    //     [log] 종료
    //     create_new → AlreadyExists: File exists (os error 17)

    println!("4. 열린 File의 읽기 위치는 하나뿐입니다");
    let mut file = File::open(&memo)?;
    let mut first = String::new();
    file.read_to_string(&mut first)?;
    let mut second = String::new();
    file.read_to_string(&mut second)?; // 이미 끝에 있으므로 읽을 것이 없습니다.
    println!("    첫 번째 read_to_string → {}바이트", first.len());
    println!("    두 번째 read_to_string → {}바이트", second.len());
    file.rewind()?; // Seek: 위치를 처음으로 되돌립니다.
    second.clear();
    file.read_to_string(&mut second)?;
    println!("    rewind 뒤              → {}바이트", second.len());
    // 출력:
    //     첫 번째 read_to_string → 19바이트
    //     두 번째 read_to_string → 0바이트
    //     rewind 뒤              → 19바이트

    println!("5. 메타데이터");
    let meta = fs::metadata(&scores)?;
    println!(
        "    {}: {}바이트, 파일인가 {}, 디렉터리인가 {}",
        name(&scores),
        meta.len(),
        meta.is_file(),
        meta.is_dir()
    );
    let dir_meta = fs::metadata(&dir)?;
    println!("    예제 디렉터리: 디렉터리인가 {}", dir_meta.is_dir());
    // 수정 시각은 실행할 때마다 다르므로 값 대신 얻을 수 있었는지만 출력합니다.
    println!(
        "    수정 시각을 얻었나 {}, 읽기 전용인가 {}",
        meta.modified().is_ok(),
        meta.permissions().readonly()
    );
    // 출력:
    //     scores.txt: 30바이트, 파일인가 true, 디렉터리인가 false
    //     예제 디렉터리: 디렉터리인가 true
    //     수정 시각을 얻었나 true, 읽기 전용인가 false

    println!("6. 복사, 이름 바꾸기, 삭제");
    let backup = dir.join("scores.bak");
    let copied = fs::copy(&scores, &backup)?; // 복사한 바이트 수를 돌려줍니다.
    println!(
        "    copy   {} → {}: {copied}바이트",
        name(&scores),
        name(&backup)
    );
    let renamed = dir.join("scores-2024.txt");
    fs::rename(&scores, &renamed)?; // 같은 파일 시스템 안에서 옮기기·이름 바꾸기
    println!(
        "    rename {} → {}: 원래 이름 있나 {}, 새 이름 있나 {}",
        name(&scores),
        name(&renamed),
        fs::exists(&scores)?,
        fs::exists(&renamed)?
    );
    fs::remove_file(&backup)?;
    match fs::metadata(&backup) {
        Ok(_) => println!("    remove_file 뒤에도 남아 있습니다"),
        Err(e) => println!("    remove_file 뒤 metadata → {:?}", e.kind()),
    }
    // 출력:
    //     copy   scores.txt → scores.bak: 30바이트
    //     rename scores.txt → scores-2024.txt: 원래 이름 있나 false, 새 이름 있나 true
    //     remove_file 뒤 metadata → NotFound

    fs::remove_dir_all(&dir)?; // 예제가 만든 디렉터리를 내용째 지웁니다(07 예제).
    Ok(())
}
