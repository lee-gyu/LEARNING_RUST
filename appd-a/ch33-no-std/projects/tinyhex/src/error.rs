//! 에러 타입. `Error` 트레이트는 1.81부터 core에 있으므로 std 기능 없이도 구현됩니다.

/// 이 크레이트의 함수가 돌려주는 에러입니다.
///
/// `alloc` 기능을 켜면 `OutOfMemory` 배리언트가 더해집니다. 기능으로 가린 배리언트가 쓰는 쪽의
/// `match`를 깨지 않도록 `#[non_exhaustive]`를 붙였습니다.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum HexError {
    /// 16진수 숫자는 두 글자가 한 바이트이므로 길이가 짝수여야 합니다.
    #[error("16진수 문자열의 길이가 홀수입니다({len}바이트)")]
    OddLength { len: usize },

    /// 16진수 숫자가 아닌 글자가 있습니다. `index`는 0부터 센 바이트 위치입니다.
    #[error("위치 {index}의 {ch:?}는 16진수 숫자가 아닙니다")]
    InvalidDigit { index: usize, ch: char },

    /// 부르는 쪽이 준 출력 버퍼가 작습니다.
    #[error("출력 버퍼가 작습니다({needed}바이트가 필요한데 {available}바이트입니다)")]
    BufferTooSmall { needed: usize, available: usize },

    /// 결과를 담을 메모리를 확보하지 못했습니다(`try_` 함수만 돌려줌).
    #[cfg(feature = "alloc")]
    #[error("결과를 담을 메모리를 확보하지 못했습니다")]
    OutOfMemory(#[source] alloc::collections::TryReserveError),
}
