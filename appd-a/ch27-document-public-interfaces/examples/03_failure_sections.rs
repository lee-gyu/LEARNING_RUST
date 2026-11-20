//! 지침 3: 실패하는 조건은 `# Errors`·`# Panics`·`# Safety` 절에 씁니다.
//!
//! 실행: cargo run -p er-ch27-document-public-interfaces --example 03_failure_sections
//! 문서: cargo doc -p er-ch27-document-public-interfaces --example 03_failure_sections --no-deps
//!
//! 쪽 범위("3-7")를 해석하고 쪽 내용을 꺼내는 작은 API를 두 판으로 만듭니다. 본문은 같고
//! 문서만 다릅니다.
//! - `v1`: "실패할 수 있습니다", "빠르게 가져옵니다"처럼 실패를 뭉뚱그리거나 빠뜨린 문서.
//! - `v2`: 에러마다 조건을 적은 `# Errors`, 패닉 조건과 대안을 적은 `# Panics`, 부르는 쪽이
//!   지킬 조건을 적은 `# Safety`.
//!
//! `main`은 `v2` 문서에 적힌 조건을 하나씩 실행해서, 문서가 말한 그 에러·그 패닉이 나오는지
//! 확인합니다. 패닉을 붙잡아 출력하려고 패닉 훅을 바꾸므로 표준 에러에는 아무것도 나오지 않습니다.

use std::ops::RangeInclusive;
use std::panic;

/// 쪽 범위 문자열을 해석하지 못한 이유입니다. 두 판이 함께 씁니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeError {
    /// 공백뿐인 문자열입니다.
    Empty,
    /// 숫자로 읽을 수 없는 부분이 있습니다. 그 부분을 담습니다.
    NotANumber(String),
    /// 시작 쪽이 끝 쪽보다 뒤에 있습니다.
    Reversed {
        /// 시작 쪽
        start: usize,
        /// 끝 쪽
        end: usize,
    },
    /// 0쪽이나, 문서에 없는 쪽을 가리킵니다.
    OutOfDocument {
        /// 문서에 없는 쪽
        page: usize,
        /// 문서의 쪽 수
        page_count: usize,
    },
}

/// 비권장: 실패를 뭉뚱그리거나 빠뜨린 문서.
pub mod v1 {
    use super::RangeError;
    use std::ops::RangeInclusive;

    /// 쪽으로 나뉜 문서.
    pub struct Document {
        pages: Vec<String>,
    }

    #[expect(
        clippy::missing_errors_doc,
        clippy::missing_panics_doc,
        clippy::missing_safety_doc,
        reason = "비권장 판: 실패 조건을 적는 절이 없는 문서를 보여 줍니다"
    )]
    impl Document {
        /// 문서를 만듭니다.
        pub fn new(pages: &[&str]) -> Document {
            Document {
                pages: pages.iter().map(|p| p.to_string()).collect(),
            }
        }

        /// 쪽 범위를 파싱합니다. 잘못된 입력이면 실패할 수 있습니다.
        pub fn parse_range(&self, spec: &str) -> Result<RangeInclusive<usize>, RangeError> {
            let spec = spec.trim();
            if spec.is_empty() {
                return Err(RangeError::Empty);
            }
            let (a, b) = spec.split_once('-').unwrap_or((spec, spec));
            let num = |s: &str| {
                let s = s.trim();
                s.parse::<usize>()
                    .map_err(|_| RangeError::NotANumber(s.to_string()))
            };
            let (start, end) = (num(a)?, num(b)?);
            if start > end {
                return Err(RangeError::Reversed { start, end });
            }
            let page_count = self.pages.len();
            for page in [start, end] {
                if page == 0 || page > page_count {
                    return Err(RangeError::OutOfDocument { page, page_count });
                }
            }
            Ok(start..=end)
        }

        /// 쪽을 가져옵니다.
        pub fn page(&self, n: usize) -> &str {
            let count = self.pages.len();
            assert!(n >= 1 && n <= count, "{n}쪽은 없습니다 (1..={count})");
            &self.pages[n - 1]
        }

        /// 쪽을 빠르게 가져옵니다.
        pub unsafe fn page_unchecked(&self, n: usize) -> &str {
            // SAFETY: 부르는 쪽이 1 <= n <= self.pages.len()을 보장한다고 가정합니다. 그런데 v1의
            // 문서는 이 조건을 어디에도 적지 않았으므로, 부르는 쪽은 무엇을 보장해야 하는지 모릅니다.
            unsafe { self.pages.get_unchecked(n - 1) }
        }
    }
}

