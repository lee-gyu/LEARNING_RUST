// 기대하는 에러: E0599 — cannot write into `File`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0599_writeln_file_with_fmt_write.rs --out-dir /tmp/rust-p1-ch9
//
// ch3에서 String에 write!를 쓰려고 std::fmt::Write를 가져왔던 습관대로, 파일에 쓸 때도 fmt::Write를 가져왔습니다.
// write!/writeln!은 첫 인자의 write_fmt 메서드를 부르는데, File의 write_fmt는 io::Write 트레이트에 있습니다.
// fmt::Write는 문자열을 받는 곳(String, Formatter)을 위한 트레이트이고 File은 이것을 구현하지 않습니다.
// 해결: use std::io::Write; 로 바꿉니다. 한 파일에서 둘 다 써야 하면
//       use std::fmt::Write as _; 처럼 한쪽을 이름 없이 가져옵니다(examples/01_read_write_traits.rs).

use std::fmt::Write;
use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("p1-ch9-compile-fail.txt");
    let mut file = File::create(&path)?;
    writeln!(file, "점수: {}", 95)?;
    Ok(())
}
