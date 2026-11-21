// 기대하는 에러: `unsafe_code` 린트를 forbid로 올린 크레이트에서 `unsafe` 블록 사용(에러 코드 없음)
// 핵심 메시지: usage of an `unsafe` block
//
// `#![forbid(unsafe_code)]`를 크레이트 맨 위에 두면 그 크레이트 안의 `unsafe` 블록·함수·구현이
// 모두 컴파일 에러가 됩니다. `forbid`는 `deny`와 달리 안쪽에서 `#[allow]`로 다시 풀 수도 없습니다.
// "이 크레이트에는 unsafe 코드가 없다"를 컴파일러가 보증하게 하는 방법입니다.
//
// 고치는 법: `unsafe` 대신 안전한 API를 씁니다(여기서는 `values[1]`이나 `values.get(1)`).
// 정말 `unsafe`가 필요하면 그 코드를 따로 작은 크레이트나 모듈로 떼어 내고, 그곳만 검토합니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch3-safety-limits compile_fail/forbid_unsafe_code.rs

#![forbid(unsafe_code)]

fn main() {
    let values = [1, 2, 3];
    // SAFETY: 1은 길이 3인 배열의 범위 안입니다.
    let v = unsafe { *values.get_unchecked(1) };
    println!("{v}");
}