/// 권장: 실패 조건을 정해진 절에 적은 문서.
pub mod v2 {
    use super::RangeError;
    use std::ops::RangeInclusive;

    /// 쪽으로 나뉜 문서. 쪽 번호는 1부터 셉니다.
    pub struct Document {
        pages: Vec<String>,
    }

    impl Document {
        /// `pages`의 첫 문자열을 1쪽으로 하는 문서를 만듭니다.
        pub fn new(pages: &[&str]) -> Document {
            Document {
                pages: pages.iter().map(|p| p.to_string()).collect(),
            }
        }

        /// 쪽 수를 돌려줍니다.
        pub fn page_count(&self) -> usize {
            self.pages.len()
        }

        /// `"3-7"`이나 `"4"` 같은 쪽 범위를 해석합니다. 범위는 양 끝을 포함하고, 숫자 앞뒤의
        /// 공백은 무시합니다.
        ///
        /// # Errors
        ///
        /// - 공백뿐이면 [`RangeError::Empty`]
        /// - `-`의 앞이나 뒤가 0 이상의 정수가 아니면 [`RangeError::NotANumber`]
        /// - 시작이 끝보다 크면 [`RangeError::Reversed`]
        /// - 0쪽이나 [`page_count`](Self::page_count)보다 큰 쪽이 있으면
        ///   [`RangeError::OutOfDocument`]
        pub fn parse_range(&self, spec: &str) -> Result<RangeInclusive<usize>, RangeError> {
            let spec = spec.trim();
            if spec.is_empty() {
                return Err(RangeError::Empty);
            }
            let (a, b) = spec.split_once('-').unwrap_or((spec, spec));
            let num = |s: &str| {
                let s = s.trim();
                s.parse::<usize>()
                    .map_err(|_| RangeError::NotANumber(s.to_string()))
            };
            let (start, end) = (num(a)?, num(b)?);
            if start > end {
                return Err(RangeError::Reversed { start, end });
            }
            let page_count = self.pages.len();
            for page in [start, end] {
                if page == 0 || page > page_count {
                    return Err(RangeError::OutOfDocument { page, page_count });
                }
            }
            Ok(start..=end)
        }

        /// `n`쪽의 내용을 돌려줍니다.
        ///
        /// # Panics
        ///
        /// `n`이 0이거나 [`page_count`](Self::page_count)보다 크면 패닉합니다. `n`이 사용자
        /// 입력처럼 바깥에서 왔다면 [`parse_range`](Self::parse_range)로 먼저 확인합니다.
        pub fn page(&self, n: usize) -> &str {
            let count = self.pages.len();
            assert!(n >= 1 && n <= count, "{n}쪽은 없습니다 (1..={count})");
            &self.pages[n - 1]
        }

        /// `n`쪽의 내용을 범위 검사 없이 돌려줍니다.
        ///
        /// # Safety
        ///
        /// `1 <= n <= self.page_count()`이어야 합니다. 어기면 미정의 동작입니다.
        pub unsafe fn page_unchecked(&self, n: usize) -> &str {
            // SAFETY: 부르는 쪽이 문서의 조건대로 1 <= n <= self.pages.len()을 보장하므로,
            // n - 1은 넘치지 않고 pages의 범위 안입니다.
            unsafe { self.pages.get_unchecked(n - 1) }
        }
    }
}

/// 클로저를 실행해 패닉하면 그 메시지를, 아니면 결과를 디버그 형식으로 돌려줍니다.
fn run<T: std::fmt::Debug>(f: impl FnOnce() -> T + panic::UnwindSafe) -> String {
    match panic::catch_unwind(f) {
        Ok(v) => format!("{v:?}"),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            format!("패닉: {msg}")
        }
    }
}

const PAGES: [&str; 10] = [
    "표지",
    "차례",
    "1장",
    "2장",
    "3장",
    "4장",
    "5장",
    "부록",
    "찾아보기",
    "뒤표지",
];

