// 2024 에디션에서 블록의 마지막 표현식(tail expression)이 만든 임시값이 그 블록의
// 지역 변수보다 먼저 버려지는 것과, 그 덕분에 되는 코드, 그리고 범위가 좁아져서
// 고쳐야 하는 코드를 보여 줍니다.
//
// 실행: cargo run -p p2-ch4-rust-2024-language --example 04_tail_expr_scope
//
// 같은 코드를 2021 에디션으로 컴파일한 결과와 비교하려면
// projects/edition_compare의 tail_expr_scope_2021 / tail_expr_scope_2024를 실행합니다.

use std::cell::RefCell;

/// 버려질 때 자기 이름을 출력하는 값입니다.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.0);
    }
}

impl Noisy {
    fn char_count(&self) -> usize {
        self.0.chars().count()
    }
}

/// 지역 변수 하나와, 마지막 표현식의 임시값 하나를 만듭니다.
fn from_function() -> usize {
    let _local = Noisy("지역 변수");
    Noisy("마지막 표현식의 임시값").char_count()
}

/// 지역 `RefCell`의 길이를 마지막 표현식에서 돌려줍니다.
/// 2021 에디션에서는 E0597이었습니다(compile_fail/e0597_tail_expr_borrow_2021.rs).
fn local_refcell_len() -> usize {
    let names = RefCell::new(vec!["가", "나", "다"]);
    names.borrow().len()
}

fn main() {
    println!("1. 마지막 표현식의 임시값이 지역 변수보다 먼저 버려집니다");
    let n = from_function();
    println!("    돌려받은 값: {n}");
    // 출력:     [drop] 마지막 표현식의 임시값
    // 출력:     [drop] 지역 변수
    // 출력:     돌려받은 값: 12

    println!("2. 그래서 지역 RefCell을 빌린 가드를 마지막 표현식에 둘 수 있습니다");
    println!("    local_refcell_len() = {}", local_refcell_len());
    // 출력:     local_refcell_len() = 3

    println!("3. 범위가 좁아져서 고쳐야 하는 코드");
    // `let len = { &String::from("1234") }.len();`은 2024 에디션에서 E0716입니다.
    // 블록이 끝날 때 임시 String이 버려지는데, 블록 밖의 .len()이 아직 그것을 빌리기 때문입니다
    // (compile_fail/e0716_tail_expr_narrowed.rs). 블록의 결과를 let으로 묶으면 됩니다.
    let text = { &String::from("1234") };
    let len = text.len();
    println!("    text = {text:?}, len = {len}");
    // 출력:     text = "1234", len = 4
    // `let text = &임시값;` 모양에서는 임시값의 수명이 변수 text만큼 늘어납니다
    // (임시값 수명 연장, temporary lifetime extension). 블록의 마지막 표현식이 `&임시값`일 때도
    // 같은 규칙이 적용되어 String이 text와 함께 살아 있습니다.
}
