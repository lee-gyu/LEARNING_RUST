// 기대하는 에러: E0308 mismatched types (expected `usize`, found `u32`)
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0308_define_is_u32.rs
//
// 손으로 쓴 선언 위의 래퍼를 생성한 선언 위로 옮기면, 래퍼의 몇 줄은 타입이 맞지 않아
// 컴파일되지 않습니다. bindgen은 `#define TALLY_NAME_MAX 31` 같은 정수 매크로를 기본으로
// `u32` 상수로 옮기는데, 손으로 쓴 선언(ch34)은 길이로 쓰기 좋게 `usize`로 적었기 때문입니다.
// 한 파일로 줄이려고 `tally_sys` 크레이트를 모듈로 흉내 냈습니다.

mod tally_sys {
    // bindgen 0.73.2가 v1 헤더에서 생성한 줄 그대로입니다.
    pub const TALLY_NAME_MAX: u32 = 31;
}

use tally_sys as ffi;

/// 손으로 쓴 판의 래퍼에 있던 줄을 그대로 옮겼습니다.
pub const NAME_MAX: usize = ffi::TALLY_NAME_MAX;

fn main() {
    println!("이름은 최대 {NAME_MAX}바이트");
}
