//! parcel 1.0.0 시절에 `#[non_exhaustive]`를 지키며 쓴 사용자 코드입니다.
//! parcel이 1.1.0으로 올라 배리언트와 필드가 늘었지만 한 글자도 고치지 않고 컴파일됩니다.
//!
//! 실행(projects/non_exhaustive 디렉터리에서): cargo run -p shop

use parcel::{Options, Status};

/// 사용자 쪽 안내 문구. 크레이트 밖에서는 `#[non_exhaustive]` 열거형의 `match`에 `_` 갈래가 꼭 필요합니다.
fn shop_message(status: Status) -> String {
    match status {
        Status::Accepted => String::from("주문이 접수되었습니다"),
        Status::InTransit => String::from("상품이 이동 중입니다"),
        Status::Delivered => String::from("배송이 끝났습니다"),
        // 1.0.0 시절에 쓴 코드라 Returned를 모릅니다. 1.1.0에서 생긴 배리언트는 이 갈래로 옵니다.
        _ => format!("배송 상태가 바뀌었습니다({})", parcel::label(status)),
    }
}

fn main() {
    for no in ["KR-0001", "KR-1203", "KR-0042", "KR-0777"] {
        if let Some(status) = parcel::status_of(no) {
            println!("{no}: {}", shop_message(status));
        }
    }

    // 구조체 리터럴 대신 라이브러리가 준 생성 함수와 세터로 만듭니다.
    let options = Options::new().signature(true);
    // 패턴에서는 `..`로 "모르는 필드가 더 있을 수 있음"을 적습니다.
    let Options { signature, .. } = options.clone();
    println!(
        "서명 {signature}, 배송비 {}원 (빠른 배송 {}원)",
        parcel::fee(&options),
        parcel::fee(&parcel::express())
    );
}
