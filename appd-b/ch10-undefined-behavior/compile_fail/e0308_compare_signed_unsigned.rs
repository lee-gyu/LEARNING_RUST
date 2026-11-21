// 기대하는 에러: E0308
// 핵심 메시지: mismatched types — expected `i32`, found `u32`
//
// C에서 int와 unsigned를 비교하면 int 쪽이 unsigned로 바뀌어 -1 < 1u가 거짓이 됩니다(compare/04_sign_compare.c).
// clang은 -Wextra를 켜야 -Wsign-compare로 경고합니다. Rust는 부호가 다른 두 정수를 비교 연산자로 바로 비교하지
// 않고, 어느 쪽으로 바꿀지 적으라고 요구합니다.
//
// 고치는 법: 둘 다 담을 수 있는 타입으로 넓혀 비교합니다.
//   if i64::from(balance) < i64::from(limit) { … }
// 컴파일러의 help는 limit.try_into().unwrap()(u32를 i32로, 안 들어가면 패닉)을 제안하는데, 이 경우 limit가 i32::MAX보다
// 크면 패닉하므로 넓히는 쪽이 낫습니다(examples/02_conversions_and_bits.rs의 2번).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch10-undefined-behavior compile_fail/e0308_compare_signed_unsigned.rs

fn main() {
    let balance: i32 = -1;
    let limit: u32 = 1;
    if balance < limit {
        // E0308
        println!("한도 안");
    }
}
