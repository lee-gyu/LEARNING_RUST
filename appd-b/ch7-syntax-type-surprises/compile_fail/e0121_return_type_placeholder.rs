// 기대하는 에러: E0121
// 핵심 메시지: the placeholder `_` is not allowed within types on item signatures for return types
//
// TypeScript는 함수의 반환 타입을 본문의 return 문으로 추론합니다. 그 습관으로 반환 타입을 `_`로 두고
// 컴파일러에게 맡기려 한 코드입니다. Rust는 함수 본문 안에서는 타입을 추론하지만, 함수 시그니처의
// 타입은 추론하지 않습니다. 컴파일러는 본문을 보고 알맞은 타입을 제안해 줍니다.
//
// 고치는 법: fn total(prices: &[u32]) -> u32 { … }
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch7-syntax-type-surprises compile_fail/e0121_return_type_placeholder.rs

fn total(prices: &[u32]) -> _ {
    prices.iter().sum::<u32>()
}

fn main() {
    println!("합계: {}", total(&[1200, 3400, 560]));
}
