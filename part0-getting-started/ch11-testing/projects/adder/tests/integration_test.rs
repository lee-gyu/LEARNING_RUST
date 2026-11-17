// 통합 테스트 파일입니다. tests/ 아래의 파일은 각각 별도의 크레이트로 컴파일됩니다.
// 그래서 바깥 사용자와 똑같이 라이브러리를 크레이트 이름 adder로 가져오고, 공개 API만 쓸 수 있습니다.
// tests/ 디렉터리는 cargo test를 실행할 때만 컴파일하므로 #[cfg(test)]를 붙이지 않습니다.

use adder::{add_two, sum};

// tests/common/mod.rs의 공용 도우미를 이 크레이트의 모듈로 등록합니다.
mod common;

#[test]
fn it_adds_two() {
    assert_eq!(add_two(2), 4);
}

#[test]
fn sum_one_to_ten() {
    let numbers = common::one_to(10);
    assert_eq!(sum(&numbers), 55);
}
