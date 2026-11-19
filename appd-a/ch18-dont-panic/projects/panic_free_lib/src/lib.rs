//! 타이머 길이("1h30m", "45s", "2h5s")를 초 단위로 읽고 쓰는 라이브러리.
//!
//! 이 크레이트는 패닉하지 않는다고 약속합니다. 사람의 주의에 기대지 않도록
//! `Cargo.toml`의 `[lints.clippy]`가 패닉할 수 있는 코드를 `cargo clippy`의 에러로 막습니다.
//! 실패는 모두 `Result`로 돌려줍니다.

use std::error::Error;
use std::fmt;

/// `parse_timer`가 실패한 이유.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerError {
    /// 빈 문자열
    Empty,
    /// 단위 앞에 숫자가 없음 (예: "h30m")
    MissingNumber(char),
    /// 마지막 숫자 뒤에 단위가 없음 (예: "90")
    MissingUnit,
    /// h, m, s가 아닌 단위 (예: "3d")
    UnknownUnit(char),
    /// u32 초로 담을 수 없을 만큼 김
    TooLong,
}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "타이머 길이가 비어 있습니다"),
            Self::MissingNumber(unit) => write!(f, "단위 {unit:?} 앞에 숫자가 없습니다"),
            Self::MissingUnit => write!(f, "마지막 숫자 뒤에 단위(h, m, s)가 없습니다"),
            Self::UnknownUnit(unit) => write!(f, "알 수 없는 단위입니다: {unit:?}"),
            Self::TooLong => write!(f, "타이머가 너무 깁니다"),
        }
    }
}

impl Error for TimerError {}

/// "1h30m"처럼 숫자와 단위(h, m, s)를 이어 쓴 타이머 길이를 초로 바꿉니다.
///
/// # Errors
///
/// 형식이 틀리면 그 이유를, 결과가 `u32` 초를 넘으면 [`TimerError::TooLong`]을 돌려줍니다.
///
/// # Examples
///
/// ```
/// use panic_free_lib::{TimerError, parse_timer};
///
/// assert_eq!(parse_timer("1h30m"), Ok(5400));
/// assert_eq!(parse_timer("3d"), Err(TimerError::UnknownUnit('d')));
/// ```
pub fn parse_timer(spec: &str) -> Result<u32, TimerError> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err(TimerError::Empty);
    }
    let mut total: u32 = 0;
    let mut number: Option<u32> = None;
    for ch in spec.chars() {
        if let Some(digit) = ch.to_digit(10) {
            // number * 10 + digit을 검사 연산으로 씁니다. 그냥 쓰면 arithmetic_side_effects가 막습니다.
            let next = number
                .unwrap_or(0)
                .checked_mul(10)
                .and_then(|n| n.checked_add(digit))
                .ok_or(TimerError::TooLong)?;
            number = Some(next);
            continue;
        }
        let unit = match ch {
            'h' => 3600,
            'm' => 60,
            's' => 1,
            other => return Err(TimerError::UnknownUnit(other)),
        };
        let n = number.take().ok_or(TimerError::MissingNumber(ch))?;
        let seconds = n.checked_mul(unit).ok_or(TimerError::TooLong)?;
        total = total.checked_add(seconds).ok_or(TimerError::TooLong)?;
    }
    if number.is_some() {
        return Err(TimerError::MissingUnit);
    }
    Ok(total)
}

/// 초를 "1h30m5s" 꼴로 씁니다. 0인 단위는 빼고, 0초는 "0s"입니다.
pub fn format_timer(seconds: u32) -> String {
    // 0이 아닌 상수로 나누는 /와 %는 패닉하지 않으므로 arithmetic_side_effects도 허용합니다.
    let (hours, rest) = (seconds / 3600, seconds % 3600);
    let (minutes, secs) = (rest / 60, rest % 60);
    let parts: Vec<String> = [(hours, 'h'), (minutes, 'm'), (secs, 's')]
        .into_iter()
        .filter(|&(value, _)| value > 0)
        .map(|(value, unit)| format!("{value}{unit}"))
        .collect();
    if parts.is_empty() {
        "0s".to_string()
    } else {
        parts.concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 테스트에서는 unwrap과 인덱싱을 씁니다. clippy.toml이 테스트 코드에서만 허용합니다.
    #[test]
    fn reads_units() {
        assert_eq!(parse_timer("1h30m").unwrap(), 5400);
        assert_eq!(parse_timer(" 45s ").unwrap(), 45);
        assert_eq!(parse_timer("2h5s").unwrap(), 7205);
    }

    #[test]
    fn rejects_bad_specs() {
        let cases = [
            ("", TimerError::Empty),
            ("h30m", TimerError::MissingNumber('h')),
            ("90", TimerError::MissingUnit),
            ("3d", TimerError::UnknownUnit('d')),
            ("1193047h", TimerError::TooLong),
        ];
        for (spec, expected) in cases {
            assert_eq!(parse_timer(spec).unwrap_err(), expected, "{spec:?}");
        }
    }

    #[test]
    fn round_trips() {
        let samples = [0, 59, 61, 3600, 5415, u32::MAX];
        for seconds in samples {
            assert_eq!(parse_timer(&format_timer(seconds)).unwrap(), seconds);
        }
        assert_eq!(format_timer(samples[4]), "1h30m15s");
    }
}
