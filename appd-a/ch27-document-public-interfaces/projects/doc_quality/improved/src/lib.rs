//! `키 = 값` 줄로 된 작은 설정 파일을 읽습니다.
//!
//! 표준 라이브러리만 쓰고, 형식도 일부러 작게 잡았습니다. 섹션(`[server]`), 따옴표,
//! 줄 끝 주석, 여러 줄에 걸친 값은 지원하지 않습니다. 그런 형식이 필요하면 TOML 파서를
//! 쓰는 편이 낫습니다.
//!
//! # 형식
//!
//! - 한 줄에 `키 = 값`을 하나 씁니다. 첫 `=`에서 나누므로 값에는 `=`가 들어가도 됩니다.
//! - 키와 값의 앞뒤 공백은 지웁니다. 값은 비어도 되지만 키는 비면 안 됩니다.
//! - 빈 줄과, 앞 공백을 지운 뒤 `#`으로 시작하는 줄은 건너뜁니다. 줄 중간의 `#`은 값의 일부입니다.
//! - 같은 키는 한 번만 쓸 수 있습니다.
//!
//! # Examples
//!
//! ```
//! let config = kvconf::parse(
//!     "# 서버 설정
//!      host = 127.0.0.1
//!      port = 8080
//!      motd = 어서 오세요 # 줄 끝 주석이 아닙니다",
//! )?;
//!
//! assert_eq!(config.get("port"), Some("8080"));
//! assert_eq!(config.get("motd"), Some("어서 오세요 # 줄 끝 주석이 아닙니다"));
//! assert_eq!(config.get("timeout"), None);
//! assert_eq!(config.keys().collect::<Vec<_>>(), ["host", "port", "motd"]);
//! # Ok::<(), kvconf::ParseError>(())
//! ```
//!
//! 잘못된 줄이 있으면 [`parse`]가 그 줄의 번호를 담은 [`ParseError`]를 돌려줍니다.
//! 읽은 값을 꺼내는 방법은 [`Config`]에 있습니다.

// 공개 항목에 문서가 없으면 경고합니다(rustc 린트, 기본 allow).
#![warn(missing_docs)]
// 문서의 항목 링크가 깨지면 `cargo doc`을 실패시킵니다(rustdoc 린트, 기본 warn).
#![deny(rustdoc::broken_intra_doc_links)]
// `Result`를 돌려주는데 `# Errors`가, 패닉하는데 `# Panics`가 없으면 경고합니다(clippy pedantic).
#![warn(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod config;
mod parse;

// 하위 모듈은 비공개로 두고 쓰는 사람이 적을 이름만 올립니다. rustdoc은 비공개 모듈에서
// 올린 항목을 크레이트 루트에 정의된 것처럼 보여 줍니다.
pub use config::Config;
pub use parse::{ParseError, parse};
