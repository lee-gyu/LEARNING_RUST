//! 지침 3: 피할 수 없는 `unsafe`는 안전한 래퍼 한 곳에 가둔다. (지침 5의 테스트도 함께)
//!
//! 염기 서열 파일에서 읽은 바이트(A, C, G, T)를 화면에 문자열로 보여 주고, 짧은 패턴을 찾고,
//! 상보 서열(A↔T, C↔G)을 만듭니다. A/C/G/T는 ASCII라서 올바른 UTF-8이므로, 확인만 했다면
//! `str::from_utf8_unchecked`로 검사 없이 `&str`을 얻을 수 있습니다. 표준 라이브러리의 `String`도
//! 같은 방법을 씁니다. `Vec<u8>`를 감싸 만들 때 한 번 UTF-8을 확인하고, `as_str`에서는 검사를 건너뜁니다.
//!
//! - 비권장: 바이트를 그대로 주고받고, 문자열이 필요한 곳마다 부르는 쪽이 확인한 뒤 `unsafe`로 바꿉니다.
//! - 권장: `DnaSeq`가 만들 때 한 번 확인하고, `unsafe`는 `as_str` 안의 한 곳에만 둡니다.
//!   필드가 비공개라 모듈 밖의 안전한 코드는 불변식을 깰 수 없습니다.
//!
//! 파일 첫 줄의 `#![deny(unsafe_code)]`가 `unsafe`를 금지하고, 허락한 모듈에서만 쓰게 합니다.
//! 이 파일의 `unsafe` 블록은 Miri로 검사하지 못했습니다(nightly 툴체인을 설치하지 않음).
//!
//! 실행: cargo run -p er-ch16-avoid-unsafe --example 04_safe_wrapper
//! 테스트: cargo test -p er-ch16-avoid-unsafe --example 04_safe_wrapper

#![deny(unsafe_code)]

use std::sync::atomic::{AtomicUsize, Ordering};

/// 전제 조건("A/C/G/T만 있다")을 확인한 횟수.
static CHECKS: AtomicUsize = AtomicUsize::new(0);

/// 염기가 아닌 첫 바이트의 위치. 모두 염기면 None. 부를 때마다 CHECKS를 1 늘립니다.
fn first_non_base(bytes: &[u8]) -> Option<usize> {
    CHECKS.fetch_add(1, Ordering::Relaxed);
    bytes
        .iter()
        .position(|b| !matches!(b, b'A' | b'C' | b'G' | b'T'))
}

fn complement_base(b: u8) -> u8 {
    match b {
        b'A' => b'T',
        b'T' => b'A',
        b'C' => b'G',
        b'G' => b'C',
        other => other,
    }
}

