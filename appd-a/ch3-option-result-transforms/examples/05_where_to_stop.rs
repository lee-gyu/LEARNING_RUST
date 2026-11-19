// 이 예제가 보여 주는 것: 변환 메서드보다 if let이나 match가 더 읽기 좋은 세 경우
// 실행: cargo run -q -p er-ch3-option-result-transforms --example 05_where_to_stop
//
// 변환 메서드는 "값을 받아 값을 돌려주는" 짧은 단계를 이을 때 빛납니다. 다음 경우에는 오히려 읽기 어려워집니다.
//   1. 값이 있을 때 부수 효과만 내고 끝날 때 → if let
//   2. 성공과 실패 갈래가 각각 여러 줄이고 하는 일이 서로 다를 때 → match
//   3. 에러 종류에 따라 일부는 처리하고 일부는 전파할 때 → match
// 각 구역의 두세 판은 같은 결과를 냅니다. 고르는 기준은 읽기 쉬움뿐입니다.

use std::num::{IntErrorKind, ParseIntError};

// ---------------------------------------------------------------------------
// 1. 값이 있을 때 부수 효과만 낼 때
// ---------------------------------------------------------------------------

// 비권장: None 갈래가 빈 블록입니다. clippy가 if let으로 바꾸라고 알려 줍니다(single_match).
#[expect(clippy::single_match, reason = "비권장 형태를 비교하려고 남겨 둡니다")]
fn apply_match(coupon: Option<&str>, log: &mut Vec<String>) {
    match coupon {
        Some(code) => log.push(format!("{code} 적용")),
        None => {}
    }
}

// 비권장: map은 "값을 바꾼 새 Option"을 만드는 메서드인데, 그 결과(Option<()>)를 버리고 있습니다.
// 읽는 사람은 결과를 어디에 쓰는지 찾게 됩니다. clippy가 잡습니다(option_map_unit_fn).
fn apply_map(coupon: Option<&str>, log: &mut Vec<String>) {
    #[expect(
        clippy::option_map_unit_fn,
        reason = "비권장 형태를 비교하려고 남겨 둡니다"
    )]
    coupon.map(|code| log.push(format!("{code} 적용")));
}

// 권장: "있으면 이것을 한다"를 그대로 적습니다.
fn apply_if_let(coupon: Option<&str>, log: &mut Vec<String>) {
    if let Some(code) = coupon {
        log.push(format!("{code} 적용"));
    }
}

// ---------------------------------------------------------------------------
// 2. 갈래마다 하는 일이 크고 서로 다를 때
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum PayError {
    Declined,
    Timeout,
}

// 비권장: map_or_else(실패 클로저, 성공 클로저)는 실패 쪽을 먼저 받습니다.
// 각 클로저가 여러 줄이 되면 어느 블록이 어느 경우인지 인자 순서를 기억해서 읽어야 합니다.
fn receipt_map_or_else(result: Result<u32, PayError>) -> String {
    result.map_or_else(
        |e| {
            let hint = match e {
                PayError::Declined => "다른 카드를 쓰세요",
                PayError::Timeout => "잠시 뒤 다시 시도하세요",
            };
            format!("결제 실패({e:?}): {hint}")
        },
        |amount| {
            let points = amount / 100;
            let shipping = if amount >= 30_000 {
                "무료 배송"
            } else {
                "배송비 3000원"
            };
            format!("결제 {amount}원, 적립 {points}점, {shipping}")
        },
    )
}

// 권장: 갈래마다 패턴(Ok, Err)이 이름표처럼 붙어 있어 순서를 외울 필요가 없습니다.
fn receipt_match(result: Result<u32, PayError>) -> String {
    match result {
        Ok(amount) => {
            let points = amount / 100;
            let shipping = if amount >= 30_000 {
                "무료 배송"
            } else {
                "배송비 3000원"
            };
            format!("결제 {amount}원, 적립 {points}점, {shipping}")
        }
        Err(e) => {
            let hint = match e {
                PayError::Declined => "다른 카드를 쓰세요",
                PayError::Timeout => "잠시 뒤 다시 시도하세요",
            };
            format!("결제 실패({e:?}): {hint}")
        }
    }
}

