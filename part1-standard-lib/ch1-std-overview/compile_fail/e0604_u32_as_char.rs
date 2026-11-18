// 기대하는 에러: error[E0604]: only `u8` can be cast as `char`, not `u32`
//
// 원인: `u32`의 모든 값이 올바른 `char`(유니코드 스칼라 값)인 것은 아닙니다. 0xD800~0xDFFF(서로게이트)와
//       0x10FFFF보다 큰 값은 문자가 될 수 없습니다. 실패할 수 없는 `as`로는 이 변환을 허용하지 않습니다.
//       0~255는 모두 올바른 문자이므로 `u8 as char`만 허용됩니다.
// 해결: `char::from_u32(code)`를 쓰고, 돌려받은 `Option<char>`를 `match`나 `if let`으로 처리합니다.
//
// 확인: rustc --edition 2024 compile_fail/e0604_u32_as_char.rs --out-dir /tmp/rust-p1-ch1

fn main() {
    let code: u32 = 44032; // '가'의 코드 포인트
    let c = code as char;
    println!("{c}");
}