/// 비권장: 바이트 슬라이스를 그대로 주고받는 함수들.
#[expect(
    unsafe_code,
    reason = "비권장 판: unsafe fn을 열어 두는 모양을 보입니다"
)]
mod scattered {
    /// 바이트를 검사 없이 &str로 봅니다.
    ///
    /// # Safety
    ///
    /// `bytes`에는 A, C, G, T 바이트만 있어야 합니다.
    pub unsafe fn as_text(bytes: &[u8]) -> &str {
        // SAFETY: A/C/G/T만 있다는 것은 부르는 쪽이 보장합니다(# Safety). 그런 바이트열은 ASCII이고,
        // ASCII 바이트열은 올바른 UTF-8입니다.
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    pub fn complement(bytes: &mut [u8]) {
        for b in bytes {
            *b = super::complement_base(*b);
        }
    }
}

#[expect(
    unsafe_code,
    reason = "비권장 판: 부르는 곳마다 unsafe 블록이 생기는 모양을 보입니다"
)]
fn scattered_demo(samples: &[&[u8]]) {
    for (n, raw) in samples.iter().enumerate() {
        let mut bytes = raw.to_vec();
        if let Some(at) = first_non_base(&bytes) {
            println!(
                "    샘플 {}: 위치 {at}의 바이트가 염기가 아니라 건너뜀",
                n + 1
            );
            continue;
        }
        // SAFETY: 바로 위의 first_non_base가 None이었으므로 A/C/G/T만 있습니다.
        let shown = unsafe { scattered::as_text(&bytes) }.to_string();

        // 실제 프로그램에서는 보여 주기, 찾기, 상보 서열이 서로 다른 함수에 있습니다. 받은 Vec<u8>가
        // 확인된 것인지, 그 뒤에 누가 고치지 않았는지 타입만 보고는 알 수 없으므로 함수마다 다시 확인합니다.
        let found = match first_non_base(&bytes) {
            // SAFETY: 바로 위에서 다시 확인했습니다.
            None => unsafe { scattered::as_text(&bytes) }.find("GCG"),
            Some(_) => None,
        };

        scattered::complement(&mut bytes);
        // complement가 A/C/G/T 밖의 바이트를 만들지 않는다는 것은 그 본문을 읽어야 알 수 있습니다.
        // 여기서 보장할 방법은 다시 확인하는 것뿐입니다.
        if first_non_base(&bytes).is_some() {
            println!("    샘플 {}: 상보 서열이 깨짐", n + 1);
            continue;
        }
        // SAFETY: 바로 위에서 다시 확인했습니다.
        let complement = unsafe { scattered::as_text(&bytes) };
        println!(
            "    샘플 {}: {shown}, \"GCG\" 위치 {found:?}, 상보 {complement}",
            n + 1
        );
    }
}

/// 권장: unsafe를 이 모듈 하나에 가둔 안전한 래퍼.
///
/// 불변식: `DnaSeq::bases`에는 A, C, G, T 바이트만 있다.
/// `as_str`의 unsafe 블록은 이 불변식에 기대므로, `bases`를 고치는 이 모듈의 **모든** 코드가
/// 불변식을 지켜야 합니다. 모듈 밖의 코드는 비공개 필드에 닿을 수 없어 검토하지 않아도 됩니다.
///
/// 단, 이 예제는 두 판을 비교하려고 `first_non_base`와 `complement_base`를 모듈 밖에 두고 함께
/// 씁니다. 이 모듈이 부르는 두 함수도 검토 범위에 듭니다. 실제 코드라면 모듈 안에 둡니다.
#[allow(
    unsafe_code,
    reason = "이 모듈만 unsafe를 허락합니다. 안전성 검토는 이 모듈 안만 하면 됩니다"
)]
mod dna {
    use std::fmt;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DnaSeq {
        bases: Vec<u8>, // 비공개: 모듈 밖에서는 만들 수도, 고칠 수도 없습니다.
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct InvalidBase {
        pub position: usize,
        pub byte: u8,
    }

    impl fmt::Display for InvalidBase {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.byte.is_ascii_graphic() {
                let c = char::from(self.byte);
                write!(f, "위치 {}의 {c:?}는 염기가 아님", self.position)
            } else {
                write!(
                    f,
                    "위치 {}의 0x{:02X}는 염기가 아님",
                    self.position, self.byte
                )
            }
        }
    }

    impl DnaSeq {
        /// 불변식을 세우는 유일한 길입니다. 여기서 한 번만 확인합니다.
        pub fn from_bytes(bytes: Vec<u8>) -> Result<DnaSeq, InvalidBase> {
            match super::first_non_base(&bytes) {
                None => Ok(DnaSeq { bases: bytes }),
                Some(position) => Err(InvalidBase {
                    position,
                    byte: bytes[position],
                }),
            }
        }

        pub fn as_str(&self) -> &str {
            // SAFETY: 불변식에 따라 bases에는 A/C/G/T만 있습니다. 그런 바이트열은 ASCII이고,
            // ASCII 바이트열은 올바른 UTF-8입니다.
            unsafe { std::str::from_utf8_unchecked(&self.bases) }
        }

        /// A↔T, C↔G로만 바꾸므로 불변식이 유지됩니다.
        pub fn complement(&mut self) {
            for b in &mut self.bases {
                *b = super::complement_base(*b);
            }
        }
    }

