//! CSV 내보내기. 기능 `csv`를 켰을 때만 컴파일됩니다.

use std::io;

use crate::book::Ledger;

/// 항목마다 `카테고리,금액,메모` 한 줄씩 씁니다.
pub fn write_csv(book: &Ledger, out: &mut dyn io::Write) -> io::Result<()> {
    writeln!(out, "category,amount,memo")?;
    for e in book.entries() {
        writeln!(out, "{},{},{}", e.category, e.amount, e.memo)?;
    }
    Ok(())
}
