//! `csrc/semver.h`의 `semver_parse`를 Rust로 다시 구현한 정적 라이브러리.
//!
//! C 프로그램(`csrc/main.c`)은 바꾸지 않고, 링크할 때 `csrc/semver.c` 대신 이 라이브러리(`libsemver_rs.a`)를
//! 넘기면 이 구현이 쓰입니다. 형식 판단은 안전한 Rust 함수 `parse`가 맡고, `unsafe`는 C에서 받은 포인터를
//! 다루는 `semver_parse` 안에만 둡니다.

use std::ffi::{CStr, c_char, c_int};

/// C의 `semver_t`와 같은 배치. 필드 순서와 타입을 헤더와 맞춥니다.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// 점으로 나눈 숫자 세 개를 읽습니다. 형식이 맞지 않거나 u32를 넘으면 `None`입니다.
pub fn parse(text: &str) -> Option<SemVer> {
    let mut parts = [0u32; 3];
    let mut count = 0;
    for piece in text.split('.') {
        // u32::from_str는 앞의 '+'를 받아들이므로, C 구현처럼 숫자만 있는지 먼저 봅니다.
        if piece.is_empty() || !piece.bytes().all(|b| b.is_ascii_digit()) || count == 3 {
            return None;
        }
        parts[count] = piece.parse().ok()?;
        count += 1;
    }
    if count != 3 {
        return None;
    }
    Some(SemVer {
        major: parts[0],
        minor: parts[1],
        patch: parts[2],
    })
}

/// `int semver_parse(const char *text, semver_t *out);`
///
/// # Safety
///
/// `text`는 NULL이거나 NUL로 끝나는 유효한 C 문자열이어야 하고, `out`은 NULL이거나 `SemVer` 하나를 쓸 수 있는
/// 유효한 포인터여야 합니다.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn semver_parse(text: *const c_char, out: *mut SemVer) -> c_int {
    if text.is_null() || out.is_null() {
        return -1;
    }
    // SAFETY: text는 NULL이 아니고, 호출자가 NUL로 끝나는 유효한 문자열을 넘긴다고 약속했습니다.
    let text = unsafe { CStr::from_ptr(text) };
    // UTF-8이 아니면 숫자와 점만으로 된 문자열일 수 없으므로 형식 오류입니다.
    let Some(version) = text.to_str().ok().and_then(parse) else {
        return -1;
    };
    // SAFETY: out은 NULL이 아니고, 호출자가 SemVer 하나를 쓸 수 있는 포인터를 넘긴다고 약속했습니다.
    unsafe { out.write(version) };
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 형식이_맞는_입력() {
        let v = SemVer {
            major: 10,
            minor: 20,
            patch: 30,
        };
        assert_eq!(parse("10.20.30"), Some(v));
        assert_eq!(parse("4294967295.0.0").map(|v| v.major), Some(u32::MAX));
    }

    #[test]
    fn 형식이_틀린_입력() {
        for text in [
            "",
            "1.2",
            "1.2.3.4",
            "1..3",
            "v1.2.3",
            " 1.2.3",
            "+1.2.3",
            "1.2.3-beta",
        ] {
            assert_eq!(parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn u32를_넘으면_형식_오류() {
        assert_eq!(parse("4294967296.0.0"), None);
    }

    #[test]
    fn c처럼_포인터로_부르기() {
        let mut out = SemVer {
            major: 9,
            minor: 9,
            patch: 9,
        };
        // SAFETY: 리터럴 C 문자열과 지역 변수의 포인터를 넘깁니다.
        assert_eq!(unsafe { semver_parse(c"1.2.3".as_ptr(), &mut out) }, 0);
        assert_eq!((out.major, out.minor, out.patch), (1, 2, 3));
        // 실패하면 out을 건드리지 않습니다(헤더의 약속).
        // SAFETY: 위와 같습니다.
        assert_eq!(unsafe { semver_parse(c"1.2".as_ptr(), &mut out) }, -1);
        assert_eq!((out.major, out.minor, out.patch), (1, 2, 3));
        // SAFETY: NULL은 함수가 먼저 검사합니다.
        assert_eq!(unsafe { semver_parse(std::ptr::null(), &mut out) }, -1);
    }
}
