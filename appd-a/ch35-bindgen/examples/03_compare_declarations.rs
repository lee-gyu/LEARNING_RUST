//! 손으로 쓴 선언과 bindgen이 생성한 선언을 항목별로 견줍니다.
//!
//! 실행: cargo run -p er-ch35-bindgen --example 03_compare_declarations
//!
//! `projects/tally_bindings`에 있는 두 파일을 `include_str!`로 읽어 글자 그대로 비교합니다.
//!
//! - 손으로 쓴 선언: `handwritten/src/ffi.rs` (ch34의 것과 같음)
//! - 생성한 선언: `generated/tally-sys/src/committed_bindings.rs` (bindgen 0.73.2가 v1 헤더에서
//!   생성해 커밋해 둔 것)
//!
//! 두 래퍼(`handwritten/src/lib.rs`, `generated/tally/src/lib.rs`)의 unsafe 블록 수도 셉니다.
//! 파서는 이 두 파일의 모양만 다루는 작은 것이라 일반적인 Rust 코드에는 쓸 수 없습니다.
//! 파일을 고치면 출력도 바뀝니다(C 코드나 libclang 없이 실행됩니다).

const HANDWRITTEN: &str = include_str!("../projects/tally_bindings/handwritten/src/ffi.rs");
const GENERATED: &str =
    include_str!("../projects/tally_bindings/generated/tally-sys/src/committed_bindings.rs");
const WRAPPER_HANDWRITTEN: &str = include_str!("../projects/tally_bindings/handwritten/src/lib.rs");
const WRAPPER_GENERATED: &str =
    include_str!("../projects/tally_bindings/generated/tally/src/lib.rs");

/// extern 블록 안의 함수 선언 하나
struct Function {
    name: String,
    /// 매개변수와 반환 타입(경로 접두사를 지운 것)
    signature: String,
    /// "safe fn", "unsafe fn", 또는 표시 없음
    marker: &'static str,
}

/// `pub const 이름: 타입 = 값;`
struct Constant {
    name: String,
    ty: String,
    value: String,
}

/// 구조체 하나: 이름, derive 목록, 필드 목록
struct Struct {
    name: String,
    derives: String,
    fields: Vec<String>,
}

/// 타입 경로의 접두사를 지워 두 파일의 표기를 맞춥니다(`::std::os::raw::c_char` → `c_char`).
fn normalize(s: &str) -> String {
    s.replace("::std::os::raw::", "")
        .replace("::core::ffi::", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// 주석과 빈 줄을 뺀 코드 줄
fn code_lines(src: &str) -> impl Iterator<Item = &str> {
    src.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
}

fn functions(src: &str) -> Vec<Function> {
    let mut out = Vec::new();
    for line in code_lines(src) {
        let (marker, rest) = if let Some(r) = line.strip_prefix("pub safe fn ") {
            ("safe fn", r)
        } else if let Some(r) = line.strip_prefix("pub unsafe fn ") {
            ("unsafe fn", r)
        } else if let Some(r) = line.strip_prefix("pub fn ") {
            ("표시 없음", r)
        } else {
            continue;
        };
        let (name, signature) = rest.split_once('(').expect("함수 선언에는 괄호가 있습니다");
        out.push(Function {
            name: name.to_string(),
            signature: normalize(&format!("({}", signature.trim_end_matches(';'))),
            marker,
        });
    }
    out
}

fn constants(src: &str) -> Vec<Constant> {
    let mut out = Vec::new();
    for line in code_lines(src) {
        let Some(rest) = line.strip_prefix("pub const ") else {
            continue;
        };
        let (name, rest) = rest.split_once(':').expect("상수에는 타입이 있습니다");
        let (ty, value) = rest.split_once('=').expect("상수에는 값이 있습니다");
        out.push(Constant {
            name: name.trim().to_string(),
            ty: normalize(ty),
            value: value.trim().trim_end_matches(';').to_string(),
        });
    }
    out
}

fn structs(src: &str) -> Vec<Struct> {
    let mut out = Vec::new();
    let mut derives = String::new();
    let mut current: Option<Struct> = None;
    for line in code_lines(src) {
        if let Some(s) = current.as_mut() {
            if line == "}" {
                out.extend(current.take());
            } else {
                s.fields.push(normalize(line.trim_end_matches(',')));
            }
        } else if let Some(d) = line.strip_prefix("#[derive(") {
            derives = d.trim_end_matches(")]").to_string();
        } else if let Some(rest) = line.strip_prefix("pub struct ") {
            current = Some(Struct {
                name: rest.trim_end_matches(" {").to_string(),
                derives: std::mem::take(&mut derives),
                fields: Vec::new(),
            });
        }
    }
    out
}

/// 손으로 쓴 상수 이름에 해당하는 생성판 상수를 찾습니다. enum 값은 `<enum>_<이름>`이 됩니다.
fn find_constant<'a>(list: &'a [Constant], name: &str) -> Option<&'a Constant> {
    list.iter()
        .find(|c| c.name == name)
        .or_else(|| list.iter().find(|c| c.name.ends_with(&format!("_{name}"))))
}

