// 기대하는 에러: E0596 — cannot borrow `*reader` as mutable, as it is behind a `&` reference
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0596_read_through_shared_ref.rs --out-dir /tmp/rust-p1-ch9
//
// "읽기만 하니까 공유 참조(&)면 충분하겠지"라고 생각하기 쉽지만, Read::read는 &mut self를 받습니다.
// 읽으면 리더 안의 "읽기 위치"가 앞으로 움직이기 때문입니다. &[u8]이라면 슬라이스의 앞부분이 잘려 나가고,
// File이라면 운영체제가 관리하는 파일 위치가 바뀝니다.
// 해결: 매개변수를 &mut dyn Read(또는 제네릭 impl Read)로 받고, 넘길 때도 &mut로 넘깁니다.

use std::io::{self, Read};

fn count_bytes(reader: &dyn Read) -> io::Result<usize> {
    let mut buf = [0u8; 16];
    let mut total = 0;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            return Ok(total);
        }
        total += n;
    }
}

fn main() -> io::Result<()> {
    let data: &[u8] = b"hello";
    println!("{}바이트", count_bytes(&data)?);
    Ok(())
}
