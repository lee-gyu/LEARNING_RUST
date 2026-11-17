// 두 번째 통합 테스트 파일입니다. integration_test.rs와는 다른 크레이트이므로 cargo test 출력에 구역이 따로 생기고,
// cargo test --test parse_test로 이 파일의 테스트만 실행할 수 있습니다.
// 같은 common 모듈을 이 파일에서도 mod common;으로 선언해 씁니다(크레이트마다 한 번씩 컴파일됩니다).

use std::num::ParseIntError;

use adder::{parse_sum, sum};

mod common;

// 여러 공개 함수(parse_sum, sum)가 함께 맞게 동작하는지 확인합니다. 통합 테스트가 잘하는 일입니다.
#[test]
fn parse_sum_matches_sum() -> Result<(), ParseIntError> {
    let total = parse_sum("1 + 2 + 3 + 4")?;
    assert_eq!(total, sum(&common::one_to(4)));
    Ok(())
}

#[test]
fn parse_sum_rejects_missing_number() {
    // 마지막 + 뒤가 비어 있으므로 빈 문자열을 숫자로 바꾸려다 실패합니다.
    let result = parse_sum("1 + 2 +");
    assert!(
        result.is_err(),
        "에러가 나와야 하는데 {result:?}가 나왔습니다"
    );
}
