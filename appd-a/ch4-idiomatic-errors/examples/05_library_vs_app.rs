// 지침 5: 라이브러리는 구체 에러 타입(열거형)을 공개하고, 애플리케이션의 위층은 anyhow::Error로 모읍니다.
//
//   1) 비권장: 라이브러리가 anyhow::Error를 돌려주면 호출한 쪽은 실패 종류로 match할 수 없습니다.
//   2) 권장: 라이브러리는 thiserror 열거형, 애플리케이션은 anyhow::Result. ?가 어떤 에러든 모으고,
//      {:#}와 {:?}가 source 체인을 보고서로 만들어 줍니다.
//   3) 애플리케이션도 필요하면 downcast_ref로 라이브러리의 구체 타입을 되찾아 대응합니다.
//   4) anyhow::Error는 포인터 하나 크기이고, Error 트레이트를 구현하지 않습니다.
//
// 실행: cargo run -p er-ch4-idiomatic-errors --example 05_library_vs_app
// 의존성: anyhow 1.0.104, thiserror 2.0.21 ([dev-dependencies])
// 주의: RUST_BACKTRACE나 RUST_LIB_BACKTRACE를 켜고 실행하면 2번 구역의 {:?} 출력 뒤에 백트레이스가 붙습니다.

use std::error::Error;
use std::mem::size_of;
use std::num::ParseIntError;
use std::str::Utf8Error;

// ── 비권장: 라이브러리가 anyhow::Error를 공개 ───────────────────────────
mod orders_anyhow {
    use anyhow::bail;
    use std::str;

    pub fn parse_line(raw: &[u8]) -> anyhow::Result<(String, u32)> {
        let text = str::from_utf8(raw)?;
        let Some((sku, qty)) = text.split_once(':') else {
            bail!("':' 구분자가 없습니다: {text:?}"); // 타입 없는 에러를 만들어 return
        };
        let qty: u32 = qty.trim().parse()?;
        if qty > 99 {
            bail!("수량 {qty}이(가) 최대 99를 넘습니다");
        }
        Ok((sku.to_string(), qty))
    }
}

// ── 권장: 라이브러리는 구체 타입을 공개 ─────────────────────────────────
mod orders {
    use std::num::ParseIntError;
    use std::str::{self, Utf8Error};

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

    #[derive(Debug, thiserror::Error)]
    #[error("{line_no}번째 줄을 처리하지 못했습니다")]
    pub struct SheetError {
        pub line_no: usize,
        #[source]
        pub cause: OrderError,
    }

    pub fn parse_line(raw: &[u8]) -> Result<(String, u32), OrderError> {
        let text = str::from_utf8(raw)?;
        let (sku, qty) = text
            .split_once(':')
            .ok_or_else(|| OrderError::MissingColon {
                line: text.to_string(),
            })?;
        let qty: u32 = qty.trim().parse()?;
        if qty > 99 {
            return Err(OrderError::TooMany {
                requested: qty,
                max: 99,
            });
        }
        Ok((sku.to_string(), qty))
    }

    pub fn total_qty(lines: &[&[u8]]) -> Result<u32, SheetError> {
        let mut total = 0;
        for (i, raw) in lines.iter().enumerate() {
            let (_, qty) = parse_line(raw).map_err(|cause| SheetError {
                line_no: i + 1,
                cause,
            })?;
            total += qty;
        }
        Ok(total)
    }
}

// ── 애플리케이션: 위층은 anyhow::Result ─────────────────────────────────
// 라이브러리의 SheetError도, 애플리케이션이 즉석에서 만드는 에러도 ? 하나로 anyhow::Error가 됩니다.
fn plan_shipment(lines: &[&[u8]], daily_limit: u32) -> anyhow::Result<u32> {
    let total = orders::total_qty(lines)?; // SheetError → anyhow::Error
    if total > daily_limit {
        // 애플리케이션에서만 쓰는 실패라면 타입을 만들 필요 없이 메시지로 충분합니다.
        anyhow::bail!("총 수량 {total}개가 하루 처리량 {daily_limit}개를 넘습니다");
    }
    Ok(total.div_ceil(10)) // 상자 하나에 10개씩
}

