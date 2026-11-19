//! 2021 에디션 라이브러리입니다. 문서 테스트가 에디션에 따라 어떻게 컴파일되고 실행되는지
//! e2024와 비교합니다(cargo test --doc -p e2021). e2024/src/lib.rs와 내용이 같고, 다른 곳은
//! 크레이트 이름, rustfmt 서식(greeting의 if), 그리고 2024 에디션 때문에 고친 두 곳입니다.
//!
//! 2021 에디션에서는 문서 테스트마다 따로 컴파일하므로, 아래 코드 블록은 표시 없이도 언제나
//! 같은 줄 번호에서 실행됩니다.
//!
//! ```
//! let here = std::panic::Location::caller();
//! assert_eq!(here.line(), 3);
//! ```
//!
//! USAGE.md 안의 include_str! 경로는 2021 에디션에서 이 파일(src/lib.rs)을 기준으로 읽습니다.

use std::sync::atomic::{AtomicUsize, Ordering};

static ISSUED: AtomicUsize = AtomicUsize::new(0);

/// 번호표를 하나 발급하고, 이 프로세스에서 그전에 발급한 개수를 돌려줍니다.
///
/// ```
/// assert_eq!(e2021::issue_ticket(), 0);
/// ```
///
/// 다른 문서 테스트가 발급한 번호표는 세지 않습니다. 문서 테스트마다 따로 컴파일하고 따로
/// 실행하기 때문입니다.
///
/// ```
/// assert_eq!(e2021::issue_ticket(), 0);
/// assert_eq!(e2021::issue_ticket(), 1);
/// ```
pub fn issue_ticket() -> usize {
    ISSUED.fetch_add(1, Ordering::SeqCst)
}

// 문서를 크레이트 루트의 USAGE.md에서 가져옵니다.
#[doc = include_str!("../USAGE.md")]
pub fn greeting(formal: bool) -> &'static str {
    if formal {
        "안녕하세요"
    } else {
        "안녕"
    }
}
