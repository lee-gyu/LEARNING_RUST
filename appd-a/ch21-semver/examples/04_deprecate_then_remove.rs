// 호환되지 않는 변경은 두 단계로 내보냅니다: 마이너에서 새 API를 더하고 옛 API에 #[deprecated]를 붙인 뒤,
// 다음 메이저에서 옛 API를 지웁니다.
//
// 실행: cargo run -p er-ch21-semver --example 04_deprecate_then_remove
//
// 가상의 택배 라이브러리 `parcel`의 배송 조회 함수를 세 버전으로 흉내 냅니다.
//   v1_0 : 1.0.0. `track(no) -> Option<Status>`. 송장 번호 형식이 틀린 것과 등록되지 않은 것이
//          모두 None이라 사용자가 둘을 구별할 수 없습니다.
//   v1_1 : 1.1.0. 둘을 구별하는 `lookup(no) -> Result<Status, LookupError>`를 더하고,
//          `track`은 `lookup`으로 다시 구현한 뒤 #[deprecated]를 붙였습니다. 옛 코드는 그대로 컴파일되고
//          경고만 받습니다.
//   v2_0 : 2.0.0. `track`을 지웠습니다. 1.1.0에서 경고를 보고 옮긴 사용자는 고칠 것이 없습니다.

/// 세 버전이 같은 배송 상태와 송장 장부를 쓰도록 바깥에 두었습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    InTransit,
    Delivered,
}

/// 송장 번호 형식: 영문 대문자 두 자 + '-' + 숫자 네 자 (예: "KR-1203")
fn well_formed(no: &str) -> bool {
    let b = no.as_bytes();
    b.len() == 7
        && b[..2].iter().all(u8::is_ascii_uppercase)
        && b[2] == b'-'
        && b[3..].iter().all(u8::is_ascii_digit)
}

fn ledger(no: &str) -> Option<Status> {
    match no {
        "KR-1203" => Some(Status::InTransit),
        "KR-0042" => Some(Status::Delivered),
        _ => None,
    }
}

/// parcel 1.0.0
mod v1_0 {
    pub use super::Status;

    /// 형식이 틀려도, 등록되지 않았어도 None입니다.
    pub fn track(no: &str) -> Option<Status> {
        if super::well_formed(no) {
            super::ledger(no)
        } else {
            None
        }
    }
}

/// parcel 1.1.0: 새 API를 더하고 옛 API는 예고만 합니다.
mod v1_1 {
    pub use super::Status;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LookupError {
        Malformed,
        NotFound,
    }

    /// 1.1.0에서 더한 함수. 실패의 이유를 구별합니다.
    pub fn lookup(no: &str) -> Result<Status, LookupError> {
        if !super::well_formed(no) {
            return Err(LookupError::Malformed);
        }
        super::ledger(no).ok_or(LookupError::NotFound)
    }

    /// 옛 함수는 지우지 않고 새 함수로 다시 구현해 동작을 그대로 지킵니다.
    /// note에는 무엇으로 옮기면 되는지와 언제 지워지는지를 적습니다.
    #[deprecated(
        since = "1.1.0",
        note = "lookup을 쓰세요. 형식 오류와 미등록을 구별합니다. 2.0.0에서 지웁니다"
    )]
    pub fn track(no: &str) -> Option<Status> {
        lookup(no).ok()
    }
}

/// parcel 2.0.0: 예고한 대로 track을 지웠습니다.
mod v2_0 {
    pub use super::v1_1::{LookupError, lookup};
}

const NUMBERS: [&str; 3] = ["KR-1203", "kr1203", "KR-9999"];

/// 1.0.0 시절에 쓴 사용자 코드
macro_rules! old_user_code {
    ($parcel:ident) => {
        NUMBERS
            .iter()
            .map(|no| format!("{no} → {:?}", $parcel::track(no)))
            .collect::<Vec<_>>()
            .join(", ")
    };
}

/// 1.1.0의 경고를 보고 옮긴 사용자 코드
macro_rules! migrated_user_code {
    ($parcel:ident) => {
        NUMBERS
            .iter()
            .map(|no| match $parcel::lookup(no) {
                Ok(status) => format!("{no} → {status:?}"),
                Err($parcel::LookupError::Malformed) => format!("{no} → 형식 오류"),
                Err($parcel::LookupError::NotFound) => format!("{no} → 미등록"),
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
}

fn main() {
    println!("[옛 코드 + 1.0.0] {}", old_user_code!(v1_0));

    // 실제 사용자 크레이트라면 여기서 deprecated 경고가 납니다. 예제는 경고 없이 빌드되도록
    // 막아 두었고, 이 속성을 지우면 README 2.6절에 옮긴 경고를 직접 볼 수 있습니다.
    #[expect(
        deprecated,
        reason = "1.1.0의 옛 코드가 경고만 받고 그대로 동작함을 보이려고 부릅니다"
    )]
    let old_on_1_1 = old_user_code!(v1_1);
    println!("[옛 코드 + 1.1.0] {old_on_1_1}");

    // old_user_code!(v2_0)은 E0425(cannot find function `track`)로 컴파일되지 않습니다.
    println!("[옮긴 코드 + 1.1.0] {}", migrated_user_code!(v1_1));
    println!("[옮긴 코드 + 2.0.0] {}", migrated_user_code!(v2_0));
    // 출력:
    // [옛 코드 + 1.0.0] KR-1203 → Some(InTransit), kr1203 → None, KR-9999 → None
    // [옛 코드 + 1.1.0] KR-1203 → Some(InTransit), kr1203 → None, KR-9999 → None
    // [옮긴 코드 + 1.1.0] KR-1203 → InTransit, kr1203 → 형식 오류, KR-9999 → 미등록
    // [옮긴 코드 + 2.0.0] KR-1203 → InTransit, kr1203 → 형식 오류, KR-9999 → 미등록
}
