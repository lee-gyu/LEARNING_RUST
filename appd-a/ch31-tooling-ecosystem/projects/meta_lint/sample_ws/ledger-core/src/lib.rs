//! 가계부 항목과 잔액을 계산합니다.

/// 가계부 항목 하나입니다. 금액은 원 단위이고, 지출은 음수입니다.
pub struct Entry {
    /// 항목 설명
    pub memo: String,
    /// 금액(원)
    pub amount: i64,
}

/// 항목들의 금액을 모두 더한 잔액을 돌려줍니다.
pub fn balance(entries: &[Entry]) -> i64 {
    entries.iter().map(|e| e.amount).sum()
}