type Check = fn(&Result<RangeInclusive<usize>, RangeError>) -> bool;

fn main() {
    panic::set_hook(Box::new(|_| {}));
    let doc = v2::Document::new(&PAGES);

    println!(
        "1. # Errors: 문서의 조건마다 그 에러가 나오는지 ({}쪽 문서)",
        doc.page_count()
    );
    // 출력: 1. # Errors: 문서의 조건마다 그 에러가 나오는지 (10쪽 문서)
    // (입력, 문서가 말한 결과) — 문서의 목록을 그대로 옮긴 표입니다.
    let cases: [(&str, Check); 7] = [
        ("3-7", |r| r == &Ok(3..=7)),
        ("4", |r| r == &Ok(4..=4)),
        ("  ", |r| matches!(r, Err(RangeError::Empty))),
        ("3-x", |r| matches!(r, Err(RangeError::NotANumber(_)))),
        ("7-3", |r| matches!(r, Err(RangeError::Reversed { .. }))),
        ("0-2", |r| {
            matches!(r, Err(RangeError::OutOfDocument { .. }))
        }),
        ("5-12", |r| {
            matches!(r, Err(RangeError::OutOfDocument { .. }))
        }),
    ];
    let mut all_match = true;
    for (input, documented) in cases {
        let result = doc.parse_range(input);
        all_match &= documented(&result);
        println!("  {input:?} → {result:?}");
    }
    // 출력:   "3-7" → Ok(3..=7)
    // 출력:   "4" → Ok(4..=4)
    // 출력:   "  " → Err(Empty)
    // 출력:   "3-x" → Err(NotANumber("x"))
    // 출력:   "7-3" → Err(Reversed { start: 7, end: 3 })
    // 출력:   "0-2" → Err(OutOfDocument { page: 0, page_count: 10 })
    // 출력:   "5-12" → Err(OutOfDocument { page: 12, page_count: 10 })
    println!("  모든 결과가 문서의 목록과 일치: {all_match}");
    // 출력:   모든 결과가 문서의 목록과 일치: true

    println!("2. # Panics: 문서의 조건에서만 패닉하는지");
    // 출력: 2. # Panics: 문서의 조건에서만 패닉하는지
    for n in [1, 10, 0, 11] {
        println!("  page({n}) → {}", run(|| doc.page(n)));
    }
    // 출력:   page(1) → "표지"
    // 출력:   page(10) → "뒤표지"
    // 출력:   page(0) → 패닉: 0쪽은 없습니다 (1..=10)
    // 출력:   page(11) → 패닉: 11쪽은 없습니다 (1..=10)

    println!("3. # Safety: 문서의 조건을 확인한 뒤에만 page_unchecked를 부름");
    // 출력: 3. # Safety: 문서의 조건을 확인한 뒤에만 page_unchecked를 부름
    for n in [2, 11] {
        if (1..=doc.page_count()).contains(&n) {
            // SAFETY: 바로 위에서 1 <= n <= doc.page_count()를 확인했습니다(# Safety의 조건).
            let text = unsafe { doc.page_unchecked(n) };
            println!("  n = {n}: 조건 확인 통과 → {text:?}");
        } else {
            println!("  n = {n}: 조건 확인 실패 → 부르지 않음");
        }
    }
    // 출력:   n = 2: 조건 확인 통과 → "차례"
    // 출력:   n = 11: 조건 확인 실패 → 부르지 않음

    println!("4. 비권장 v1: 본문이 같으니 결과도 같지만, 문서로는 무엇을 대비할지 알 수 없음");
    // 출력: 4. 비권장 v1: 본문이 같으니 결과도 같지만, 문서로는 무엇을 대비할지 알 수 없음
    let old = v1::Document::new(&PAGES);
    println!("  v1 parse_range(\"7-3\") → {:?}", old.parse_range("7-3"));
    // 출력:   v1 parse_range("7-3") → Err(Reversed { start: 7, end: 3 })
    println!("  v1 page(11) → {}", run(|| old.page(11)));
    // 출력:   v1 page(11) → 패닉: 11쪽은 없습니다 (1..=10)
}
