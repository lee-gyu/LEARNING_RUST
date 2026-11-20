// 기대하는 에러(에러 코드 없음): functions used as tests can not have any arguments
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --test --out-dir <임시 디렉터리> compile_fail/test_fn_with_arguments.rs
//
// 여러 입력으로 같은 성질을 확인하고 싶어서 테스트 함수가 입력을 받게 쓴 코드입니다.
// 테스트 러너는 테스트 함수를 인자 없이 부르므로, 인자를 받는 `#[test]` 함수는 컴파일되지 않습니다.
// 입력은 테스트 함수 안에서 만들어 반복문으로 넣습니다(projects/tinytlv/tests/arbitrary_bytes.rs).
// `proptest` 같은 크레이트는 매크로로 이 반복문을 대신 만들어 줍니다.

pub fn parse_digit(b: u8) -> Option<u8> {
    b.is_ascii_digit().then(|| b - b'0')
}

#[test]
fn never_panics(input: u8) {
    let _ = parse_digit(input);
}
