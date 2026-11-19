//! parcel 1.6.0 (연습용 레지스트리의 가짜 크레이트)

pub const VERSION: &str = "1.6.0";

/// 배송비(원): 기본 2500원에 1kg마다 500원을 더합니다.
pub fn fee(weight_g: u32) -> u32 {
    2500 + weight_g / 1000 * 500
}

/// 1.6.0에서 더한 함수: 물품 가액의 1%를 보험료로 더한 배송비(원)
pub fn fee_with_insurance(weight_g: u32, value_won: u32) -> u32 {
    fee(weight_g) + value_won / 100
}
