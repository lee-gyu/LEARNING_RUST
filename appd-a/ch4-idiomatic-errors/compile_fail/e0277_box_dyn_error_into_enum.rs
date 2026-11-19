// 기대하는 에러: E0277 — `?` couldn't convert the error to `StockError`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_box_dyn_error_into_enum.rs --out-dir /tmp/rust-er-ch4
//
// 저장소 계층(read_stock)은 Box<dyn Error>를 돌려주고, 그 위의 reserve는 구체 타입 StockError를 돌려줍니다.
// StockError에는 From<ParseIntError>가 있으니 ?가 될 것 같지만, ?가 받는 것은 ParseIntError가 아니라
// 종류가 지워진 Box<dyn Error>입니다. 지워진 종류를 ?가 되살릴 수는 없습니다.
// 해결: (1) read_stock이 구체 타입을 돌려주게 바꿉니다.
//       (2) StockError에 Box를 담는 배리언트와 From<Box<..>>를 둡니다(examples/03_trait_object.rs의 3번 구역).
//       (3) 이 자리에서 downcast로 직접 꺼냅니다: e.downcast::<ParseIntError>()

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

fn read_stock(record: &str) -> Result<u32, Box<dyn Error>> {
    Ok(record.trim().parse::<u32>()?)
}

#[derive(Debug)]
enum StockError {
    BadRecord(ParseIntError),
    OutOfStock { have: u32, want: u32 },
}

impl fmt::Display for StockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRecord(_) => write!(f, "재고 기록을 읽지 못했습니다"),
            Self::OutOfStock { have, want } => write!(f, "재고가 부족합니다 ({have} < {want})"),
        }
    }
}

impl Error for StockError {}

impl From<ParseIntError> for StockError {
    fn from(e: ParseIntError) -> Self {
        Self::BadRecord(e)
    }
}

fn reserve(record: &str, want: u32) -> Result<u32, StockError> {
    let have = read_stock(record)?;
    if have < want {
        return Err(StockError::OutOfStock { have, want });
    }
    Ok(have - want)
}

fn main() {
    println!("{:?}", reserve("12", 5));
}