fn main() {
    let (hand_fns, gen_fns) = (functions(HANDWRITTEN), functions(GENERATED));
    println!(
        "1. 함수 (손으로 쓴 판 {}개, 생성판 {}개)",
        hand_fns.len(),
        gen_fns.len()
    );
    for h in &hand_fns {
        let Some(g) = gen_fns.iter().find(|g| g.name == h.name) else {
            println!("   {:<22} 생성판에 없음", h.name);
            continue;
        };
        let same = if h.signature == g.signature {
            "시그니처 같음"
        } else {
            "시그니처 다름"
        };
        println!(
            "   {:<22} {same}, 손: {:<9} 생성: {}",
            h.name, h.marker, g.marker
        );
    }

    let (hand_consts, gen_consts) = (constants(HANDWRITTEN), constants(GENERATED));
    println!("2. 상수 (손으로 쓴 판 {}개)", hand_consts.len());
    for h in &hand_consts {
        let gen_text = match find_constant(&gen_consts, &h.name) {
            Some(g) => format!("{}: {} = {}", g.name, g.ty, g.value),
            None => "없음".to_string(),
        };
        println!("   손: {}: {} = {}", h.name, h.ty, h.value);
        println!("       생성: {gen_text}");
    }

    let (hand_structs, gen_structs) = (structs(HANDWRITTEN), structs(GENERATED));
    println!("3. 구조체");
    for h in &hand_structs {
        let Some(g) = gen_structs.iter().find(|g| g.name == h.name) else {
            println!("   {} 생성판에 없음", h.name);
            continue;
        };
        if h.fields == g.fields {
            println!("   {}: 필드 같음 ({})", h.name, h.fields.join(", "));
        } else {
            println!("   {}: 필드 다름", h.name);
            println!("       손: {}", h.fields.join(", "));
            println!("       생성: {}", g.fields.join(", "));
        }
        println!("       derive 손: [{}] 생성: [{}]", h.derives, g.derives);
    }

    println!("4. 생성판에만 있는 것");
    for line in code_lines(GENERATED).filter(|l| l.starts_with("pub type ")) {
        println!("   {}", normalize(line));
    }
    let count = |key: &str| GENERATED.matches(key).count();
    println!(
        "   레이아웃 검사: 크기 {}개, 정렬 {}개, 필드 오프셋 {}개",
        count("[\"Size of "),
        count("[\"Alignment of "),
        count("[\"Offset of field: ")
    );
    println!(
        "   extern 블록: 손으로 쓴 판 {}개, 생성판 {}개",
        HANDWRITTEN.matches("unsafe extern \"C\" {").count(),
        GENERATED.matches("unsafe extern \"C\" {").count()
    );

    println!("5. 그 위의 래퍼");
    let unsafe_blocks = |src: &str| src.matches("unsafe {").count();
    println!(
        "   unsafe 블록: 손으로 쓴 판 위 {}개, 생성판 위 {}개",
        unsafe_blocks(WRAPPER_HANDWRITTEN),
        unsafe_blocks(WRAPPER_GENERATED)
    );
    println!(
        "   생성판 래퍼에서 [생성판] 표시를 단 곳: {}곳",
        WRAPPER_GENERATED.matches("// [생성판]").count()
    );
}
