//! 통합 테스트: 공개 API만 써서 동작을 확인합니다.
//! 이 파일은 별도의 크레이트로 컴파일되므로 사용자 코드와 같은 것만 보입니다.
//! (`Cursor`, `Record`의 필드, `#[cfg(test)]`로 가린 항목은 보이지 않습니다.)

use std::error::Error;

use tinytlv::{ParseError, Record, ValueTooLong, encode, parse};

#[test]
fn encode_then_parse_gives_back_the_same_records() -> Result<(), Box<dyn Error>> {
    let records = vec![
        Record::new(1, "온도")?,
        Record::new(2, [0x01, 0x2c])?,
        Record::new(3, [])?,
        Record::new(4, vec![0xff; Record::MAX_VALUE_LEN])?,
    ];
    let bytes = encode(&records);
    assert_eq!(bytes.len(), 8 + 4 + 2 + (2 + Record::MAX_VALUE_LEN));
    assert_eq!(parse(&bytes)?, records);
    Ok(())
}

#[test]
fn empty_input_is_an_empty_message() {
    assert_eq!(parse(&[]), Ok(vec![]));
}

#[test]
fn errors_can_be_compared_and_displayed() {
    let err = parse(&[5, 3, 1]).unwrap_err();
    assert_eq!(
        err,
        ParseError::ValueOverrun {
            offset: 0,
            len: 3,
            remaining: 1
        }
    );
    assert_eq!(
        err.to_string(),
        "0번 위치의 레코드: 값이 3바이트라고 했지만 1바이트만 남았습니다"
    );
}

/// 사용자 코드처럼 `?`로 이어 쓸 수 있는지 확인합니다.
#[test]
fn errors_work_with_question_mark() {
    fn user_code(bytes: &[u8]) -> Result<usize, Box<dyn Error>> {
        let mut records = parse(bytes)?;
        records.push(Record::new(9, vec![0; 300])?);
        Ok(records.len())
    }

    let err = user_code(&[1, 0]).unwrap_err();
    assert_eq!(err.downcast_ref(), Some(&ValueTooLong { len: 300 }));

    let err = user_code(&[1, 5]).unwrap_err();
    assert_eq!(
        err.downcast_ref(),
        Some(&ParseError::ValueOverrun {
            offset: 0,
            len: 5,
            remaining: 0
        })
    );
}
