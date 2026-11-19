// 지침 3: Box<dyn Error>로 에러의 종류를 지우는 방법은 "호출한 쪽이 종류를 보지 않을 때"에 씁니다.
// 얻는 것과 잃는 것을 함께 봅니다.
//
//   1) 얻는 것: 타입을 하나도 정의하지 않고 여러 종류의 에러를 ?로 담습니다.
//   2) 잃는 것: 종류를 알려면 다운캐스트해야 하고, 라이브러리가 에러를 한 겹 감싸기만 해도
//      호출한 쪽의 다운캐스트가 컴파일 에러 없이 빗나갑니다(비권장). source 체인을 따라가며
//      찾으면 버틸 수 있습니다.
//   3) 절충(권장): 호출한 쪽이 대응할 실패만 배리언트로 두고, 나머지는 Box<dyn Error + Send + Sync>
//      배리언트 하나에 담아 source 체인으로 내줍니다.
//
// 실행: cargo run -p er-ch4-idiomatic-errors --example 03_trait_object

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::{self, Utf8Error};

const MAX_QTY: u32 = 99;

// ── 1. 정의 없이 무엇이든 담기 ─────────────────────────────────────────
// 02의 parse_line과 같은 일을 하지만 에러 타입을 하나도 만들지 않았습니다.
fn parse_line_boxed(raw: &[u8]) -> Result<(String, u32), Box<dyn Error>> {
    let text = str::from_utf8(raw)?; // Utf8Error를 그대로 담음
    let (sku, qty) = text.split_once(':').ok_or("':' 구분자가 없습니다")?; // &str → 비공개 타입
    let qty: u32 = qty.trim().parse()?; // ParseIntError를 그대로 담음
    if qty > MAX_QTY {
        return Err(format!("수량 {qty}이(가) 최대 {MAX_QTY}를 넘습니다").into()); // String → 비공개 타입
    }
    Ok((sku.to_string(), qty))
}

// 담긴 것이 무엇인지는 실행 중에 하나씩 물어봐야 압니다.
fn inner_type(e: &(dyn Error + 'static)) -> &'static str {
    if e.is::<Utf8Error>() {
        "Utf8Error"
    } else if e.is::<ParseIntError>() {
        "ParseIntError"
    } else {
        "알 수 없음 (문자열에서 만든 에러)"
    }
}

// ── 2. 리팩터링에 약한 다운캐스트 ─────────────────────────────────────
// 같은 라이브러리의 두 버전입니다. 시그니처는 똑같이 Box<dyn Error>를 돌려줍니다.
mod qty_v1 {
    use std::error::Error;

    pub fn parse_qty(text: &str) -> Result<u32, Box<dyn Error>> {
        Ok(text.trim().parse::<u32>()?) // ParseIntError를 그대로 상자에 담습니다.
    }
}

mod qty_v2 {
    use std::error::Error;
    use std::fmt;
    use std::num::ParseIntError;

    // v2는 메시지에 입력을 넣으려고 ParseIntError를 자기 에러로 한 겹 감쌌습니다. 좋은 개선입니다.
    #[derive(Debug)]
    pub struct QtyError {
        text: String,
        source: ParseIntError,
    }

    impl fmt::Display for QtyError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "수량 {:?}을(를) 읽지 못했습니다", self.text)
        }
    }

    impl Error for QtyError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.source)
        }
    }

    pub fn parse_qty(text: &str) -> Result<u32, Box<dyn Error>> {
        let qty = text.trim().parse::<u32>().map_err(|source| QtyError {
            text: text.to_string(),
            source,
        })?;
        Ok(qty)
    }
}

// 비권장: 맨 바깥 에러만 다운캐스트합니다.
fn react_outer(result: &Result<u32, Box<dyn Error>>) -> String {
    match result {
        Ok(n) => format!("{n}개"),
        Err(e) if e.is::<ParseIntError>() => "형식 오류 → 0개로 처리".to_string(),
        Err(e) => format!("모르는 에러 → 작업 중단 ({e})"),
    }
}

// source 체인을 따라 내려가며 T를 찾습니다. 감싸는 층이 늘어도 찾을 수 있습니다.
fn find_cause<'a, T: Error + 'static>(err: &'a (dyn Error + 'static)) -> Option<&'a T> {
    let mut current = Some(err);
    while let Some(e) = current {
        if let Some(found) = e.downcast_ref::<T>() {
            return Some(found);
        }
        current = e.source();
    }
    None
}

fn react_chain(result: &Result<u32, Box<dyn Error>>) -> String {
    match result {
        Ok(n) => format!("{n}개"),
        Err(e) => match find_cause::<ParseIntError>(e.as_ref()) {
            Some(p) => format!("형식 오류({:?}) → 0개로 처리", p.kind()),
            None => format!("모르는 에러 → 작업 중단 ({e})"),
        },
    }
}

