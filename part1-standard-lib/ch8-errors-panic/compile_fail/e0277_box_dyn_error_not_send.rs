// 기대하는 에러: E0277 — `dyn std::error::Error` cannot be sent between threads safely
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_box_dyn_error_not_send.rs --out-dir /tmp/rust-p1-ch8
//
// thread::spawn의 클로저가 돌려주는 값은 join을 부른 스레드로 옮겨지므로 Send여야 합니다.
// Box<dyn Error>는 "Error를 구현한 아무 타입"이라 Rc를 품은 에러처럼 Send가 아닌 것도 담을 수 있으므로,
// 컴파일러는 이 상자를 다른 스레드로 보내도 된다고 보장할 수 없습니다.
// 해결: 에러 타입을 Box<dyn Error + Send + Sync>로 바꿉니다. &str, String, ParseIntError 같은
//       Send + Sync 에러는 그대로 ?로 변환됩니다. (examples/04_box_dyn_error.rs의 parse_usage_sendable)

use std::error::Error;
use std::thread;

fn parse_usage(line: &str) -> Result<u32, Box<dyn Error>> {
    let (_, value) = line.split_once('=').ok_or("'='가 없습니다")?;
    Ok(value.trim().parse()?)
}

fn main() {
    let handle = thread::spawn(|| parse_usage("kim=42"));
    let result = handle.join().expect("작업 스레드는 패닉하지 않아야 합니다");
    println!("{result:?}");
}
