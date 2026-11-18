// 기대하는 에러: E0080
// 핵심 메시지: evaluation panicked: called `Option::unwrap()` on a `None` value
//
// const의 값은 컴파일할 때 계산됩니다. NonZero::new(0)은 None이므로 unwrap이 컴파일 중에 패닉하고,
// 그 패닉이 실행 중 패닉 대신 컴파일 에러로 나타납니다. "0이 아님"을 실행 전에 확인받는 방법입니다.
//
// 확인: rustc --edition 2024 compile_fail/e0080_const_nonzero_zero.rs --out-dir /tmp/rust-p1-ch14

use std::num::NonZero;

const WORKERS: NonZero<u32> = NonZero::new(0).unwrap(); // → E0080

fn main() {
    println!("작업자 {WORKERS}명");
}
