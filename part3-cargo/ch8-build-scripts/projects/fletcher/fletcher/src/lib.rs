//! libfletcher를 안전하게 감싼 API입니다.

/// 빌드할 때 링크한 C 라이브러리의 버전입니다(빌드 스크립트가 넘김).
pub const C_VERSION: &str = env!("FLETCHER_C_VERSION");

/// `data`의 Fletcher-16 체크섬을 구합니다.
pub fn checksum(data: &[u8]) -> u16 {
    // SAFETY: `data.as_ptr()`부터 `data.len()`바이트는 읽을 수 있는 메모리이고,
    // C 함수는 그 범위만 읽고 포인터를 보관하지 않습니다.
    unsafe { fletcher_sys::fletcher16(data.as_ptr(), data.len()) }
}

/// `data`의 체크섬이 `expected`와 같은지 C 라이브러리로 확인합니다.
/// C 라이브러리 버전 2 이상과 링크했을 때만 있습니다.
#[cfg(fletcher_v2)]
pub fn verify(data: &[u8], expected: u16) -> bool {
    // SAFETY: `checksum`과 같습니다.
    unsafe { fletcher_sys::fletcher16_verify(data.as_ptr(), data.len(), expected) != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        assert_eq!(checksum(b"abcde"), 0xC8F0);
        assert_eq!(checksum(b"abcdef"), 0x2057);
        assert_eq!(checksum(b""), 0);
    }

    #[cfg(fletcher_v2)]
    #[test]
    fn verify_matches_checksum() {
        assert!(verify(b"abcde", 0xC8F0));
        assert!(!verify(b"abcde", 0x0000));
    }
}
