//! 지침 2: 예외는 가장 좁은 범위에, 이유와 함께, `#[allow]`가 아니라 `#[expect]`로 둡니다.
//!
//! 실행: cargo run -p er-ch29-listen-to-clippy --example 02_expect_with_reason
//!
//! `cast_possible_truncation`은 pedantic 그룹이라 기본으로 꺼져 있습니다. 이 파일은 바로 아래
//! 속성으로 이 파일에서만 켭니다. `reason`을 적어 두면 경고가 날 때 그 이유가 `note:`로 함께
//! 나옵니다(README 2.3절). 프로젝트 전체에 켜는 방법은 `projects/lint_config`에서 봅니다.
#![warn(
    clippy::cast_possible_truncation,
    reason = "길이를 1바이트 헤더에 담는 코드가 있어 잘림을 놓치지 않으려고 켬"
)]

/// 1바이트 헤더에 항목 수를 적고 항목을 이어 붙인 레코드를 흉내 냅니다.
/// 읽는 쪽은 헤더에 적힌 수만큼만 항목을 읽습니다.
fn read_back(header: u8, items: &[String]) -> usize {
    items.iter().take(usize::from(header)).count()
}

// 비권장: 캐스트 하나가 의도한 것이라서 모듈 전체에 이유 없는 `#[allow]`를 붙였습니다.
// 그 바람에 같은 모듈에 있는 의도하지 않은 캐스트(`header`)의 경고까지 사라졌습니다.
mod v1 {
    #![allow(clippy::cast_possible_truncation)]

    /// 진행률(%)입니다. `done`을 `total` 이하로 묶었으므로 결과는 0..=100입니다.
    pub fn percent(done: u32, total: u32) -> u8 {
        let done = done.min(total);
        (u64::from(done) * 100 / u64::from(total)) as u8
    }

    /// 헤더에 적을 항목 수입니다. 항목이 256개를 넘으면 조용히 잘립니다.
    pub fn header(items: &[String]) -> u8 {
        items.len() as u8
    }
}

// 권장: 예외는 의도한 캐스트가 있는 문장 하나에만, 이유와 함께 둡니다.
// 의도하지 않은 캐스트는 경고를 그대로 받아 고쳤습니다.
mod v2 {
    pub fn percent(done: u32, total: u32) -> u8 {
        let done = done.min(total);
        // 린트 속성은 식이 아니라 문장이나 항목에 붙으므로 결과를 `let` 문으로 받았습니다.
        // (속성이 붙은 `let` 문에는 clippy::let_and_return이 경고하지 않았습니다.)
        #[expect(
            clippy::cast_possible_truncation,
            reason = "done을 total 이하로 묶었으므로 결과는 0..=100"
        )]
        let percent = (u64::from(done) * 100 / u64::from(total)) as u8;
        percent
    }

    /// 헤더에 담을 수 없으면 실제 항목 수를 `Err`로 돌려줍니다.
    pub fn header(items: &[String]) -> Result<u8, usize> {
        u8::try_from(items.len()).map_err(|_| items.len())
    }
}

fn main() {
    let items: Vec<String> = (1..=300).map(|i| format!("항목{i}")).collect();

    println!("1. 비권장 v1: 모듈 전체에 이유 없는 #[allow]");
    println!("  percent(30, 40) = {}", v1::percent(30, 40)); // 출력: 75
    let h = v1::header(&items);
    println!(
        "  항목 {}개 → 헤더 {h}, 읽는 쪽이 되찾은 항목 {}개 (경고 없음)",
        items.len(),
        read_back(h, &items)
    ); // 출력: 항목 300개 → 헤더 44, 읽는 쪽이 되찾은 항목 44개 (경고 없음)

    println!("2. 권장 v2: 문장 하나에만 #[expect(…, reason)], 나머지는 고침");
    println!("  percent(30, 40) = {}", v2::percent(30, 40)); // 출력: 75
    match v2::header(&items) {
        Ok(h) => println!("  헤더 {h}, 되찾은 항목 {}개", read_back(h, &items)),
        Err(n) => println!("  항목 {n}개는 1바이트 헤더에 담을 수 없음 → 기록하지 않고 알림"),
    }
    // 출력: 항목 300개는 1바이트 헤더에 담을 수 없음 → 기록하지 않고 알림
    let few = &items[..3];
    if let Ok(h) = v2::header(few) {
        println!(
            "  항목 {}개 → 헤더 {h}, 되찾은 항목 {}개",
            few.len(),
            read_back(h, few)
        ); // 출력: 항목 3개 → 헤더 3, 되찾은 항목 3개
    }
}
