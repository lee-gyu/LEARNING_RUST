//! `lib.rs`의 단위 테스트입니다. `lib.rs`의 `#[cfg(test)] mod tests;`가 이 파일을 불러옵니다.
//! 같은 크레이트 안이라 비공개 필드까지 씁니다. 테스트 코드라 `unwrap()`도 괜찮습니다.

use super::*;

#[test]
fn new_accepts_255_bytes_and_rejects_256() {
    assert!(Record::new(1, vec![0; 255]).is_ok());
    assert_eq!(Record::new(1, vec![0; 256]), Err(ValueTooLong { len: 256 }));
}

#[test]
fn parse_builds_records_directly() {
    // 비공개 필드로 기대값을 직접 만듭니다. `Record::new`를 거치지 않으므로 이 테스트는 parse만 봅니다.
    let expected = vec![
        Record {
            tag: 1,
            value: vec![0xaa, 0xbb],
        },
        Record {
            tag: 2,
            value: vec![],
        },
    ];
    assert_eq!(parse(&[1, 2, 0xaa, 0xbb, 2, 0]).unwrap(), expected);
}

#[test]
fn error_offset_points_at_record_start() {
    assert_eq!(
        parse(&[1, 0, 2, 5, 0]),
        Err(ParseError::ValueOverrun {
            offset: 2,
            len: 5,
            remaining: 1
        })
    );
}

/// 회귀 테스트: 길이 바이트 없이 태그 바이트로 끝나는 입력.
/// 임의 바이트 테스트(`tests/arbitrary_bytes.rs`)가 찾아 줄 만한 입력을 고정해 둔 것입니다.
#[test]
fn regression_tag_without_length_byte() {
    assert_eq!(
        parse(&[1, 0, 7]),
        Err(ParseError::MissingLength { offset: 2 })
    );
}

/// 잘못된 인자에 패닉하는 공개 함수는 `#[should_panic]`으로 그 패닉을 확인합니다.
#[test]
#[should_panic(expected = "버퍼가 작습니다")]
fn write_to_panics_on_short_buffer() {
    let r = Record::new(1, [1, 2, 3]).unwrap();
    r.write_to(&mut [0; 4]);
}