    // 래퍼의 테스트는 불변식의 경계를 겨냥합니다. 거부해야 할 입력, 빈 입력,
    // 불변식을 유지해야 하는 메서드. Miri가 있다면 이 테스트들을 Miri로 돌립니다.
    #[cfg(test)]
    mod tests {
        use super::*;

        fn seq(bytes: &[u8]) -> Result<DnaSeq, InvalidBase> {
            DnaSeq::from_bytes(bytes.to_vec())
        }

        #[test]
        fn accepts_only_upper_acgt() {
            assert!(seq(b"ACGT").is_ok());
            for bad in [&b"acgt"[..], b"ACGU", b"AC GT", b"AC\xFF", "가".as_bytes()] {
                assert!(seq(bad).is_err(), "{bad:?}를 받아들임");
            }
        }

        #[test]
        fn reports_first_invalid_position() {
            let err = seq(b"ACXGY").unwrap_err();
            assert_eq!(
                err,
                InvalidBase {
                    position: 2,
                    byte: b'X'
                }
            );
        }

        #[test]
        fn empty_sequence_is_valid() {
            let s = seq(b"").unwrap();
            assert_eq!(s.as_str(), "");
        }

        #[test]
        fn as_str_shows_same_bytes() {
            let s = seq(b"GATTACA").unwrap();
            assert_eq!(s.as_str().as_bytes(), b"GATTACA");
        }

        #[test]
        fn complement_keeps_invariant() {
            let mut s = seq(b"ACGTTGCA").unwrap();
            s.complement();
            assert_eq!(s.as_str(), "TGCAACGT");
            assert!(seq(s.as_str().as_bytes()).is_ok()); // 다시 검사해도 통과
            s.complement();
            assert_eq!(s.as_str(), "ACGTTGCA"); // 두 번 하면 원래대로
        }
    }
}

fn wrapper_demo(samples: &[&[u8]]) {
    for (n, raw) in samples.iter().enumerate() {
        let mut seq = match dna::DnaSeq::from_bytes(raw.to_vec()) {
            Ok(seq) => seq,
            Err(e) => {
                println!("    샘플 {}: 만들 때 거부: {e}", n + 1);
                continue;
            }
        };
        // 여기부터는 unsafe도, 다시 확인할 일도 없습니다.
        let shown = seq.as_str().to_string();
        let found = seq.as_str().find("GCG");
        seq.complement();
        println!(
            "    샘플 {}: {shown}, \"GCG\" 위치 {found:?}, 상보 {}",
            n + 1,
            seq.as_str()
        );
    }
}

fn main() {
    let samples: [&[u8]; 3] = [b"ATGCGTAC", b"ATGUGA", b"GGCGCCAT"];

    println!("1. 비권장: 바이트를 주고받고, 부르는 곳마다 확인한 뒤 unsafe");
    CHECKS.store(0, Ordering::Relaxed);
    scattered_demo(&samples);
    // 출력:     샘플 1: ATGCGTAC, "GCG" 위치 Some(2), 상보 TACGCATG
    // 출력:     샘플 2: 위치 3의 바이트가 염기가 아니라 건너뜀
    // 출력:     샘플 3: GGCGCCAT, "GCG" 위치 Some(1), 상보 CCGCGGTA
    println!(
        "    전제 조건을 확인한 횟수: {}번",
        CHECKS.load(Ordering::Relaxed)
    ); // 출력:     전제 조건을 확인한 횟수: 7번

    println!("2. 권장: 안전한 래퍼 DnaSeq");
    CHECKS.store(0, Ordering::Relaxed);
    wrapper_demo(&samples);
    // 출력:     샘플 1: ATGCGTAC, "GCG" 위치 Some(2), 상보 TACGCATG
    // 출력:     샘플 2: 만들 때 거부: 위치 3의 'U'는 염기가 아님
    // 출력:     샘플 3: GGCGCCAT, "GCG" 위치 Some(1), 상보 CCGCGGTA
    println!(
        "    전제 조건을 확인한 횟수: {}번",
        CHECKS.load(Ordering::Relaxed)
    ); // 출력:     전제 조건을 확인한 횟수: 3번
}
