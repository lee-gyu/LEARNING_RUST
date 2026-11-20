// 기대하는 에러: E0599 no method named `sqrt` found for type `f64` in the current scope
//   method not found in `f64`
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0599_f64_sqrt_in_no_std.rs
//
// `f64`는 core의 원시 타입이지만 `sqrt`, `floor`, `round`, `powi`, `sin` 같은 메서드는 std가 더한
// `impl` 블록에 있습니다(1.100.0 로컬 문서의 core판 `f64` 페이지에는 `abs`, `clamp`, `total_cmp` 등만
// 있음). 같은 일을 하는 `core::f64::math`는 nightly 전용(`core_float_math`)입니다. stable의 `no_std`
// 라이브러리라면 순수 Rust로 수학 함수를 구현한 `libm` 같은 크레이트를 쓰거나, 정수 연산으로 바꿉니다.
// 컴파일러는 이 메서드가 std에 있다고 알려 주지 않으므로, std 크레이트에서 옮겨 온 코드에서 특히 헷갈립니다.

#![no_std]

/// 가속도 세 축의 크기를 구합니다.
pub fn magnitude(x: f64, y: f64, z: f64) -> f64 {
    (x * x + y * y + z * z).sqrt()
}
