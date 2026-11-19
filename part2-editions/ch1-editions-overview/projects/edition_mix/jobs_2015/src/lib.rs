//! 2015 에디션으로 작성된 작은 "작업 예약" 라이브러리입니다.
//!
//! 2015 에디션에서 `async`와 `await`는 키워드가 아닙니다(2018 에디션부터 키워드입니다).
//! 그래서 이 크레이트는 두 이름을 평범한 함수 이름으로 씁니다. 에디션이 다른 크레이트가
//! 이 함수들을 어떻게 부르는지는 `app_2024/src/main.rs`에 있습니다.

/// 예약된 작업입니다.
///
/// 구조체 같은 타입은 에디션과 상관없이 크레이트 사이를 오갑니다.
/// 2024 에디션 크레이트도 이 타입의 값을 그대로 받아 메서드를 부릅니다.
pub struct Job {
    pub name: String,
    pub steps: u32,
    paused: bool,
}

/// 작업을 예약합니다. 2015 에디션에서 `async`는 평범한 식별자입니다.
pub fn async(name: &str, steps: u32) -> Job {
    Job {
        name: name.to_string(),
        steps,
        paused: false,
    }
}

/// 작업이 끝날 때까지 기다린 것으로 치고 결과 문자열을 돌려줍니다.
pub fn await(job: Job) -> String {
    let state = if job.paused { "일시정지" } else { "완료" };
    // 서식 문자열에 변수 이름을 바로 쓰는 문법(1.58)도 에디션과 상관없이 쓸 수 있습니다.
    format!("{}: {}단계 {state}", job.name, job.steps)
}

impl Job {
    /// 일시정지 상태를 뒤집습니다.
    ///
    /// `bool::toggle`은 1.100.0에서 안정화된 메서드입니다. 표준 라이브러리는 모든 에디션이
    /// 함께 쓰므로 2015 에디션 크레이트에서도 쓸 수 있습니다. 대신 이 크레이트를 컴파일하려면
    /// rustc 1.100.0 이상이 필요하고, 그래서 Cargo.toml에 `rust-version = "1.100"`을 적었습니다.
    pub fn toggle_pause(&mut self) {
        self.paused.toggle();
    }
}

/// 이름의 첫 글자를 돌려줍니다. 빈 문자열이면 `'?'`입니다.
///
/// let-else(1.65)처럼 새로 들어온 문법도, 기존 코드를 깨뜨리지 않는 것이라면
/// 2015 에디션에서 그대로 쓸 수 있습니다.
pub fn initial(name: &str) -> char {
    let Some(first) = name.chars().next() else {
        return '?';
    };
    first
}

/// 작업 하나를 예약하고 곧바로 기다립니다.
///
/// 매크로 본문의 `async`, `await`는 이 크레이트에서 쓴 토큰이므로 2015 에디션의 규칙으로
/// 읽힙니다. 그래서 2024 에디션 크레이트에서 이 매크로를 불러도 식별자로 해석됩니다.
#[macro_export]
macro_rules! run_now {
    ($name:expr) => {{
        let async = $crate::async($name, 1);
        $crate::await(async)
    }};
}

/// 배열의 첫 원소를 `into_iter()`로 꺼냅니다.
///
/// 2015·2018 에디션에서 배열에 메서드 호출 문법으로 `.into_iter()`를 부르면 참조를
/// 내놓습니다(2021 에디션부터 값). 아래 `into_iter`는 이 크레이트에서 쓴 토큰이므로,
/// 어느 크레이트에서 펼쳐지든 2015 에디션의 규칙을 따릅니다.
#[macro_export]
macro_rules! first_item {
    ($array:expr) => {
        $array.into_iter().next()
    };
}
