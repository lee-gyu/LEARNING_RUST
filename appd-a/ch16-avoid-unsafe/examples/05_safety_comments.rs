//! 지침 4: 전제 조건을 안전성 주석으로 적고, `unsafe` 블록을 작게 만들고, 린트로 강제한다.
//!
//! 기차표 예매 기록 파일은 한 줄이 25바이트인 고정 폭 ASCII 형식입니다.
//!
//! | 위치 | 내용 | 예 |
//! | --- | --- | --- |
//! | 0..3 | 열차 종류 | `KTX` |
//! | 3..7 | 열차 번호 | `0101` |
//! | 7..14 | 출발역(빈칸으로 채움) | `SEOUL  ` |
//! | 14..21 | 도착역(빈칸으로 채움) | `BUSAN  ` |
//! | 21..25 | 출발 시각 | `0930` |
//!
//! 한 줄에서 칸 하나를 잘라 `&str`로 만드는 `unsafe fn`을 세 판으로 씁니다. 세 판은 같은 결과를
//! 내고, 다른 것은 컴파일러와 Clippy가 하는 말, 그리고 리뷰할 때 확인해야 할 양입니다.
//!
//! - `field_2021`(비권장): 2021 에디션까지의 습관. `unsafe fn` 본문에 블록 없이 unsafe 연산을 씀.
//! - `field_one_block`(비권장): 연산 둘을 블록 하나에 넣고 주석 하나로 뭉뚱그림.
//! - `field`(권장): 문서에 `# Safety` 절, 연산마다 블록 하나와 `SAFETY:` 주석.
//!
//! 맨 위의 `#![warn(..)]`은 기본으로 꺼진 Clippy 린트 두 개를 이 파일에서 켭니다.
//! 이 파일의 `unsafe` 블록은 Miri로 검사하지 못했습니다(nightly 툴체인을 설치하지 않음).
//!
//! 실행: cargo run -p er-ch16-avoid-unsafe --example 05_safety_comments

#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::multiple_unsafe_ops_per_block
)]

use std::ops::Range;

const LINE_LEN: usize = 25;
const FIELDS: [Range<usize>; 5] = [0..3, 3..7, 7..14, 14..21, 21..25];

/// 예매 기록 네 줄. 3번 줄은 짧고, 4번 줄은 도착역에 한글("대전 ", UTF-8 7바이트)이 들어 있습니다.
const LINES: [&[u8]; 4] = [
    b"KTX0101SEOUL  BUSAN  0930",
    b"SRT0305SUSEO  MOKPO  1415",
    b"ITX1203YONGSAN",
    b"KTX0107SEOUL  \xEB\x8C\x80\xEC\xA0\x84 0800",
];

/// 비권장 1: 2021 에디션까지는 unsafe fn의 본문 전체가 unsafe 블록처럼 취급됐습니다.
/// 2024 에디션에서는 rustc가 unsafe_op_in_unsafe_fn으로 연산마다 경고합니다.
#[expect(
    unsafe_op_in_unsafe_fn,
    reason = "비권장 판: 본문의 어느 연산이 위험한지 드러나지 않는 모양"
)]
#[expect(
    clippy::missing_safety_doc,
    reason = "비권장 판: 부르는 쪽이 지킬 조건을 적지 않음"
)]
pub unsafe fn field_2021(line: &[u8], range: Range<usize>) -> &str {
    std::str::from_utf8_unchecked(line.get_unchecked(range))
}

/// 줄에서 `range` 부분을 `&str`로 봅니다.
///
/// # Safety
///
/// `range`가 줄 안에 있고 그 부분이 올바른 UTF-8이어야 합니다.
#[expect(
    clippy::multiple_unsafe_ops_per_block,
    reason = "비권장 판: 블록 하나에 전제 조건이 다른 연산 둘"
)]
pub unsafe fn field_one_block(line: &[u8], range: Range<usize>) -> &str {
    // SAFETY: 부르는 쪽이 보장합니다.
    unsafe { std::str::from_utf8_unchecked(line.get_unchecked(range)) }
}

