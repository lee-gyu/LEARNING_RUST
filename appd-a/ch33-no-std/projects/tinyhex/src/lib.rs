//! 바이트 열과 16진수 문자열을 오가는 작은 `no_std` 라이브러리입니다.
//!
//! 쓸 수 있는 API는 켠 기능에 따라 늘어납니다. 기능은 API를 더하기만 합니다.
//!
//! - 기능 없음(`core`만): [`HexError`], [`Hex`], [`decode_to_slice`]
//! - `alloc`: `encode`, `decode`, `try_encode`, `try_decode`
//! - `std`(기본): `write_hex`, `read_hex`, `HexError`에서 `std::io::Error`로의 변환

// 크레이트 루트는 어느 기능 조합에서나 `no_std`입니다. 그래서 모든 빌드가 core의 프렐류드를 쓰고,
// alloc·std의 이름은 어느 빌드에서나 직접 가져와야 합니다(compile_fail/e0425_std_feature_prelude.rs 참고).
#![no_std]

// alloc 기능이 켜졌을 때만 alloc 크레이트를 연결합니다.
#[cfg(feature = "alloc")]
extern crate alloc;

// std 기능이 켜졌을 때와 단위 테스트를 빌드할 때만 std를 연결합니다. 테스트 하네스 자체는 std를
// 스스로 연결하므로, 이 줄은 테스트 코드가 `std::format!` 같은 이름을 쓰기 위한 것입니다.
#[cfg(any(feature = "std", test))]
extern crate std;

mod error;
mod slice;

#[cfg(feature = "alloc")]
mod owned;

#[cfg(feature = "std")]
mod io;

pub use error::HexError;
pub use slice::{Hex, decode_to_slice};

#[cfg(feature = "alloc")]
pub use owned::{decode, encode, try_decode, try_encode};

#[cfg(feature = "std")]
pub use io::{read_hex, write_hex};
