//! 일부러 컴파일에 실패하는 사용자 코드입니다.
//!
//! 기대하는 에러(projects/non_exhaustive/broken_shop 디렉터리에서 `cargo build`):
//!   E0004 non-exhaustive patterns: `_` not covered                    (shop_message의 match)
//!   E0639 cannot create non-exhaustive struct using struct expression  (리터럴 두 곳)
//!   E0638 `..` required with struct marked as non-exhaustive           (구조 분해 패턴)
//!
//! parcel의 `#[non_exhaustive]`를 모르는 것처럼 쓴 코드입니다. 두 가지로 읽을 수 있습니다.
//!   1. `#[non_exhaustive]` 타입을 크레이트 밖에서 쓸 때 막히는 세 가지 모양
//!   2. parcel에 `#[non_exhaustive]`가 없던 시절에는 컴파일되던 코드. 그래서 이 속성을 나중에 붙이는
//!      것 자체가 호환되지 않는 변경입니다(parcel/src/lib.rs에서 두 속성을 지우면 이 패키지가 컴파일됩니다).
//! 고친 코드는 ../shop/src/main.rs에 있습니다.

use parcel::{Options, Status};

fn shop_message(status: Status) -> &'static str {
    // E0004: 배리언트를 모두 나열해도 크레이트 밖에서는 `_` 갈래가 필요합니다.
    // 다음 버전에 배리언트가 늘 수 있기 때문입니다.
    match status {
        Status::Accepted => "주문이 접수되었습니다",
        Status::InTransit => "상품이 이동 중입니다",
        Status::Delivered => "배송이 끝났습니다",
        Status::Returned => "반송되었습니다",
    }
}

fn main() {
    if let Some(status) = parcel::status_of("KR-1203") {
        println!("{}", shop_message(status));
    }

    // E0639: 크레이트 밖에서는 구조체 리터럴로 만들 수 없습니다.
    let options = Options {
        signature: true,
        insured: false,
    };
    // E0639: 구조체 갱신 문법도 리터럴이라 막힙니다(ch7 2.3절의 `Default` + `..`).
    let with_insurance = Options {
        insured: true,
        ..Default::default()
    };
    // E0638: 구조 분해 패턴에는 `..`가 있어야 합니다.
    let Options { signature, insured } = options;
    println!("서명 {signature}, 보험 {insured}, {with_insurance:?}");
}
