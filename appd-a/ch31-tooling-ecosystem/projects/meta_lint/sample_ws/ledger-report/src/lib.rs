//! 가계부 항목을 한 줄 요약으로 만듭니다.

use ledger_core::{Entry, balance};

/// 항목 수와 잔액을 한 줄로 요약합니다.
pub fn summary(entries: &[Entry]) -> String {
    format!("항목 {}개, 잔액 {}원", entries.len(), balance(entries))
}
