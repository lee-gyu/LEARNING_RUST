// 기대하는 에러: error[E0308]: mismatched types — expected `u32`, found `usize`
//
// 원인: 문서의 시그니처는 `pub const fn pow(self, exp: u32) -> i32`입니다. 지수 `exp`는 밑의 타입과 상관없이
//       항상 `u32`입니다. `len()`은 `usize`를 돌려주므로 그대로 넘기면 타입이 맞지 않습니다.
//       Rust는 정수 타입 사이를 자동으로 변환하지 않습니다.
// 해결: 컴파일러의 help처럼 `.try_into().unwrap()`으로 변환합니다(넘치면 패닉).
//       실패를 직접 처리하려면 `u32::try_from(digits.len())`의 `Result`를 `match`로 다룹니다.
//       길이가 `u32` 범위를 넘지 않는다고 확신하면 `as u32`도 됩니다.
//
// 확인: rustc --edition 2024 compile_fail/e0308_pow_exponent_usize.rs --out-dir /tmp/rust-p1-ch1

fn main() {
    let digits = [1, 0, 0];
    let scale = 10_i32.pow(digits.len());
    println!("{scale}");
}