fn main() {
    let lines: [&[u8]; 3] = [b"KR-102:3", b"KR-205:120", b"KR-310:x"];

    println!("1. 같은 처리 방침(수량 초과는 보류), 두 가지 라이브러리");
    for raw in lines {
        let shown = String::from_utf8_lossy(raw);
        // 비권장 판: match할 배리언트가 없으므로 다운캐스트로 물어봅니다. 표준 에러는 타입이 남아
        // 있지만, bail!로 만든 에러는 타입이 없어 메시지를 읽는 것 말고는 방법이 없습니다.
        let a = match orders_anyhow::parse_line(raw) {
            Ok((sku, qty)) => format!("접수 {sku} x{qty}"),
            Err(e) if e.is::<ParseIntError>() || e.is::<Utf8Error>() => format!("버림 → {e}"),
            Err(e) => format!("타입 없음, 메시지뿐 → {e}"),
        };
        // 권장 판: 배리언트로 match합니다.
        let b = match orders::parse_line(raw) {
            Ok((sku, qty)) => format!("접수 {sku} x{qty}"),
            Err(orders::OrderError::TooMany { requested, max }) => {
                format!("보류 ({requested} > {max})")
            }
            Err(e) => format!("버림 → {e}"),
        };
        println!("    {shown}");
        println!("      anyhow를 공개한 라이브러리:   {a}");
        println!("      구체 타입을 공개한 라이브러리: {b}");
    }
    // 출력(KR-205:120): anyhow를 공개한 라이브러리:   타입 없음, 메시지뿐 → 수량 120이(가) 최대 99를 넘습니다
    //                   구체 타입을 공개한 라이브러리: 보류 (120 > 99)

    println!("2. 애플리케이션의 에러 보고");
    let good: [&[u8]; 2] = [b"KR-102:30", b"US-310:25"];
    let bad: [&[u8]; 3] = [b"KR-102:3", b"US-310:4", b"KR-205:x"];
    match plan_shipment(&good, 100) {
        Ok(boxes) => println!("    good, 한도 100 → 상자 {boxes}개"),
        Err(e) => println!("    good, 한도 100 → {e}"),
    }
    if let Err(e) = plan_shipment(&good, 50) {
        println!("    good, 한도 50  → {{}}  {e}");
    }
    let Err(e) = plan_shipment(&bad, 100) else {
        println!("    bad: 예상과 달리 성공했습니다");
        return;
    };
    println!("    bad,  한도 100 → {{}}  {e}");
    println!("    bad,  한도 100 → {{:#}} {e:#}");
    println!("    bad,  한도 100 → {{:?}} 아래 여러 줄");
    for line in format!("{e:?}").lines() {
        println!("        |{}{line}", if line.is_empty() { "" } else { " " });
    }
    // 출력: {}는 맨 바깥 에러만, {:#}는 체인 전체를 ": "로 이어 한 줄로,
    //       {:?}는 "Caused by:" 아래에 원인을 나열합니다(여기서는 원인이 둘이라 0:, 1: 번호가 붙음).

    println!("3. 필요하면 구체 타입을 되찾습니다");
    if let Some(sheet) = e.downcast_ref::<orders::SheetError>() {
        println!(
            "    SheetError: line_no = {}, cause = {:?}",
            sheet.line_no, sheet.cause
        );
    }
    // 출력: SheetError: line_no = 3, cause = BadQty(ParseIntError { kind: InvalidDigit })
    // chain()은 source 체인을 반복자로 내줍니다(맨 바깥 에러부터).
    let depth = e.chain().count();
    let root = e.root_cause();
    println!("    체인 길이 = {depth}, 맨 아래 원인 = {root}");
    // 출력: 체인 길이 = 3, 맨 아래 원인 = invalid digit found in string

    println!("4. anyhow::Error의 성질");
    // anyhow::Error는 Error를 구현하지 않습니다. &dyn Error가 필요하면 as_ref()로 빌립니다.
    let as_dyn: &(dyn Error + Send + Sync + 'static) = e.as_ref();
    println!("    as_ref()로 빌린 &dyn Error: {as_dyn}");
    println!(
        "    size_of: anyhow::Error = {}, Box<dyn Error + Send + Sync> = {}",
        size_of::<anyhow::Error>(),
        size_of::<Box<dyn Error + Send + Sync>>()
    );
    println!(
        "    size_of: Result<(), anyhow::Error> = {}, Result<(), Box<dyn Error + Send + Sync>> = {}",
        size_of::<Result<(), anyhow::Error>>(),
        size_of::<Result<(), Box<dyn Error + Send + Sync>>>()
    );
    // 출력: 8, 16 / 8, 16 (64비트 플랫폼)  ← anyhow::Error는 포인터 하나, Box<dyn Error>는 포인터 둘
}
