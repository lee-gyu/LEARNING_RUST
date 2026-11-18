// 기대하는 에러: error[E0425]: cannot find function `swap` in this scope
//
// 원인: `std::mem` 모듈의 함수 중 `drop`, `size_of`, `size_of_val`, `align_of`, `align_of_val`은
//       프렐류드에 들어 있지만 `swap`, `replace`, `take`는 들어 있지 않습니다.
//       같은 모듈에 있다고 해서 모두 프렐류드에 있는 것은 아닙니다.
// 해결: `use std::mem;`으로 모듈을 가져와 `mem::swap(..)`으로 부르거나, `std::mem::swap(..)`처럼 경로를 적습니다.
//
// 확인: rustc --edition 2024 compile_fail/e0425_swap_not_in_prelude.rs --out-dir /tmp/rust-p1-ch1

fn main() {
    let mut a = 1;
    let mut b = 2;
    println!("size_of::<i32>() = {}", size_of::<i32>()); // 프렐류드에 있으므로 됩니다.
    swap(&mut a, &mut b); // 프렐류드에 없으므로 에러입니다.
    println!("a = {a}, b = {b}");
}