// ── 3. 절충: 대응할 실패는 배리언트로, 나머지는 Box로 ───────────────────
// 재고 기록을 읽는 "저장소" 계층입니다. 어떤 에러가 날지는 저장소 구현에 따라 달라지고,
// 호출한 쪽은 그 종류에 관심이 없다고 가정합니다.
fn read_stock(record: &[u8]) -> Result<u32, Box<dyn Error + Send + Sync>> {
    Ok(str::from_utf8(record)?.trim().parse()?)
}

#[derive(Debug)]
enum StockError {
    // 호출한 쪽이 대응하는 실패: 필요한 값을 담은 배리언트
    OutOfStock { sku: String, have: u32, want: u32 },
    // 대응하지 않고 보고만 하는 실패: 종류를 지워 한곳에
    Storage(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for StockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfStock { sku, have, want } => {
                write!(f, "{sku} 재고가 부족합니다 ({have}개 있음, {want}개 필요)")
            }
            Self::Storage(_) => write!(f, "재고 기록을 읽지 못했습니다"),
        }
    }
}

impl Error for StockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OutOfStock { .. } => None,
            // Box 안의 에러를 빌려 내줍니다. Send + Sync는 떼어 내도 되는 표시라 그대로 변환됩니다.
            Self::Storage(e) => Some(e.as_ref()),
        }
    }
}

// 포괄 구현 `impl<E: Error> From<E>`는 E0119로 막히지만(Part 1 ch8 4.2절),
// Box 한 타입에서 오는 From은 문제없습니다. 덕분에 read_stock(..)?가 됩니다.
impl From<Box<dyn Error + Send + Sync>> for StockError {
    fn from(e: Box<dyn Error + Send + Sync>) -> Self {
        Self::Storage(e)
    }
}

fn reserve(sku: &str, record: &[u8], want: u32) -> Result<u32, StockError> {
    let have = read_stock(record)?;
    if have < want {
        return Err(StockError::OutOfStock {
            sku: sku.to_string(),
            have,
            want,
        });
    }
    Ok(have - want)
}

fn main() {
    println!("1. Box<dyn Error>는 정의 없이 무엇이든 담습니다");
    let inputs: [&[u8]; 4] = [b"KR-102:3", b"KR-4\xff2:1", b"KR-310:x", b"KR-205:120"];
    for raw in inputs {
        let shown = String::from_utf8_lossy(raw);
        match parse_line_boxed(raw) {
            Ok((sku, qty)) => println!("    {shown:<11} → {sku} x{qty}"),
            Err(e) => println!("    {shown:<11} → [{}] {e}", inner_type(e.as_ref())),
        }
    }
    // 출력: KR-310:x → [ParseIntError] invalid digit found in string
    //       KR-205:120 → [알 수 없음 (문자열에서 만든 에러)] 수량 120이(가) 최대 99를 넘습니다

    println!("2. 라이브러리가 에러를 한 겹 감싸면");
    for text in ["7", "x7"] {
        let v1 = qty_v1::parse_qty(text);
        let v2 = qty_v2::parse_qty(text);
        println!("    {text:?}");
        println!("      v1 바깥만 다운캐스트: {}", react_outer(&v1));
        println!("      v2 바깥만 다운캐스트: {}", react_outer(&v2));
        println!("      v2 체인에서 찾기:     {}", react_chain(&v2));
    }
    // 출력("x7"): v1 바깥만 다운캐스트: 형식 오류 → 0개로 처리
    //             v2 바깥만 다운캐스트: 모르는 에러 → 작업 중단 (수량 "x7"을(를) 읽지 못했습니다)
    //             v2 체인에서 찾기:     형식 오류(InvalidDigit) → 0개로 처리
    // 컴파일 에러도 경고도 없이 처리 방침이 바뀌었습니다.

    println!("3. 대응할 실패만 배리언트로, 나머지는 Box로");
    let cases: [(&[u8], u32); 4] = [(b"12", 5), (b"3", 5), (b"1a", 5), (b"\xff", 5)];
    for (record, want) in cases {
        let shown = String::from_utf8_lossy(record);
        print!("    재고 기록 {shown:?}, {want}개 예약 → ");
        match reserve("KR-102", record, want) {
            Ok(left) => println!("완료 (남은 재고 {left}개)"),
            Err(StockError::OutOfStock { sku, have, want }) => {
                println!("입고 요청: {sku} {}개", want - have)
            }
            Err(e @ StockError::Storage(_)) => {
                println!("보고: {e}");
                let mut cause = e.source();
                while let Some(c) = cause {
                    println!("        원인: {c}");
                    cause = c.source();
                }
            }
        }
    }
    // 출력: "3" → 입고 요청: KR-102 2개   (OutOfStock은 match로 대응)
    //       "1a" → 보고: 재고 기록을 읽지 못했습니다 / 원인: invalid digit found in string
}
