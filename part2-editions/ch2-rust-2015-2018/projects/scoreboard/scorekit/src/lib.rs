//! 점수 문자열을 읽고 막대를 그려 주는 작은 라이브러리입니다.
//!
//! 2015 에디션으로 작성한 "옛 크레이트"라는 설정입니다. app2015와 app2018이 이 크레이트를
//! 함께 씁니다. 에디션은 크레이트마다 따로 정하므로, 2018 에디션 크레이트(app2018)도
//! 2015 에디션 크레이트(scorekit)에 의존할 수 있습니다.

use std::fmt;

/// 점수로 읽을 수 없는 문자열을 만났을 때의 에러입니다.
#[derive(Debug)]
pub struct ScoreError {
    pub text: String,
}

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "점수가 아닙니다: {:?}", self.text)
    }
}

/// 문자열을 0~100의 점수로 읽어 봅니다(try).
///
/// 2015 에디션에서 `try`는 키워드가 아니므로 함수 이름으로 쓸 수 있습니다.
/// 2018 이후 에디션의 크레이트에서는 `try`가 예약 키워드라서 `scorekit::try`라고 쓰면
/// 파싱 에러가 나고, `scorekit::r#try`처럼 raw 식별자로 불러야 합니다.
pub fn try(text: &str) -> Result<u32, ScoreError> {
    match text.trim().parse::<u32>() {
        Ok(n) if n <= 100 => Ok(n),
        _ => Err(ScoreError {
            text: text.to_string(),
        }),
    }
}

/// 점수 막대를 그리는 매크로입니다. 10점마다 한 칸입니다.
///
/// - `bar!(점수)`: `#`으로 그립니다.
/// - `bar!(점수, 문자)`: 그 문자로 그립니다.
///
/// `#[macro_export]`는 이 매크로를 크레이트 루트(`scorekit::bar`)로 내보냅니다.
/// 2015 에디션의 사용자는 `#[macro_use] extern crate scorekit;`로,
/// 2018 이후 에디션의 사용자는 `use scorekit::bar;`로 가져옵니다.
#[macro_export]
macro_rules! bar {
    ($score:expr) => {
        $crate::bar_with($score, '#')
    };
    ($score:expr, $ch:expr) => {
        $crate::bar_with($score, $ch)
    };
}

/// `bar!`가 펼쳐진 코드가 부르는 함수입니다.
///
/// 매크로는 그것을 부른 크레이트 안에서 펼쳐집니다. 그 자리에서도 이 함수를 찾을 수 있도록
/// 매크로 안에서는 `$crate::bar_with`로 가리킵니다. `$crate`는 "이 매크로를 정의한 크레이트"를
/// 뜻하며 모든 에디션에서 같습니다.
#[doc(hidden)]
pub fn bar_with(score: u32, ch: char) -> String {
    ch.to_string().repeat((score / 10) as usize)
}
