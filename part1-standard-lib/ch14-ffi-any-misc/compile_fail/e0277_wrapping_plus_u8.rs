// 기대하는 에러: E0277
// 핵심 메시지: cannot add `u8` to `Wrapping<u8>`
//
// Wrapping<u8>의 + 는 Wrapping<u8>끼리만 구현되어 있습니다(Add<Wrapping<u8>>).
// 감싸지 않은 u8과 더하면 결과를 래핑으로 계산할지 보통 덧셈으로 계산할지 모호하므로 구현되어 있지 않습니다.
// 복합 대입 hash += 10u8 은 됩니다(AddAssign<u8> for Wrapping<u8>, 1.60.0).
//
// 확인: rustc --edition 2024 compile_fail/e0277_wrapping_plus_u8.rs --out-dir /tmp/rust-p1-ch14

use std::num::Wrapping;

fn main() {
    let hash = Wrapping(250u8);
    let next = hash + 10u8; // → E0277
    println!("{next}");
}
