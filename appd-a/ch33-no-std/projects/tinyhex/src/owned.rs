//! alloc 기능의 부분: 결과를 `String`·`Vec`으로 돌려줍니다.
//!
//! 크레이트가 `#![no_std]`라서 이 이름들은 프렐류드에 없습니다. std 기능이 켜진 빌드에서도
//! 마찬가지이므로 alloc 경로로 가져옵니다.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::slice::digits;
use crate::{Hex, HexError, decode_to_slice};

/// 바이트를 16진수 문자열로 바꿉니다.
///
/// 할당이 실패하면 돌려줄 방법이 없습니다. std를 링크한 프로그램에서는 프로세스가 중단됩니다.
/// 힙이 제한된 환경에서는 [`try_encode`]를 씁니다.
pub fn encode(bytes: &[u8]) -> String {
    // core의 `Display`(Hex)와 alloc의 `ToString`을 이어 붙인 것입니다.
    Hex(bytes).to_string()
}

/// [`encode`]의 실패할 수 있는 판입니다.
///
/// 필요한 용량을 `try_reserve_exact`로 먼저 확보하고, 그 용량 안에서만 `push`하므로 그 뒤로는
/// 할당이 일어나지 않습니다.
pub fn try_encode(bytes: &[u8]) -> Result<String, HexError> {
    let mut s = String::new();
    s.try_reserve_exact(bytes.len() * 2)
        .map_err(HexError::OutOfMemory)?;
    for &b in bytes {
        s.extend(digits(b)); // 확보한 용량 안이므로 할당하지 않음
    }
    Ok(s)
}

/// 16진수 문자열을 바이트로 바꿉니다. 할당 실패는 [`encode`]와 같이 다룹니다.
pub fn decode(hex: &str) -> Result<Vec<u8>, HexError> {
    let mut out = vec![0; hex.len() / 2];
    decode_to_slice(hex, &mut out)?;
    Ok(out)
}

/// [`decode`]의 실패할 수 있는 판입니다.
pub fn try_decode(hex: &str) -> Result<Vec<u8>, HexError> {
    let mut out = Vec::new();
    out.try_reserve_exact(hex.len() / 2)
        .map_err(HexError::OutOfMemory)?;
    out.resize(hex.len() / 2, 0); // 확보한 용량 안이므로 할당하지 않음
    decode_to_slice(hex, &mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::error::Error;
    use std::string::ToString;

    #[test]
    fn fallible_and_infallible_agree() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        assert_eq!(try_encode(&bytes), Ok(encode(&bytes)));
        assert_eq!(try_decode("DEADbeef"), decode("DEADbeef"));
    }

    #[test]
    fn out_of_memory_keeps_its_source() {
        // 용량 오버플로는 결정적으로 실패하므로 `TryReserveError`를 만들 때 씁니다.
        let reason = Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err();
        let err = HexError::OutOfMemory(reason);
        assert_eq!(err.to_string(), "결과를 담을 메모리를 확보하지 못했습니다");
        let source = err.source().map(|s| s.to_string());
        assert_eq!(
            source.as_deref(),
            Some(
                "memory allocation failed because the computed capacity exceeded the collection's maximum"
            )
        );
    }
}
