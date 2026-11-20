//! core만으로 되는 부분. 힙을 쓰지 않으므로 결과를 담을 곳은 부르는 쪽이 마련합니다.

use core::fmt::{self, Write};

use crate::HexError;

const DIGITS: &[u8; 16] = b"0123456789abcdef";

/// 한 바이트를 소문자 16진수 두 글자로 바꿉니다. `Hex`와 alloc 기능의 `try_encode`가 함께 씁니다.
pub(crate) fn digits(b: u8) -> [char; 2] {
    [
        char::from(DIGITS[usize::from(b >> 4)]),
        char::from(DIGITS[usize::from(b & 0x0f)]),
    ]
}

/// 바이트 슬라이스를 소문자 16진수로 표시합니다. 할당 없이 `Formatter`에 바로 씁니다.
///
/// `write!`로 고정 크기 버퍼에 쓸 수도 있고, alloc이 있으면 `to_string()`이나 `format!`으로
/// 문자열을 만들 수도 있습니다.
#[derive(Debug, Clone, Copy)]
pub struct Hex<'a>(pub &'a [u8]);

impl fmt::Display for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &b in self.0 {
            let [hi, lo] = digits(b);
            f.write_char(hi)?;
            f.write_char(lo)?;
        }
        Ok(())
    }
}

/// 16진수 문자열을 `out`에 풀어 쓰고, 채운 앞부분을 돌려줍니다. 대소문자를 모두 받습니다.
///
/// ```
/// let mut buf = [0u8; 8];
/// let bytes = tinyhex::decode_to_slice("CAFE", &mut buf).unwrap();
/// assert_eq!(bytes, [0xca, 0xfe]);
/// ```
pub fn decode_to_slice<'a>(hex: &str, out: &'a mut [u8]) -> Result<&'a [u8], HexError> {
    let (pairs, rest) = hex.as_bytes().as_chunks::<2>();
    if !rest.is_empty() {
        return Err(HexError::OddLength { len: hex.len() });
    }
    let needed = pairs.len();
    let available = out.len();
    let Some(out) = out.get_mut(..needed) else {
        return Err(HexError::BufferTooSmall { needed, available });
    };
    for (i, (slot, [hi, lo])) in out.iter_mut().zip(pairs).enumerate() {
        *slot = (nibble(hex, 2 * i, *hi)? << 4) | nibble(hex, 2 * i + 1, *lo)?;
    }
    Ok(out)
}

/// 16진수 숫자 하나를 값으로 바꿉니다.
fn nibble(hex: &str, index: usize, b: u8) -> Result<u8, HexError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => {
            // 앞의 바이트는 모두 ASCII 숫자였으므로 `index`는 글자 경계입니다.
            let ch = hex[index..]
                .chars()
                .next()
                .unwrap_or(char::REPLACEMENT_CHARACTER);
            Err(HexError::InvalidDigit { index, ch })
        }
    }
}

// 단위 테스트는 이 크레이트와 같은 `#![no_std]` 크레이트 안에서 빌드됩니다. 프렐류드도 core의
// 것이라서, std의 이름은 lib.rs의 `#[cfg(test)] extern crate std;` 덕분에 `std::`로 씁니다.
#[cfg(test)]
mod tests {
    use super::*;
    use std::format;

    #[test]
    fn display_writes_lowercase() {
        assert_eq!(format!("{}", Hex(&[0x00, 0xab, 0xff])), "00abff");
    }

    #[test]
    fn decodes_into_caller_buffer() {
        let mut buf = [0u8; 4];
        assert_eq!(decode_to_slice("0aFf", &mut buf), Ok(&[0x0a, 0xff][..]));
        assert_eq!(buf, [0x0a, 0xff, 0, 0]);
    }

    #[test]
    fn reports_each_error() {
        let mut buf = [0u8; 1];
        assert_eq!(
            decode_to_slice("abc", &mut buf),
            Err(HexError::OddLength { len: 3 })
        );
        assert_eq!(
            decode_to_slice("a가", &mut [0u8; 2]),
            Err(HexError::InvalidDigit {
                index: 1, ch: '가'
            })
        );
        assert_eq!(
            decode_to_slice("abcd", &mut buf),
            Err(HexError::BufferTooSmall {
                needed: 2,
                available: 1
            })
        );
    }
}
