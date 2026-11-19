// 지침 4: 라이브러리의 에러 열거형은 thiserror로 만들면 반복을 덜 수 있습니다.
// thiserror는 손으로 쓰던 Display, Error(source), From 구현을 대신 써 줄 뿐이라서,
// 만들어지는 것은 표준 트레이트의 구현이고 공개 API에 thiserror의 타입이 드러나지 않습니다.
//
//   1) 02의 OrderError를 손으로 쓴 판과 thiserror 판으로 나란히 두고 Display, source, Debug를 비교합니다.
//   2) #[from]이 만든 From 덕분에 두 판의 parse_line 본문이 글자 하나 다르지 않습니다.
//   3) 구조체에도 쓸 수 있고, #[source]는 From 없이 원인만 지정합니다.
//
// 실행: cargo run -p er-ch4-idiomatic-errors --example 04_thiserror
// 의존성: thiserror 2.0.21 ([dev-dependencies])

use std::error::Error;

const MAX_QTY: u32 = 99;

// ── 손으로 쓴 판 (02_nested_enum.rs와 같음) ────────────────────────────
mod by_hand {
    use super::MAX_QTY;
    use std::error::Error;
    use std::fmt;
    use std::num::ParseIntError;
    use std::str::{self, Utf8Error};

    #[derive(Debug)]
    pub enum OrderError {
        Encoding(Utf8Error),
        MissingColon { line: String },
        BadQty(ParseIntError),
        TooMany { requested: u32, max: u32 },
    }

    impl fmt::Display for OrderError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Encoding(_) => write!(f, "주문 줄이 UTF-8이 아닙니다"),
                Self::MissingColon { line } => write!(f, "':' 구분자가 없습니다: {line:?}"),
                Self::BadQty(_) => write!(f, "수량을 읽지 못했습니다"),
                Self::TooMany { requested, max } => {
                    write!(f, "수량 {requested}이(가) 최대 {max}를 넘습니다")
                }
            }
        }
    }

    impl Error for OrderError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::Encoding(e) => Some(e),
                Self::BadQty(e) => Some(e),
                Self::MissingColon { .. } | Self::TooMany { .. } => None,
            }
        }
    }

    impl From<Utf8Error> for OrderError {
        fn from(e: Utf8Error) -> Self {
            Self::Encoding(e)
        }
    }

    impl From<ParseIntError> for OrderError {
        fn from(e: ParseIntError) -> Self {
            Self::BadQty(e)
        }
    }

    pub fn parse_line(raw: &[u8]) -> Result<(String, u32), OrderError> {
        let text = str::from_utf8(raw)?;
        let (sku, qty) = text
            .split_once(':')
            .ok_or_else(|| OrderError::MissingColon {
                line: text.to_string(),
            })?;
        let qty: u32 = qty.trim().parse()?;
        if qty > MAX_QTY {
            return Err(OrderError::TooMany {
                requested: qty,
                max: MAX_QTY,
            });
        }
        Ok((sku.to_string(), qty))
    }
}

// ── thiserror 판 ───────────────────────────────────────────────────────
mod with_thiserror {
    use super::MAX_QTY;
    use std::num::ParseIntError;
    use std::str::{self, Utf8Error};

