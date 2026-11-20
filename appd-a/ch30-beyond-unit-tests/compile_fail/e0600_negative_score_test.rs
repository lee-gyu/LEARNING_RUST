// 기대하는 에러: E0600 cannot apply unary operator `-` to type `u8`
//   note: unsigned values cannot be negated
//   help: you may have meant the maximum value of `u8`
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --test --out-dir <임시 디렉터리> compile_fail/e0600_negative_score_test.rs
//
// `--test` 없이 컴파일하면 `#[cfg(test)]` 모듈이 통째로 빠지므로 에러 없이 컴파일됩니다.
// 테스트 코드의 에러는 테스트 빌드(`cargo test`, `rustc --test`)에서만 드러납니다.
//
// "음수 점수를 넣으면 거부하는가"를 확인하려는 테스트입니다. `grade`가 `u8`을 받으므로
// 음수는 애초에 넘길 수 없고, 컴파일러가 그 테스트 자체를 거부합니다. 타입이 이미 보장하는
// 것이니 테스트할 일이 없습니다(examples/01_what_types_guarantee.rs). 대신 `u8`이 막아 주지 않는
// 101~255점을 테스트합니다. help가 제안하는 `u8::MAX`(255)가 마침 그런 입력입니다.

pub fn grade(score: u8) -> Result<char, u8> {
    match score {
        60..=100 => Ok('P'),
        0..=59 => Ok('F'),
        _ => Err(score),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_negative_score() {
        assert!(grade(-1).is_err());
    }
}
