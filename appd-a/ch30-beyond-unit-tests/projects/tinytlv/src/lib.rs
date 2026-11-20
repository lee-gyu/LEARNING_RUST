//! 태그-길이-값(TLV) 형식의 작은 이진 레코드 코덱입니다.
//!
//! 레코드 하나는 `[태그 1바이트][길이 1바이트][값 0~255바이트]`로 기록되고, 레코드를 이어 붙인
//! 바이트열이 메시지 하나입니다.
//!
//! ```
//! use tinytlv::{Record, encode, parse};
//!
//! let records = vec![Record::new(1, "온도")?, Record::new(2, [0x01, 0x2c])?];
//! let bytes = encode(&records);
//! assert_eq!(bytes, [1, 6, 0xec, 0x98, 0xa8, 0xeb, 0x8f, 0x84, 2, 2, 0x01, 0x2c]);
//! assert_eq!(parse(&bytes)?, records);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::error::Error;
use std::fmt;

mod cursor;
#[cfg(test)]
mod tests;

use cursor::Cursor;

/// 레코드 하나: 태그와 0~255바이트의 값.
///
/// 필드가 비공개라 [`Record::new`]나 [`parse`]를 거치지 않고는 만들 수 없습니다.
/// 그래서 값이 255바이트를 넘는 `Record`는 존재하지 않습니다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Record {
    tag: u8,
    value: Vec<u8>,
}

impl Record {
    /// 값의 최대 길이(바이트)입니다. 길이를 1바이트에 적기 때문입니다.
    pub const MAX_VALUE_LEN: usize = 255;

    /// 레코드를 만듭니다.
    ///
    /// ```
    /// use tinytlv::Record;
    ///
    /// let r = Record::new(7, "안녕")?;
    /// assert_eq!(r.tag(), 7);
    /// assert_eq!(r.value(), "안녕".as_bytes());
    /// # Ok::<(), tinytlv::ValueTooLong>(())
    /// ```
    ///
    /// # Errors
    ///
    /// 값이 [`Record::MAX_VALUE_LEN`]보다 길면 [`ValueTooLong`]을 돌려줍니다.
    pub fn new(tag: u8, value: impl Into<Vec<u8>>) -> Result<Self, ValueTooLong> {
        let value = value.into();
        if value.len() > Self::MAX_VALUE_LEN {
            return Err(ValueTooLong { len: value.len() });
        }
        Ok(Self { tag, value })
    }

    /// 태그를 돌려줍니다.
    pub fn tag(&self) -> u8 {
        self.tag
    }

    /// 값을 돌려줍니다.
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// 이 레코드를 기록하는 데 드는 바이트 수(머리 2바이트 + 값)입니다.
    pub fn encoded_len(&self) -> usize {
        2 + self.value.len()
    }

    /// 이 레코드를 `buf`의 앞쪽에 기록하고, 기록한 바이트 수를 돌려줍니다.
    ///
    /// ```
    /// use tinytlv::Record;
    ///
    /// let r = Record::new(3, [0xaa])?;
    /// let mut buf = [0u8; 8];
    /// let n = r.write_to(&mut buf);
    /// assert_eq!(buf[..n], [3, 1, 0xaa]);
    /// # Ok::<(), tinytlv::ValueTooLong>(())
    /// ```
    ///
    /// # Panics
    ///
    /// `buf`가 [`Record::encoded_len`]보다 짧으면 패닉합니다.
    #[track_caller]
    pub fn write_to(&self, buf: &mut [u8]) -> usize {
        let n = self.encoded_len();
        assert!(
            buf.len() >= n,
            "버퍼가 작습니다: {n}바이트가 필요한데 {}바이트입니다",
            buf.len()
        );
        let len = u8::try_from(self.value.len()).expect("Record는 값이 255바이트 이하입니다");
        buf[0] = self.tag;
        buf[1] = len;
        buf[2..n].copy_from_slice(&self.value);
        n
    }
}

/// [`Record::new`]에 255바이트보다 긴 값을 넘겼습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueTooLong {
    /// 넘긴 값의 길이
    pub len: usize,
}

impl fmt::Display for ValueTooLong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "값이 {}바이트라 최대 길이 {}바이트를 넘습니다",
            self.len,
            Record::MAX_VALUE_LEN
        )
    }
}

impl Error for ValueTooLong {}

/// [`parse`]가 받아들이지 못한 입력입니다. `offset`은 문제가 된 레코드가 시작하는 위치입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// 태그 바이트 뒤에 길이 바이트가 없습니다.
    MissingLength { offset: usize },
    /// 길이 바이트가 남은 입력보다 긴 값을 가리킵니다.
    ValueOverrun {
        offset: usize,
        len: usize,
        remaining: usize,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MissingLength { offset } => {
                write!(f, "{offset}번 위치의 레코드: 길이 바이트가 없습니다")
            }
            Self::ValueOverrun {
                offset,
                len,
                remaining,
            } => write!(
                f,
                "{offset}번 위치의 레코드: 값이 {len}바이트라고 했지만 {remaining}바이트만 남았습니다"
            ),
        }
    }
}

impl Error for ParseError {}

/// 바이트열을 레코드 목록으로 읽습니다. [`encode`]의 반대입니다.
///
/// 입력은 어디서 왔든(파일, 네트워크) 아무 바이트열이어도 됩니다.
/// 잘못된 입력에는 패닉하지 않고 [`ParseError`]를 돌려줍니다.
///
/// ```
/// use tinytlv::{ParseError, parse};
///
/// let records = parse(&[9, 0, 1, 2, 0xaa, 0xbb])?;
/// assert_eq!(records.len(), 2);
/// assert!(records[0].value().is_empty());
/// assert_eq!(records[1].value(), [0xaa, 0xbb]);
///
/// // 2번 위치에서 시작하는 레코드가 3바이트라고 했지만 1바이트만 남았습니다.
/// assert_eq!(
///     parse(&[9, 0, 1, 3, 0xaa]),
///     Err(ParseError::ValueOverrun { offset: 2, len: 3, remaining: 1 }),
/// );
/// # Ok::<(), ParseError>(())
/// ```
///
/// # Errors
///
/// 레코드의 머리나 값이 입력 끝에서 잘렸으면 [`ParseError`]를 돌려줍니다.
pub fn parse(input: &[u8]) -> Result<Vec<Record>, ParseError> {
    let mut cur = Cursor::new(input);
    let mut records = Vec::new();
    loop {
        let offset = cur.pos();
        let Some(tag) = cur.byte() else {
            break;
        };
        let len = cur.byte().ok_or(ParseError::MissingLength { offset })?;
        let len = usize::from(len);
        let remaining = cur.remaining();
        let value = cur.take(len).ok_or(ParseError::ValueOverrun {
            offset,
            len,
            remaining,
        })?;
        records.push(Record {
            tag,
            value: value.to_vec(),
        });
    }
    Ok(records)
}

/// 레코드 목록을 바이트열로 기록합니다. [`parse`]의 반대입니다.
///
/// ```
/// use tinytlv::{Record, encode, parse};
///
/// let records = [Record::new(1, [10, 20])?, Record::new(2, [])?];
/// let bytes = encode(&records);
/// assert_eq!(bytes, [1, 2, 10, 20, 2, 0]);
/// assert_eq!(parse(&bytes)?, records);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn encode(records: &[Record]) -> Vec<u8> {
    let total = records.iter().map(Record::encoded_len).sum();
    let mut out = vec![0; total];
    let mut pos = 0;
    for r in records {
        pos += r.write_to(&mut out[pos..]);
    }
    out
}