    // #[error("..")]  → 그 배리언트의 Display. 필드는 {line}, {0}처럼 이름이나 번호로 씁니다.
    // #[from]         → 그 필드 타입에서 오는 From 구현 + source()가 그 필드를 돌려줌
    #[derive(Debug, thiserror::Error)]
    pub enum OrderError {
        #[error("주문 줄이 UTF-8이 아닙니다")]
        Encoding(#[from] Utf8Error),
        #[error("':' 구분자가 없습니다: {line:?}")]
        MissingColon { line: String },
        #[error("수량을 읽지 못했습니다")]
        BadQty(#[from] ParseIntError),
        #[error("수량 {requested}이(가) 최대 {max}를 넘습니다")]
        TooMany { requested: u32, max: u32 },
    }

    // 본문은 by_hand::parse_line과 똑같습니다.
    pub fn parse_line(raw: &[u8]) -> Result<(String, u32), OrderError> {
        let text = str::from_utf8(raw)?;
        let (sku, qty) = text
            .split_once(':')
            .ok_or_else(|| OrderError::MissingColon {
                line: text.to_string(),
            })?;
        let qty: u32 = qty.trim().parse()?;
        if qty > MAX_QTY {
            return Err(OrderError::TooMany {
                requested: qty,
                max: MAX_QTY,
            });
        }
        Ok((sku.to_string(), qty))
    }

    // 구조체에도 씁니다. #[source]는 "이 필드가 원인"이라는 표시만 하고 From은 만들지 않습니다.
    // 줄 번호처럼 원인 에러만으로는 채울 수 없는 필드가 있을 때 씁니다.
    #[derive(Debug, thiserror::Error)]
    #[error("{line_no}번째 줄을 처리하지 못했습니다")]
    pub struct LineError {
        pub line_no: usize,
        #[source]
        pub cause: OrderError,
    }

    pub fn parse_sheet(lines: &[&[u8]]) -> Result<u32, LineError> {
        let mut total = 0;
        for (i, raw) in lines.iter().enumerate() {
            let (_, qty) = parse_line(raw).map_err(|cause| LineError {
                line_no: i + 1,
                cause,
            })?;
            total += qty;
        }
        Ok(total)
    }
}

// 에러 하나를 "Display / source의 Display / Debug" 세 문자열로 펼칩니다.
fn describe<E: Error>(e: &E) -> [String; 3] {
    let source = match e.source() {
        Some(s) => s.to_string(),
        None => "없음".to_string(),
    };
    [e.to_string(), source, format!("{e:?}")]
}

fn main() {
    let inputs: [&[u8]; 5] = [
        b"KR-102:3",
        b"KR-4\xff2:1",
        b"KR-102",
        b"KR-310:x",
        b"KR-205:120",
    ];

    println!("1. 손으로 쓴 판과 thiserror 판 비교");
    for raw in inputs {
        let shown = String::from_utf8_lossy(raw);
        match (by_hand::parse_line(raw), with_thiserror::parse_line(raw)) {
            (Ok(a), Ok(b)) => println!("    {shown:?}: 둘 다 Ok, 결과 같음 = {}", a == b),
            (Err(a), Err(b)) => {
                let [a_display, a_source, a_debug] = describe(&a);
                let [b_display, b_source, b_debug] = describe(&b);
                println!(
                    "    {shown:?}: Display 같음 = {}, source 같음 = {}, Debug 같음 = {}",
                    a_display == b_display,
                    a_source == b_source,
                    a_debug == b_debug
                );
                println!("        Display: {b_display}");
                println!("        source:  {b_source}");
                println!("        Debug:   {b_debug}");
            }
            _ => println!("    {shown:?}: 두 판의 결과가 다릅니다"),
        }
    }
    // 출력: 네 가지 에러 모두 "Display 같음 = true, source 같음 = true, Debug 같음 = true"

    println!("2. #[from]이 만든 From");
    let parse_err = "x".parse::<u32>().unwrap_err();
    let a = by_hand::OrderError::from(parse_err.clone());
    let b = with_thiserror::OrderError::from(parse_err);
    println!("    손으로: {a:?}");
    println!("    thiserror: {b:?}");
    // 출력: 둘 다 BadQty(ParseIntError { kind: InvalidDigit })

    println!("3. 구조체와 #[source]");
    let sheet: [&[u8]; 3] = [b"KR-102:3", b"US-310:4", b"KR-205:x"];
    match with_thiserror::parse_sheet(&sheet) {
        Ok(total) => println!("    합계 {total}개"),
        Err(e) => {
            println!("    에러: {e} (line_no = {})", e.line_no);
            let mut cause = e.source();
            while let Some(c) = cause {
                println!("      원인: {c}");
                cause = c.source();
            }
            // cause 필드는 공개되어 있으므로 match로 종류를 구분할 수도 있습니다.
            let is_qty = matches!(e.cause, with_thiserror::OrderError::BadQty(_));
            println!("    원인이 BadQty 배리언트인가: {is_qty}");
            // 출력: 에러: 3번째 줄을 처리하지 못했습니다 (line_no = 3)
            //         원인: 수량을 읽지 못했습니다
            //         원인: invalid digit found in string
            //       원인이 BadQty 배리언트인가: true
        }
    }
}
