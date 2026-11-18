// 기대하는 에러: E0382 — borrow of moved value: `file`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0382_file_moved_into_bufreader.rs --out-dir /tmp/rust-p1-ch9
//
// BufReader::new(file)은 File의 소유권을 가져갑니다. 감싼 뒤에는 원래 이름으로 쓸 수 없습니다.
// 해결 (셋 중 하나):
//   1) 감싼 값을 빌려 씁니다: reader.get_ref().metadata()  — 단, lines()는 reader를 소비하므로 그 전에 부릅니다.
//   2) 빌린 파일을 감쌉니다: BufReader::new(&file)  — &File도 Read를 구현합니다(Part 0 ch20의 &TcpStream과 같음).
//   3) 메타데이터를 먼저 읽어 둡니다: let size = file.metadata()?.len(); 다음에 BufReader::new(file)

use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() -> io::Result<()> {
    let file = File::open("Cargo.toml")?;
    let reader = BufReader::new(file);
    let count = reader.lines().count();
    let size = file.metadata()?.len();
    println!("{count}줄, {size}바이트");
    Ok(())
}
