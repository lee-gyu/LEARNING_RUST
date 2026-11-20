//! 통합 테스트는 tinyhex를 의존성으로 쓰는 별도의 보통 크레이트입니다. `#![no_std]`가 아니므로
//! std의 프렐류드(`format!`, `Vec`, `String`)를 그대로 씁니다. `cfg(feature = "…")`는 같은 패키지의
//! 기능을 가리키므로, 켠 기능에 맞는 테스트만 컴파일됩니다.

use tinyhex::{Hex, HexError, decode_to_slice};

#[test]
fn core_api_roundtrip() {
    let original = [0x00, 0x7f, 0x80, 0xff];
    let text = format!("{}", Hex(&original));
    let mut buf = [0u8; 4];
    assert_eq!(decode_to_slice(&text, &mut buf), Ok(&original[..]));
    assert_eq!(
        decode_to_slice("0", &mut buf),
        Err(HexError::OddLength { len: 1 })
    );
}

#[cfg(feature = "alloc")]
#[test]
fn alloc_api_roundtrip() {
    let original = "가나다".as_bytes();
    assert_eq!(tinyhex::encode(original), "eab080eb8298eb8ba4");
    assert_eq!(
        tinyhex::try_decode(&tinyhex::encode(original)).as_deref(),
        Ok(original)
    );
}

#[cfg(feature = "std")]
#[test]
fn std_api_reports_invalid_data() {
    let mut out = Vec::new();
    tinyhex::write_hex(&mut out, b"\x01\x02").unwrap();
    assert_eq!(out, b"0102");

    let err = tinyhex::read_hex("01zz".as_bytes()).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    // io::Error 안에 담긴 HexError를 꺼냅니다.
    let inner = err.get_ref().and_then(|e| e.downcast_ref::<HexError>());
    // `#[non_exhaustive]`이므로 다른 크레이트에서 하는 `match`에는 `_` 갈래가 있어야 합니다.
    let index = match inner {
        Some(HexError::InvalidDigit { index, .. }) => Some(*index),
        Some(_) | None => None,
    };
    assert_eq!(index, Some(2));
}
