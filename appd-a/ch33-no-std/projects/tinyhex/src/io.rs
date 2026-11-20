//! std 기능의 부분: 운영체제의 입출력(`std::io`)과 이어 줍니다.
//!
//! `std::io`는 1.100.0의 stable에서 std에만 있습니다(`core::io`는 nightly 전용). 그래서 이 모듈만
//! std 기능 뒤에 둡니다. 크레이트 루트가 `#![no_std]`이므로 이 모듈에서도 `String`은 직접 가져옵니다.

use alloc::string::String;
use alloc::vec::Vec;
use std::io;

use crate::{Hex, HexError};

/// `out`에 16진수를 씁니다.
pub fn write_hex<W: io::Write>(out: &mut W, bytes: &[u8]) -> io::Result<()> {
    write!(out, "{}", Hex(bytes))
}

/// `input`에서 16진수 문자열을 모두 읽어 바이트로 바꿉니다. 앞뒤 공백은 무시합니다.
pub fn read_hex<R: io::Read>(mut input: R) -> io::Result<Vec<u8>> {
    let mut text = String::new();
    input.read_to_string(&mut text)?;
    // 아래 `From` 구현 덕분에 `HexError`도 `?`로 `io::Error`가 됩니다.
    Ok(crate::decode(text.trim())?)
}

/// `HexError`를 `io::Error`로 바꿉니다. 원래 에러는 `io::Error` 안에 그대로 담깁니다.
impl From<HexError> for io::Error {
    fn from(e: HexError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }
}
