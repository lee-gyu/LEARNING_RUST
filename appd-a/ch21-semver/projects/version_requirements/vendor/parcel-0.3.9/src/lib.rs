//! parcel 0.3.9 (연습용 레지스트리의 가짜 크레이트)

pub const VERSION: &str = "0.3.9";

/// 배송비(원): 기본 2500원에 1kg마다 500원을 더합니다.
pub fn fee(weight_g: u32) -> u32 {
    2500 + weight_g / 1000 * 500
}
