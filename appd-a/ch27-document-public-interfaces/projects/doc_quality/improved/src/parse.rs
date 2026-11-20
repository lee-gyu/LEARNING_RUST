//! 텍스트를 줄 단위로 읽어 [`Config`]를 만듭니다. 비공개 모듈이라 이 문서는 공개 문서에
//! 나오지 않고, 이 파일을 고치는 사람을 위한 설명입니다.

use std::fmt;

use crate::Config;

/// [`parse`]가 받아들이지 않은 줄과 그 이유입니다.
///
/// 줄 번호는 편집기에 보이는 번호처럼 1부터 셉니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// 빈 줄도 주석도 아닌데 `=`가 없습니다.
    MissingEquals {
        /// 1부터 센 줄 번호
        line: usize,
    },
    /// `=` 앞이 비어 있습니다(`= 8080`).
    EmptyKey {
        /// 1부터 센 줄 번호
        line: usize,
    },
    /// 앞에서 이미 나온 키입니다.
    DuplicateKey {
        /// 두 번째로 나온 줄의 번호(1부터)
        line: usize,
        /// 겹친 키(앞뒤 공백을 지운 것)
        key: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingEquals { line } => write!(f, "{line}번째 줄에 `=`가 없습니다"),
            ParseError::EmptyKey { line } => write!(f, "{line}번째 줄의 키가 비어 있습니다"),
            ParseError::DuplicateKey { line, key } => {
                write!(f, "{line}번째 줄: `{key}` 키가 이미 있습니다")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// 설정 파일의 내용을 읽어 [`Config`]를 만듭니다.
///
/// 형식은 [크레이트 문서](crate)에 있습니다. 첫 번째 잘못된 줄에서 멈추고, 그 앞에서 읽은
/// 줄도 돌려주지 않습니다. 일부만 읽힌 설정으로 프로그램이 시작하는 일을 막기 위해서입니다.
///
/// # Errors
///
/// - 빈 줄도 주석도 아닌데 `=`가 없으면 [`ParseError::MissingEquals`]
/// - `=` 앞이 비어 있으면 [`ParseError::EmptyKey`]
/// - 앞에서 이미 나온 키이면 [`ParseError::DuplicateKey`]. 나중 값으로 덮어쓰지 않는 것은,
///   설정 파일에서 같은 키를 두 번 적는 일은 대개 실수이기 때문입니다.
///
/// # Examples
///
/// ```
/// use kvconf::ParseError;
///
/// let err = kvconf::parse("port = 8080\nport = 9090").unwrap_err();
/// assert_eq!(err, ParseError::DuplicateKey { line: 2, key: "port".to_string() });
/// assert_eq!(err.to_string(), "2번째 줄: `port` 키가 이미 있습니다");
/// ```
pub fn parse(text: &str) -> Result<Config, ParseError> {
    let mut config = Config::default();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            return Err(ParseError::MissingEquals { line });
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(ParseError::EmptyKey { line });
        }
        if config.get(key).is_some() {
            let key = key.to_string();
            return Err(ParseError::DuplicateKey { line, key });
        }
        config.insert(key, value.trim());
    }
    Ok(config)
}
