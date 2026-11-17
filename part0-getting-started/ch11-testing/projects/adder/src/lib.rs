// 라이브러리 크레이트 adder의 루트입니다 (src/lib.rs).
// 덧셈 함수 몇 개와, 그 함수들을 검사하는 단위 테스트(맨 아래 tests 모듈)가 함께 들어 있습니다.
// 통합 테스트는 tests/ 디렉터리에 따로 있습니다.
//
// `///`로 시작하는 줄은 문서 주석(documentation comment)입니다. cargo doc이 만드는 API 문서에 들어가고,
// 문서 주석 안의 ``` 코드 블록은 cargo test가 문서 테스트로 실제로 컴파일하고 실행합니다.

use std::num::ParseIntError;

/// 두 수를 더합니다.
///
/// ```
/// let sum = adder::add(2, 3);
/// assert_eq!(sum, 5);
/// ```
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

/// 2를 더합니다.
pub fn add_two(a: u64) -> u64 {
    internal_adder(a, 2)
}

// 비공개 함수입니다. 바깥 크레이트(통합 테스트, src/main.rs)에서는 부를 수 없지만,
// 같은 파일의 tests 모듈은 이 모듈의 자식이라서 부를 수 있습니다.
fn internal_adder(left: u64, right: u64) -> u64 {
    left + right
}

/// 목록의 수를 모두 더합니다. 합계가 u64의 범위를 넘으면 패닉합니다.
pub fn sum(numbers: &[u64]) -> u64 {
    let mut total: u64 = 0;
    for n in numbers {
        // checked_add는 오버플로가 나면 None을 돌려줍니다.
        // 디버그 빌드든 릴리스 빌드든 같은 메시지로 패닉하도록 직접 검사합니다.
        total = total
            .checked_add(*n)
            .expect("합계가 u64의 범위를 넘었습니다");
    }
    total
}

/// "1 + 2 + 3"처럼 +로 이은 식을 계산합니다. 숫자가 아닌 부분이 있으면 에러를 돌려줍니다.
pub fn parse_sum(expr: &str) -> Result<u64, ParseIntError> {
    let mut numbers: Vec<u64> = Vec::new();
    for part in expr.split('+') {
        numbers.push(part.trim().parse()?);
    }
    Ok(sum(&numbers))
}

// 단위 테스트: 테스트할 코드와 같은 파일에 tests 모듈을 두는 것이 관례입니다.
// #[cfg(test)] 덕분에 cargo build로 만드는 라이브러리에는 이 모듈이 들어가지 않습니다.
#[cfg(test)]
mod tests {
    // 부모 모듈(크레이트 루트)의 항목을 모두 가져옵니다. 비공개 항목도 포함됩니다.
    use super::*;

    #[test]
    fn add_works() {
        assert_eq!(add(2, 2), 4);
    }

    // 비공개 함수 테스트: 자식 모듈은 조상의 비공개 항목을 쓸 수 있다는 ch7의 규칙 그대로입니다.
    #[test]
    fn internal() {
        assert_eq!(internal_adder(2, 2), 4);
    }

    #[test]
    fn sum_of_empty_is_zero() {
        assert_eq!(sum(&[]), 0);
    }

    #[test]
    #[should_panic(expected = "범위를 넘었습니다")]
    fn sum_overflow_panics() {
        sum(&[u64::MAX, 1]);
    }

    #[test]
    fn parse_sum_works() -> Result<(), ParseIntError> {
        assert_eq!(parse_sum("1 + 2 + 3")?, 6);
        Ok(())
    }

    #[test]
    fn parse_sum_rejects_text() {
        assert!(parse_sum("1 + 둘").is_err());
    }

    // 오래 걸리는 테스트는 #[ignore]로 평소 실행에서 빼 두고, 필요할 때 cargo test -- --ignored로 실행합니다.
    #[test]
    #[ignore = "오래 걸림: cargo test -- --ignored로 실행"]
    fn add_two_many_times() {
        let mut value = 0;
        for _ in 0..50_000_000 {
            value = add_two(value);
        }
        assert_eq!(value, 100_000_000);
    }
}
