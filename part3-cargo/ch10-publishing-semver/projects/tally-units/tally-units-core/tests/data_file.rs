//! 큰 데이터 파일로 변환을 확인합니다. 데이터 파일은 게시하는 .crate에는 넣지 않습니다(exclude).

use tally_units_core::{Unit, convert};

#[test]
fn converts_every_line() {
    let text = std::fs::read_to_string("tests/data/many.txt").unwrap();
    let total: f64 = text
        .lines()
        .map(|l| l.trim_end_matches(" km").parse::<f64>().unwrap())
        .map(|v| convert(v, Unit::Kilometer, Unit::Meter))
        .sum();
    assert_eq!(total, (0..200).sum::<u32>() as f64 * 1000.0);
}