// ---------------------------------------------------------------------------
// 3. 에러 종류에 따라 일부는 처리하고 일부는 전파할 때
// ---------------------------------------------------------------------------
// 수량 입력란: 비워 두면 1개, 너무 크면 최대값(255), 그 밖에 읽을 수 없으면 에러를 돌려줍니다.

// 비권장 쪽: or_else의 클로저 안에 match가 들어갑니다. 결국 match를 쓰면서 한 겹 더 깊어졌습니다.
fn quantity_or_else(text: &str) -> Result<u8, ParseIntError> {
    text.trim().parse::<u8>().or_else(|e| match e.kind() {
        IntErrorKind::Empty => Ok(1),
        IntErrorKind::PosOverflow => Ok(u8::MAX),
        _ => Err(e),
    })
}

// 권장 쪽: 네 가지 결과가 같은 깊이에 나란히 놓입니다. 매치 가드로 에러의 종류까지 함께 봅니다.
fn quantity_match(text: &str) -> Result<u8, ParseIntError> {
    match text.trim().parse::<u8>() {
        Ok(n) => Ok(n),
        Err(e) if *e.kind() == IntErrorKind::Empty => Ok(1),
        Err(e) if *e.kind() == IntErrorKind::PosOverflow => Ok(u8::MAX),
        Err(e) => Err(e),
    }
}

fn main() {
    println!("== 1. 부수 효과만 낼 때: if let ==");
    for coupon in [Some("WELCOME10"), None] {
        let (mut a, mut b, mut c) = (Vec::new(), Vec::new(), Vec::new());
        apply_match(coupon, &mut a);
        apply_map(coupon, &mut b);
        apply_if_let(coupon, &mut c);
        println!("{coupon:?} → match판 {a:?}, map판 {b:?}, if let판 {c:?}");
    }
    // 출력:
    // Some("WELCOME10") → match판 ["WELCOME10 적용"], map판 ["WELCOME10 적용"], if let판 ["WELCOME10 적용"]
    // None → match판 [], map판 [], if let판 []

    println!();
    println!("== 2. 갈래마다 하는 일이 다를 때: match ==");
    for payment in [
        Ok(42_000),
        Ok(9_900),
        Err(PayError::Declined),
        Err(PayError::Timeout),
    ] {
        // 두 함수 모두 Result를 소비하므로 한쪽에는 복제본을 넘깁니다.
        let by_method = receipt_map_or_else(payment.clone());
        let by_match = receipt_match(payment);
        println!("{by_match} (두 판 일치: {})", by_method == by_match);
    }
    // 출력:
    // 결제 42000원, 적립 420점, 무료 배송 (두 판 일치: true)
    // 결제 9900원, 적립 99점, 배송비 3000원 (두 판 일치: true)
    // 결제 실패(Declined): 다른 카드를 쓰세요 (두 판 일치: true)
    // 결제 실패(Timeout): 잠시 뒤 다시 시도하세요 (두 판 일치: true)

    println!();
    println!("== 3. 일부 에러만 처리할 때: match + 매치 가드 ==");
    for text in ["3", "", "300", "-2", "셋"] {
        let a = quantity_or_else(text);
        let b = quantity_match(text);
        println!("{text:?} → {b:?} (두 판 일치: {})", a == b);
    }
    // 출력:
    // "3" → Ok(3) (두 판 일치: true)
    // "" → Ok(1) (두 판 일치: true)
    // "300" → Ok(255) (두 판 일치: true)
    // "-2" → Err(ParseIntError { kind: InvalidDigit }) (두 판 일치: true)
    // "셋" → Err(ParseIntError { kind: InvalidDigit }) (두 판 일치: true)
}
