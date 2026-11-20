//! 크레이트 루트에 `#![no_std]`를 붙이면 std의 프렐류드가 사라지고 core의 프렐류드만 남습니다.
//! `Vec`, `String`, `Box`, `BTreeMap`, `format!`, `vec!`, `.to_string()`은 모두 `alloc`에서 직접
//! 가져와야 합니다.
//!
//! 실행: cargo run -p er-ch33-no-std --example 02_no_std_imports
//!
//! 이 파일은 실행 파일이라서 `extern crate std;`로 std를 다시 연결했습니다. 그래야 패닉 처리기,
//! 전역 할당기, 프로그램 진입점이 생깁니다. 이 줄을 지우면(`std::println!`도 지우고) 빌드가
//! `no global memory allocator found but one is required`, ``#[panic_handler]` function required,
//! but not found``, `unwinding panics are not supported without std`로 실패합니다(`no_std` 실행
//! 파일은 Part 8 `ch1-intro-no-std`에서 다룹니다). std를 연결해도 프렐류드는 core의 것 그대로이므로, 아래
//! `portable` 모듈은 `no_std` 라이브러리와 똑같은 가져오기를 써야 컴파일됩니다. std는 `main`에서
//! 출력할 때만 씁니다.

#![no_std]

extern crate alloc;
extern crate std;

mod portable {
    // core의 프렐류드에 있는 것(`Option`, `Result`, `Iterator`, `Clone`, `Default`, `TryFrom` 등)은
    // 그대로 씁니다. 아래는 std의 프렐류드에는 있지만 core의 프렐류드에는 없는 이름입니다.
    use alloc::boxed::Box;
    use alloc::collections::BTreeMap;
    use alloc::format;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloc::vec::Vec;
    use core::fmt;

    /// 한 줄이 "이름 수량" 모양이 아닐 때의 에러입니다.
    #[derive(Debug)]
    pub struct LineError {
        pub line: usize,
        pub text: String,
    }

    impl fmt::Display for LineError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}번째 줄 {:?}은 '이름 수량' 모양이 아닙니다",
                self.line, self.text
            )
        }
    }

    // 1.81부터 `Error` 트레이트가 core에 있으므로 std 없이 구현할 수 있습니다.
    impl core::error::Error for LineError {}

    /// "이름 수량" 줄을 이름별 합계로 모읍니다. 이름 순서로 보여 주려고 `BTreeMap`을 씁니다.
    pub fn tally(input: &str) -> Result<BTreeMap<String, u32>, Box<dyn core::error::Error>> {
        let mut totals = BTreeMap::new();
        for (i, line) in input.lines().enumerate() {
            let mut words = line.split_whitespace();
            let (Some(name), Some(qty), None) = (words.next(), words.next(), words.next()) else {
                let err = LineError {
                    line: i + 1,
                    text: line.to_string(), // `ToString`을 가져오지 않으면 E0599
                };
                return Err(Box::new(err));
            };
            // `ParseIntError`(core)를 `Box<dyn Error>`(alloc)로 바꾸는 `From` 구현도 alloc에 있습니다.
            let qty: u32 = qty.parse()?;
            *totals.entry(name.to_string()).or_insert(0) += qty;
        }
        Ok(totals)
    }

    /// 보고서 줄을 만듭니다.
    pub fn report(totals: &BTreeMap<String, u32>) -> Vec<String> {
        let mut lines = vec![format!("품목 {}개", totals.len())];
        lines.extend(totals.iter().map(|(name, qty)| format!("  {name}: {qty}")));
        lines
    }
}

fn main() {
    // `println!`도 std의 매크로라서 프렐류드에 없습니다. 경로를 붙여 부릅니다.
    for input in ["사과 3\n배 5\n사과 2", "사과 3\n귤", "배 다섯"] {
        match portable::tally(input) {
            Ok(totals) => {
                for line in portable::report(&totals) {
                    std::println!("{line}");
                }
            }
            Err(e) => std::println!("에러: {e}"),
        }
    }
    // 출력: 품목 2개
    // 출력:   배: 5
    // 출력:   사과: 5
    // 출력: 에러: 2번째 줄 "귤"은 '이름 수량' 모양이 아닙니다
    // 출력: 에러: invalid digit found in string
}
