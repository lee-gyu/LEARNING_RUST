// 지침 1: 에러 타입에는 Error 트레이트를 구현합니다.
// 메시지 하나면 충분한 "최소 에러"도 String을 그대로 쓰지 말고 String을 감싼 뉴타입으로 만듭니다.
//
//   1) std::error::Error의 정체는 core의 트레이트이고, 뉴타입은 String과 크기가 같습니다.
//   2) 비권장(String)과 권장(뉴타입)은 메시지가 똑같습니다. 차이는 다른 코드와 섞일 때 나타납니다.
//   3) 위층 에러의 원인(source)으로 넣을 수 있는가
//   4) Box<dyn Error>에 담긴 뒤 어디서 온 에러인지 물어볼 수 있는가
//
// 실행: cargo run -p er-ch4-idiomatic-errors --example 01_string_to_newtype

use std::any::type_name;
use std::error::Error;
use std::fmt;
use std::mem::size_of;

// ── 검사 규칙 ─────────────────────────────────────────────────────────
// 상품 코드는 "영문 대문자 2자 + '-' + 숫자 3자"(예: KR-102)입니다.
// 이 함수는 예전부터 있던 코드라고 생각합니다. 실패를 String으로 알립니다.
fn check_format(code: &str) -> Result<(), String> {
    let Some((prefix, digits)) = code.split_once('-') else {
        return Err(format!("상품 코드에 '-'가 없습니다: {code:?}"));
    };
    if prefix.len() != 2 || !prefix.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(format!(
            "상품 코드의 앞부분은 영문 대문자 2자여야 합니다: {code:?}"
        ));
    }
    if digits.len() != 3 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!(
            "상품 코드의 뒷부분은 숫자 3자여야 합니다: {code:?}"
        ));
    }
    Ok(())
}

// ── 비권장: String을 에러 타입으로 그대로 공개 ──────────────────────────
fn check_sku_str(code: &str) -> Result<(), String> {
    check_format(code)
}

// ── 권장: String을 감싼 뉴타입 ────────────────────────────────────────
// String에는 Error를 구현할 수 없습니다(고아 규칙, compile_fail/e0117_error_for_string_alias.rs).
// 내 크레이트의 타입으로 한 번 감싸면 Debug, Display, Error를 모두 구현할 수 있습니다.
#[derive(Debug)]
struct SkuError(String);

impl fmt::Display for SkuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// 필수 메서드가 없으므로 빈 블록이면 됩니다. 원인이 없는 에러라 source()도 기본 구현(None) 그대로입니다.
impl Error for SkuError {}

// From<String>을 구현해 두면 String을 돌려주는 옛 함수에도 ?를 쓸 수 있습니다.
impl From<String> for SkuError {
    fn from(message: String) -> Self {
        SkuError(message)
    }
}

fn check_sku(code: &str) -> Result<(), SkuError> {
    check_format(code)?; // String → SkuError (From<String>)
    Ok(())
}

// ── 3번 구역에서 쓰는 위층 에러 ───────────────────────────────────────
// 권장: 원인을 필드로 품고 source()로 내줍니다.
#[derive(Debug)]
struct LineError {
    line_no: usize,
    source: SkuError,
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}번째 줄을 처리하지 못했습니다", self.line_no)
    }
}

impl Error for LineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

// 비권장 쪽: String은 Error가 아니라서 source()로 내줄 수 없습니다
// (compile_fail/e0277_string_as_source.rs). String을 그대로 품으면 메시지를 이어 붙이는 수밖에 없습니다.
#[derive(Debug)]
struct LineErrorFlat {
    line_no: usize,
    message: String,
}

impl fmt::Display for LineErrorFlat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}번째 줄을 처리하지 못했습니다: {}",
            self.line_no, self.message
        )
    }
}

impl Error for LineErrorFlat {}

fn first_bad_line_str(codes: &[&str]) -> Result<(), LineErrorFlat> {
    for (i, code) in codes.iter().enumerate() {
        check_sku_str(code).map_err(|message| LineErrorFlat {
            line_no: i + 1,
            message,
        })?;
    }
    Ok(())
}

