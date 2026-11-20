use std::fmt;

use crate::Config;

/// 파싱 에러.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    MissingEquals { line: usize },
    EmptyKey { line: usize },
    DuplicateKey { line: usize, key: String },
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

/// 텍스트를 파싱합니다.
///
/// # Arguments
///
/// * `text` - 파싱할 `String`
///
/// # Returns
///
/// `Config` 또는 `ParseError`
///
/// 같은 키가 여러 번 나오면 마지막 값이 이깁니다.
///
/// 서버의 `main`이 설정 파일을 읽은 직후에 부릅니다.
///
/// ```text
/// let config = parse(&text).unwrap();
/// let port = config.value("port").unwrap();
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
