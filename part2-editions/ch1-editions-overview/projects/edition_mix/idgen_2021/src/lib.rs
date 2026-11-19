//! 2021 에디션으로 작성된 ID 생성 라이브러리입니다.
//!
//! 2021 에디션에서 `gen`은 평범한 식별자입니다(2024 에디션부터 예약 키워드입니다).

/// 접두사와 번호로 ID를 만듭니다. 번호가 `u16`의 범위를 넘으면 `None`입니다.
pub fn gen(prefix: &str, n: u32) -> Option<String> {
    // TryFrom은 2021 에디션의 프렐류드에 들어 있어 use 없이 씁니다.
    // 같은 줄을 2015·2018 에디션 크레이트에 쓰면 `use std::convert::TryFrom;`이 필요합니다.
    let n = u16::try_from(n).ok()?;
    Some(format!("{prefix}-{n:05}"))
}

/// 배열의 첫 원소를 `into_iter()`로 꺼냅니다. 본문은 `jobs_2015::first_item!`과 똑같습니다.
///
/// 이 `into_iter`는 2021 에디션 크레이트에서 쓴 토큰이므로 배열의 원소를 값으로 내놓습니다.
#[macro_export]
macro_rules! first_item {
    ($array:expr) => {
        $array.into_iter().next()
    };
}
