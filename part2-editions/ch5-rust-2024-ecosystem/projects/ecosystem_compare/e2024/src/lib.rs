//! 2024 에디션 라이브러리입니다. 문서 테스트가 에디션에 따라 어떻게 컴파일되고 실행되는지
//! e2021과 비교합니다(cargo test --doc -p e2024). e2021/src/lib.rs와 내용이 같고, 다른 곳은
//! 크레이트 이름, rustfmt 서식(greeting의 if), 그리고 2024 에디션 때문에 고친 두 곳입니다.
//!
//! 고친 곳 1: 아래 코드 블록에 `standalone_crate`를 붙였습니다. 2024 에디션은 문서 테스트들을
//! 한 바이너리로 묶어 컴파일하므로, 코드가 몇 번째 줄에 놓이는지에 기대는 테스트는 묶이면
//! 실패합니다. 이 표시가 있으면 2021 에디션처럼 따로 컴파일합니다.
//!
//! ```standalone_crate
//! let here = std::panic::Location::caller();
//! assert_eq!(here.line(), 3);
//! ```
//!
//! 고친 곳 2: USAGE.md 안의 include_str! 경로를 USAGE.md 기준으로 바꾸고, 그 코드 블록에도
//! `standalone_crate`를 붙였습니다(greeting의 문서).

use std::sync::atomic::{AtomicUsize, Ordering};

static ISSUED: AtomicUsize = AtomicUsize::new(0);

/// 번호표를 하나 발급하고, 이 프로세스에서 그전에 발급한 개수를 돌려줍니다.
///
/// ```
/// assert_eq!(e2024::issue_ticket(), 0);
/// ```
///
/// 다른 문서 테스트가 발급한 번호표는 세지 않습니다. 문서 테스트를 한 바이너리로 묶어 컴파일해도
/// 실행은 테스트마다 따로 하기 때문입니다.
///
/// ```
/// assert_eq!(e2024::issue_ticket(), 0);
/// assert_eq!(e2024::issue_ticket(), 1);
/// ```
pub fn issue_ticket() -> usize {
    ISSUED.fetch_add(1, Ordering::SeqCst)
}

// 문서를 크레이트 루트의 USAGE.md에서 가져옵니다. 그 안의 문서 테스트가 쓰는 include_str!의
// 경로는 2024 에디션에서 USAGE.md를 기준으로 읽습니다.
#[doc = include_str!("../USAGE.md")]
pub fn greeting(formal: bool) -> &'static str {
    if formal { "안녕하세요" } else { "안녕" }
}
