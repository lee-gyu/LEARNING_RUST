use ledger_core::Entry;
use ledger_report::summary;

fn main() {
    let entries = [
        Entry {
            memo: "월급".to_string(),
            amount: 2_500_000,
        },
        Entry {
            memo: "월세".to_string(),
            amount: -700_000,
        },
    ];
    for e in &entries {
        println!("{}: {}원", e.memo, e.amount);
    }
    println!("{}", summary(&entries));
}
