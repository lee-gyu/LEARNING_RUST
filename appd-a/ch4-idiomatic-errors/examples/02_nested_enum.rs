// 지침 2: 호출한 쪽이 실패 종류에 따라 다르게 대응해야 한다면, 실패 종류를 배리언트로 가진
// 에러 열거형을 만들고 아래 계층의 에러는 배리언트 안에 품습니다(중첩 에러).
//
//   1) 비권장(String)과 권장(열거형)으로 같은 주문 줄 파서를 만들고, 같은 처리 방침으로 결과를 비교합니다.
//      String 판은 메시지에 든 낱말로 종류를 짐작하므로, 입력이 메시지에 섞이면 틀린 결정을 내립니다.
//   2) 열거형은 아래 계층의 에러(Utf8Error, ParseIntError)를 source()로 내줍니다.
//
// 실행: cargo run -p er-ch4-idiomatic-errors --example 02_nested_enum

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::{self, Utf8Error};

const MAX_QTY: u32 = 99;

// 주문 줄 "상품코드:수량"을 바이트로 받아(네트워크나 파일에서 왔다고 생각합니다) 읽은 결과입니다.
#[derive(Debug)]
struct OrderLine {
    sku: String,
    qty: u32,
}

// ── 비권장: 실패를 모두 String 메시지로 ────────────────────────────────
fn parse_line_str(raw: &[u8]) -> Result<OrderLine, String> {
    let text = str::from_utf8(raw).map_err(|e| format!("주문 줄이 UTF-8이 아닙니다: {e}"))?;
    // 메시지에 입력을 넣어 두는 것은 사람에게는 친절한 습관입니다. 1번 구역의 문제는 여기서 생깁니다.
    let (sku, qty) = text
        .split_once(':')
        .ok_or_else(|| format!("':' 구분자가 없습니다: {text:?}"))?;
    let qty: u32 = qty
        .trim()
        .parse()
        .map_err(|e| format!("수량을 읽지 못했습니다: {e}"))?;
    if qty > MAX_QTY {
        return Err(format!("수량 {qty}이(가) 최대 {MAX_QTY}를 넘습니다"));
    }
    Ok(OrderLine {
        sku: sku.to_string(),
        qty,
    })
}

// ── 권장: 실패 종류마다 배리언트, 아래 계층의 에러는 배리언트 안에 ──────
#[derive(Debug)]
enum OrderError {
    Encoding(Utf8Error),                  // 아래 계층: 바이트 → 문자열
    MissingColon { line: String },        // 이 계층이 판단한 실패
    BadQty(ParseIntError),                // 아래 계층: 문자열 → 숫자
    TooMany { requested: u32, max: u32 }, // 이 계층이 판단한 실패, 대응에 필요한 값을 담음
}

// Display에는 이 계층의 사정만 씁니다. 아래 계층의 메시지는 source()가 맡습니다(Part 1 ch8 2.3절).
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

// 아래 계층의 에러 타입마다 From을 구현하면 본문에서 map_err가 사라집니다.
// 이 두 impl은 OrderError를 정의한 크레이트만 쓸 수 있습니다(고아 규칙).
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

fn parse_line(raw: &[u8]) -> Result<OrderLine, OrderError> {
    let text = str::from_utf8(raw)?; // Utf8Error → OrderError::Encoding
    let (sku, qty) = text
        .split_once(':')
        .ok_or_else(|| OrderError::MissingColon {
            line: text.to_string(),
        })?;
    let qty: u32 = qty.trim().parse()?; // ParseIntError → OrderError::BadQty
    if qty > MAX_QTY {
        return Err(OrderError::TooMany {
            requested: qty,
            max: MAX_QTY,
        });
    }
    Ok(OrderLine {
        sku: sku.to_string(),
        qty,
    })
}

// ── 호출한 쪽의 처리 방침 ──────────────────────────────────────────────
// 성공하면 접수, 수량만 넘친 줄은 담당자 확인을 위해 보류, 나머지는 버립니다.

fn decide_str(raw: &[u8]) -> String {
    match parse_line_str(raw) {
        Ok(line) => format!("접수 {} x{}", line.sku, line.qty),
        // 종류를 알 방법이 메시지뿐이므로 낱말로 짐작합니다.
        Err(msg) if msg.contains("최대") => "보류".to_string(),
        Err(_) => "버림".to_string(),
    }
}

fn decide(raw: &[u8]) -> String {
    match parse_line(raw) {
        Ok(line) => format!("접수 {} x{}", line.sku, line.qty),
        Err(OrderError::TooMany { requested, max }) => format!("보류 ({requested} > {max})"),
        // `_`로 뭉뚱그리지 않고 배리언트를 모두 적으면, 배리언트가 늘었을 때 이 match가
        // 컴파일 에러(E0004)로 알려 줍니다.
        Err(OrderError::Encoding(_) | OrderError::MissingColon { .. } | OrderError::BadQty(_)) => {
            "버림".to_string()
        }
    }
}

fn main() {
    let inputs: [&[u8]; 5] = [
        b"KR-102:3",
        b"KR-205:120",
        b"KR-310:x",
        "최대한 빨리 보내 주세요".as_bytes(), // 주문 목록에 섞여 들어온 메모 줄
        b"KR-4\xff2:1",                       // UTF-8이 아닌 바이트 0xFF
    ];

    println!("1. 같은 처리 방침, 두 가지 에러 타입");
    for raw in inputs {
        let shown = String::from_utf8_lossy(raw);
        println!("    {shown}");
        println!("      String 판: {}", decide_str(raw));
        println!("      열거형 판: {}", decide(raw));
    }
    // 출력: 나머지 줄은 두 판이 같고, "최대한 빨리 보내 주세요" 줄에서만 갈립니다.
    //   String 판: 보류 ← 구분자가 없다는 메시지에 입력 "최대한…"이 들어 있어서 수량 초과로 잘못 짐작
    //   열거형 판: 버림 ← MissingColon 배리언트로 정확히 구분

    println!("2. 메시지만 보면 무엇이 문제였나");
    for raw in inputs {
        if let Err(msg) = parse_line_str(raw) {
            println!("    String 판: {msg}");
        }
    }
    // 출력(세 번째 줄): String 판: ':' 구분자가 없습니다: "최대한 빨리 보내 주세요"  ← "최대"가 들어 있음

    println!("3. 열거형은 아래 계층의 에러를 source()로 내줍니다");
    for raw in inputs {
        if let Err(e) = parse_line(raw) {
            println!("    {e:?}");
            println!("      Display: {e}");
            match e.source() {
                Some(cause) => println!("      원인:    {cause}"),
                None => println!("      원인:    없음 (이 계층이 판단한 실패)"),
            }
        }
    }
    // 출력(예): BadQty(ParseIntError { kind: InvalidDigit })
    //             Display: 수량을 읽지 못했습니다
    //             원인:    invalid digit found in string
}
