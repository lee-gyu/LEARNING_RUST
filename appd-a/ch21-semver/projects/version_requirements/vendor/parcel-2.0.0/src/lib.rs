//! parcel 2.0.0 (연습용 레지스트리의 가짜 크레이트)
//!
//! 2.0.0에서 무게를 단위가 드러나는 타입 `Grams`로 받도록 `fee`의 시그니처를 바꿨습니다.
//! 호환되지 않는 변경이므로 메이저 버전을 올렸습니다.

pub const VERSION: &str = "2.0.0";

/// 그램 단위 무게
pub struct Grams(pub u32);

/// 배송비(원): 기본 2500원에 1kg마다 500원을 더합니다.
pub fn fee(weight: Grams) -> u32 {
    2500 + weight.0 / 1000 * 500
}

/// 물품 가액의 1%를 보험료로 더한 배송비(원)
pub fn fee_with_insurance(weight: Grams, value_won: u32) -> u32 {
    fee(weight) + value_won / 100
}
