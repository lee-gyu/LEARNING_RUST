//! 기대하는 에러: E0453 — expect(missing_docs) incompatible with previous forbid
//!
//! 확인: rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0453_expect_under_forbid.rs
//!
//! "공개 API에는 반드시 문서가 있어야 한다"는 정책을 `forbid`로 걸었습니다. `forbid`는 크레이트
//! 안의 어디에서도 수준을 낮출 수 없으므로, 이유를 적은 `#[expect]`도 거부됩니다.
//! 예외가 있을 수 있는 린트는 `deny`로 겁니다(README 4.1절).
#![forbid(missing_docs)]

/// 주문 금액에 부가세 10%를 더합니다.
pub fn with_tax(price: u64) -> u64 {
    price + price / 10
}

/// 테스트에서만 쓰는 도우미 모음입니다.
pub mod testing {
    #[expect(missing_docs, reason = "테스트 전용 도우미라 문서를 쓰지 않음")] // E0453
    pub fn sample_price() -> u64 {
        12_000
    }
}
