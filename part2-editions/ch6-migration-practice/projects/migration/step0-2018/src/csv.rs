//! CSV 내보내기. 기능 `csv`를 켰을 때만 컴파일됩니다.

use std::io;

use crate::book::Ledger;

/// 항목마다 `카테고리,금액,메모` 한 줄씩 씁니다.
// 2018 에디션 시절의 `dyn` 없는 트레이트 객체입니다(경고를 막아 둠).
#[allow(bare_trait_objects)]
pub fn write_csv(book: &Ledger, out: &mut io::Write) -> io::Result<()> {
    writeln!(out, "category,amount,memo")?;
    for e in book.entries() {
        writeln!(out, "{},{},{}", e.category, e.amount, e.memo)?;
    }
    Ok(())
}
