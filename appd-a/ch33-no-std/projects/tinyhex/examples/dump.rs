//! tinyhex의 std 기능 API를 쓰는 보통의 std 프로그램입니다.
//!
//! 실행(projects/tinyhex 디렉터리에서): cargo run --example dump
//! `Cargo.toml`의 `required-features = ["std"]` 때문에 `--no-default-features`로는 빌드되지 않습니다.

use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut out = io::stdout().lock();

    tinyhex::write_hex(&mut out, "Rust".as_bytes())?;
    writeln!(out)?; // 출력: 52757374

    let bytes = tinyhex::read_hex("  52757374\n".as_bytes())?;
    writeln!(out, "{}", String::from_utf8_lossy(&bytes))?; // 출력: Rust

    // 잘못된 입력은 `HexError`가 `io::Error`(InvalidData)로 바뀌어 돌아옵니다.
    match tinyhex::read_hex("5275zz".as_bytes()) {
        Ok(b) => writeln!(out, "예상과 달리 성공: {b:?}")?,
        Err(e) => writeln!(out, "{:?}: {e}", e.kind())?, // 출력: InvalidData: 위치 4의 'z'는 16진수 숫자가 아닙니다
    }
    Ok(())
}
