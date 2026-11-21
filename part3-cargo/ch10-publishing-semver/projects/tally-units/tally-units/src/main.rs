//! 길이를 바꿉니다. 사용법: tally-units <값> <원래 단위> <바꿀 단위>
//!
//! 실행: cargo run -q -p tally-units -- 2.5 km m

use std::process::ExitCode;
use tally_units_core::{Unit, convert};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [value, from, to] = args.as_slice() else {
        eprintln!("사용법: tally-units <값> <원래 단위> <바꿀 단위>");
        return ExitCode::from(2);
    };
    let (Ok(value), Some(from), Some(to)) =
        (value.parse::<f64>(), Unit::parse(from), Unit::parse(to))
    else {
        eprintln!("값은 수, 단위는 mm·cm·m·km 가운데 하나여야 합니다");
        return ExitCode::from(2);
    };
    println!("{}", convert(value, from, to));
    ExitCode::SUCCESS
}
