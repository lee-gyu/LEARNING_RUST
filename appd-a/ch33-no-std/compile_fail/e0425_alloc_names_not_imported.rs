// 기대하는 에러: E0425 cannot find type `Vec` in this scope (`String`도 같음)
//   help: consider importing this struct
//         use alloc::vec::Vec;
// 함께 나오는 에러: cannot find macro `vec` / `format` in this scope (에러 코드 없음, help: use alloc::vec;),
//   E0599 no method named `to_string` found for type `u32` (help: use crate::alloc::string::ToString;)
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0425_alloc_names_not_imported.rs
//
// `extern crate alloc;`은 alloc 크레이트를 연결할 뿐, 그 안의 이름을 프렐류드에 넣지 않습니다.
// `#![no_std]` 크레이트의 프렐류드는 core의 것이라 `Vec`, `String`, `vec!`, `format!`, `ToString`이
// 없습니다. 하나씩 `use alloc::…`로 가져오면 컴파일됩니다(examples/02_no_std_imports.rs).
// `extern crate alloc;` 줄마저 없으면 같은 에러에서 `help:`(가져올 경로 제안)가 사라집니다.

#![no_std]

extern crate alloc;

/// 기준값을 넘은 측정값만 골라 냅니다.
pub fn over(readings: &[u32], limit: u32) -> Vec<u32> {
    let mut picked = vec![];
    for r in readings {
        if *r > limit {
            picked.push(*r);
        }
    }
    picked
}

/// 측정값을 단위와 함께 문자열로 만듭니다.
pub fn label(r: u32) -> String {
    let mut s = r.to_string();
    s.push_str("mV");
    s
}

/// 보고서 제목을 만듭니다.
pub fn title(count: usize) -> String {
    format!("측정값 {count}개")
}
