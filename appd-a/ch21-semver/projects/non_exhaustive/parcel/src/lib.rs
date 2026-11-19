//! 가상의 택배 라이브러리 parcel 1.1.0
//!
//! 1.0.0에서 1.1.0으로 오면서 열거형 `Status`에 배리언트 `Returned`를, 구조체 `Options`에 필드
//! `insured`를 더했습니다. 두 타입에 1.0.0부터 `#[non_exhaustive]`를 붙여 두었기 때문에 이 추가는
//! 사용자를 깨지 않는 마이너 변경입니다. 같은 속성을 1.1.0에서 처음 붙였다면 그것이 오히려 호환되지
//! 않는 변경이 됩니다(../broken_shop이 그때 깨지는 사용자 코드입니다).

/// 배송 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Status {
    Accepted,
    InTransit,
    Delivered,
    /// 1.1.0에서 더한 배리언트
    Returned,
}

/// 배송 옵션. 필드는 공개라 읽고 고칠 수 있지만, 크레이트 밖에서는 구조체 리터럴로 만들 수 없습니다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Options {
    /// 받는 사람 서명을 받을지
    pub signature: bool,
    /// 1.1.0에서 더한 필드: 파손 보험에 들지
    pub insured: bool,
}

impl Options {
    /// 크레이트 밖에서 `Options`를 만드는 길. 필드가 늘어도 이 함수의 시그니처는 그대로입니다.
    pub fn new() -> Self {
        Options::default()
    }

    pub fn signature(mut self, yes: bool) -> Self {
        self.signature = yes;
        self
    }

    /// 1.1.0에서 더한 세터
    pub fn insured(mut self, yes: bool) -> Self {
        self.insured = yes;
        self
    }
}

/// 상태의 짧은 이름.
///
/// 크레이트 안에서는 `#[non_exhaustive]`가 효과가 없으므로 `_` 갈래 없이 모든 배리언트를 나열할 수
/// 있습니다. 배리언트를 더하면 이 `match`가 E0004로 알려 주므로 라이브러리 쪽은 빠뜨리지 않습니다.
pub fn label(status: Status) -> &'static str {
    match status {
        Status::Accepted => "접수",
        Status::InTransit => "배송 중",
        Status::Delivered => "배송 완료",
        Status::Returned => "반송",
    }
}

/// 빠른 배송 옵션. 크레이트 안에서는 구조체 리터럴도 그대로 쓸 수 있습니다.
pub fn express() -> Options {
    Options {
        signature: true,
        insured: true,
    }
}

/// 송장 번호로 상태를 조회합니다(연습용 장부).
pub fn status_of(no: &str) -> Option<Status> {
    match no {
        "KR-0001" => Some(Status::Accepted),
        "KR-1203" => Some(Status::InTransit),
        "KR-0042" => Some(Status::Delivered),
        "KR-0777" => Some(Status::Returned),
        _ => None,
    }
}

/// 배송비(원): 기본 3000원, 서명 500원, 보험 1000원
pub fn fee(options: &Options) -> u32 {
    let mut won = 3000;
    if options.signature {
        won += 500;
    }
    if options.insured {
        won += 1000;
    }
    won
}