fn first_bad_line(codes: &[&str]) -> Result<(), LineError> {
    for (i, code) in codes.iter().enumerate() {
        check_sku(code).map_err(|source| LineError {
            line_no: i + 1,
            source,
        })?;
    }
    Ok(())
}

// source 체인을 따라 내려가며 출력하고, 체인의 맨 아래가 SkuError인지 확인합니다.
// 로그 도구나 anyhow 같은 라이브러리도 이렇게 source()만 보고 체인을 다룹니다.
fn report(label: &str, err: &(dyn Error + 'static)) {
    println!("    [{label}]");
    println!("      에러: {err}");
    let mut bottom = err;
    while let Some(cause) = bottom.source() {
        println!("        원인: {cause}");
        bottom = cause;
    }
    println!(
        "      체인의 맨 아래가 SkuError인가: {}",
        bottom.is::<SkuError>()
    );
}

// ── 4번 구역: Box<dyn Error>로 올려 보내기 ─────────────────────────────
// 표준 라이브러리에 From<String> for Box<dyn Error>가 있으므로 두 함수 모두 ?로 올라갑니다.
fn validate_all_str(codes: &[&str]) -> Result<(), Box<dyn Error>> {
    for code in codes {
        check_sku_str(code)?;
    }
    Ok(())
}

fn validate_all(codes: &[&str]) -> Result<(), Box<dyn Error>> {
    for code in codes {
        check_sku(code)?;
    }
    Ok(())
}

fn main() {
    println!("1. Error 트레이트의 정체와 뉴타입의 크기");
    println!(
        "    type_name::<dyn std::error::Error>() = {}",
        type_name::<dyn Error>()
    );
    // 출력: dyn core::error::Error  ← std의 Error는 core의 트레이트를 다시 내보낸 것입니다.
    println!(
        "    size_of::<String>() = {}, size_of::<SkuError>() = {}",
        size_of::<String>(),
        size_of::<SkuError>()
    );
    // 출력: 24, 24  ← 뉴타입으로 감싸도 크기가 늘지 않습니다.

    // "KR-1O2"의 가운데 글자는 숫자 0이 아니라 영문 O입니다.
    let codes = ["KR-102", "KR102", "kr-102", "KR-1O2"];

    println!("2. 메시지는 똑같습니다");
    for code in codes {
        let a = match check_sku_str(code) {
            Ok(()) => "통과".to_string(),
            Err(e) => e,
        };
        let b = match check_sku(code) {
            Ok(()) => "통과".to_string(),
            Err(e) => e.to_string(),
        };
        println!("    {code:<7} String: {a}");
        println!("    {:<7} 뉴타입: {b}", "");
    }

    println!("3. 위층 에러에 원인으로 넣기");
    let sheet = ["KR-102", "KR-1O2", "US-310"];
    if let Err(e) = first_bad_line_str(&sheet) {
        report("String", &e);
    }
    if let Err(e) = first_bad_line(&sheet) {
        report("뉴타입", &e);
    }
    // 출력(String): 에러 한 줄에 메시지가 이어 붙고, 체인의 맨 아래가 SkuError인가: false
    // 출력(뉴타입): 에러와 원인이 두 층으로 나뉘고, 체인의 맨 아래가 SkuError인가: true

    println!("4. Box<dyn Error>에 담긴 뒤 어디서 온 에러인지 묻기");
    let batch = ["KR-102", "KR102"];
    if let Err(e) = validate_all_str(&batch) {
        // String에서 만든 에러는 표준 라이브러리의 비공개 타입에 담깁니다.
        // e.is::<String>()은 String이 Error가 아니라서 컴파일되지 않습니다.
        println!("    [String] {{:?}} = {e:?}");
        println!("             is::<SkuError>() = {}", e.is::<SkuError>());
        // 출력: {:?} = "상품 코드에 '-'가 없습니다: \"KR102\"", is::<SkuError>() = false
    }
    if let Err(e) = validate_all(&batch) {
        println!("    [뉴타입] {{:?}} = {e:?}");
        println!("             is::<SkuError>() = {}", e.is::<SkuError>());
        // 출력: {:?} = SkuError("상품 코드에 '-'가 없습니다: \"KR102\""), is::<SkuError>() = true
    }
}
