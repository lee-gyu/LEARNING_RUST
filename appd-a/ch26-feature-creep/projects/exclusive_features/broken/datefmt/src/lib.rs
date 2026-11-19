//! 비권장: 서로 함께 켤 수 없는 기능을 가진 날짜 서식 라이브러리.
//!
//! `iso`와 `korean`은 같은 이름의 함수 `render`를 서로 다르게 정의합니다. 이 크레이트만 보면
//! "둘 중 하나를 고르는 스위치"처럼 보이지만, Cargo는 빌드 그래프 어디선가 요청된 기능을 모두
//! 합쳐서(기능 통합) 이 크레이트를 한 번만 빌드합니다. 한 크레이트가 `iso`를, 다른 크레이트가
//! `korean`을 요청하면 두 기능이 함께 켜집니다.

/// 연, 월, 일. 이 타입은 기능과 상관없이 늘 있습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

// Cargo Book이 "어쩔 수 없이 배타적인 기능을 둔다면" 권하는 안전장치입니다. 두 기능이 함께
// 켜지면 무엇이 잘못됐는지 알려 주는 메시지로 컴파일을 멈춥니다. 메시지가 친절해질 뿐,
// 이 에러를 받은 쪽(app)이 고칠 방법은 여전히 없습니다.
#[cfg(all(feature = "iso", feature = "korean"))]
compile_error!("datefmt: `iso`와 `korean` 기능은 함께 켤 수 없습니다");

/// 날짜를 ISO 8601 모양으로 씁니다.
#[cfg(feature = "iso")]
pub fn render(d: &Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

/// 날짜를 한국어 모양으로 씁니다.
#[cfg(feature = "korean")]
pub fn render(d: &Date) -> String {
    format!("{}년 {}월 {}일", d.year, d.month, d.day)
}