/// 줄에서 `range` 부분을 `&str`로 봅니다. 아무것도 검사하지 않습니다.
///
/// # Safety
///
/// 1. `range.start <= range.end <= line.len()`이어야 합니다.
/// 2. `line[range]`가 올바른 UTF-8이어야 합니다. 줄 전체가 UTF-8이어도 칸 경계가 글자 중간을
///    자르면 그 칸은 UTF-8이 아니므로, 줄 전체가 ASCII인지 확인하는 편이 쉽습니다.
pub unsafe fn field(line: &[u8], range: Range<usize>) -> &str {
    // SAFETY: 조건 1에 따라 range는 line 안에 있습니다.
    let bytes = unsafe { line.get_unchecked(range) };
    // SAFETY: 조건 2에 따라 bytes는 올바른 UTF-8입니다.
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

type FieldFn = unsafe fn(&[u8], Range<usize>) -> &str;

/// 부르는 쪽: 전제 조건을 한 번 확인한 뒤 다섯 칸을 자릅니다.
fn split_fields(line: &[u8], field_fn: FieldFn) -> Result<[&str; 5], String> {
    if line.len() != LINE_LEN {
        return Err(format!("길이가 {LINE_LEN}가 아님({}바이트)", line.len()));
    }
    if !line.is_ascii() {
        return Err(String::from("ASCII가 아님"));
    }
    Ok(FIELDS.map(|range| {
        // SAFETY: FIELDS의 범위는 모두 0..25 안이고 위에서 line.len() == 25를 확인했으므로 조건 1을,
        // line 전체가 ASCII임을 확인했으므로 어느 부분이든 올바른 UTF-8이라 조건 2를 지킵니다.
        unsafe { field_fn(line, range) }
    }))
}

/// 안전한 판: 검사하는 get과 str::from_utf8을 씁니다. unsafe가 없습니다.
fn split_fields_safe(line: &[u8]) -> Result<[&str; 5], String> {
    if line.len() != LINE_LEN {
        return Err(format!("길이가 {LINE_LEN}가 아님({}바이트)", line.len()));
    }
    let mut out = [""; 5];
    for (slot, range) in out.iter_mut().zip(FIELDS) {
        let bytes = line.get(range).ok_or("칸이 줄 밖에 있음")?;
        *slot = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

fn describe([kind, number, from, to, time]: [&str; 5]) -> String {
    let time = match time.split_at_checked(2) {
        Some((h, m)) => format!("{h}:{m}"),
        None => time.to_string(),
    };
    format!(
        "{kind} {number} {} → {} {time}",
        from.trim_end(),
        to.trim_end()
    )
}

fn main() {
    println!("1. 세 판의 unsafe fn (부르는 쪽이 길이와 ASCII를 한 번 확인)");
    for (n, line) in LINES.iter().enumerate() {
        let results = [field_2021, field_one_block, field].map(|f| split_fields(line, f));
        match &results {
            [Ok(a), Ok(b), Ok(c)] => {
                let same = a == b && b == c;
                println!("    줄 {}: {} (세 판이 같음: {same})", n + 1, describe(*c));
            }
            [.., Err(e)] => println!("    줄 {}: {e} → unsafe fn을 부르지 않음", n + 1),
            _ => println!("    줄 {}: 세 판의 결과가 다름", n + 1),
        }
    }
    // 출력:     줄 1: KTX 0101 SEOUL → BUSAN 09:30 (세 판이 같음: true)
    // 출력:     줄 2: SRT 0305 SUSEO → MOKPO 14:15 (세 판이 같음: true)
    // 출력:     줄 3: 길이가 25가 아님(14바이트) → unsafe fn을 부르지 않음
    // 출력:     줄 4: ASCII가 아님 → unsafe fn을 부르지 않음

    println!("2. 안전한 판 (get + str::from_utf8, unsafe 없음)");
    for (n, line) in LINES.iter().enumerate() {
        match split_fields_safe(line) {
            Ok(fields) => println!("    줄 {}: {}", n + 1, describe(fields)),
            Err(e) => println!("    줄 {}: {e}", n + 1),
        }
    }
    // 출력:     줄 1: KTX 0101 SEOUL → BUSAN 09:30
    // 출력:     줄 2: SRT 0305 SUSEO → MOKPO 14:15
    // 출력:     줄 3: 길이가 25가 아님(14바이트)
    // 출력:     줄 4: KTX 0107 SEOUL → 대전 08:00
}
