// 기대하는 에러: error[E0277]: the trait bound `u8: From<i32>` is not satisfied
// 핵심 메시지: the trait `From<i32>` is not implemented for `u8`
//             = note: required for `i32` to implement `Into<u8>`
//
// into()는 From 구현이 있을 때만 생깁니다. i32 → u8은 값이 잘릴 수 있어 From이 없습니다.
// 해결: 넘칠 때의 처리를 정해 u8::try_from(total)을 씁니다(examples/02_try_from_as.rs).
//       잘림이 의도한 동작이라면 total as u8도 되지만, 그 경우 300은 44가 됩니다.
//
// 확인: rustc --edition 2024 compile_fail/e0277_into_narrowing.rs --out-dir <임시 디렉터리>

fn main() {
    let total: i32 = 300;
    let level: u8 = total.into();
    println!("{level}");
}
