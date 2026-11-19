// 에디션별 결과: 2021에서 실패, 2024에서 통과
// 기대하는 에러(2021): E0597 — `names` does not live long enough
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2021 compile_fail/e0597_tail_expr_borrow_2021.rs --out-dir /tmp/rust-p2-ch4   # 실패
//   rustc --edition 2024 compile_fail/e0597_tail_expr_borrow_2021.rs --out-dir /tmp/rust-p2-ch4   # 통과
//
// 2021 에디션에서는 블록의 마지막 표현식이 만든 임시값(여기서는 `names.borrow()`가 돌려준
// `Ref` 대여 가드)이 그 블록의 지역 변수(`names`)보다 나중에 버려집니다. 가드가 버려질 때
// `names`를 다시 건드리는데, 그때는 `names`가 이미 없으므로 대여 검사기가 거부합니다.
// 2024 에디션에서는 마지막 표현식의 임시값이 지역 변수보다 먼저 버려지므로 통과합니다.
// 해결(2021에 머문다면): 컴파일러의 help대로 `let n = names.borrow().len(); n`처럼
// 값을 지역 변수에 먼저 담은 뒤 그 변수를 마지막 표현식으로 둡니다.

use std::cell::RefCell;

fn count_names() -> usize {
    let names = RefCell::new(vec!["가", "나", "다"]);
    names.borrow().len() // → 2021에서 E0597
}

fn main() {
    println!("{}", count_names());
}
