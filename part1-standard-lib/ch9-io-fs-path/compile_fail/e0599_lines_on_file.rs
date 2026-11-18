// 기대하는 에러: E0599 — the method `lines` exists for struct `File`, but its trait bounds were not satisfied
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0599_lines_on_file.rs --out-dir /tmp/rust-p1-ch9
//
// BufRead를 가져왔으니 lines()를 부를 수 있을 것 같지만, File은 Read만 구현하고 BufRead는 구현하지 않습니다.
// 줄을 나누려면 줄바꿈이 나올 때까지 미리 읽어 둘 버퍼가 필요한데, File에는 그런 버퍼가 없기 때문입니다.
// 해결: BufReader로 감쌉니다. for line in BufReader::new(file).lines() { ... }
//       (examples/02_bufread_lines.rs의 5번 구역)

use std::fs::File;
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let file = File::open("Cargo.toml")?;
    for line in file.lines() {
        println!("{}", line?);
    }
    Ok(())
}
