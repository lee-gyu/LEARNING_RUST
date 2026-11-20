//! 지침 4: 기본으로 꺼진 린트 목록도 읽어 보고, 쓸모 있는 것은 골라서 켭니다.
//!
//! 실행: cargo run -p er-ch29-listen-to-clippy --example 05_default_off_lints
//!
//! 아래 두 린트는 pedantic 그룹이라 기본 설정의 Clippy는 이 파일의 비권장 판에 아무 말도 하지
//! 않습니다. 이 파일은 바로 아래 속성으로 두 린트를 골라 켭니다. 비권장 판에 붙인 `#[expect]`를
//! 지우면 둘 다 경고하고, 이 속성까지 지우면 경고가 사라집니다(README 2.5절).
#![warn(clippy::float_cmp, clippy::needless_for_each)]

// ── 1. clippy::float_cmp: 부동소수점을 ==로 비교함 ─────────────────────────────────────
mod money {
    // 비권장: 금액을 f64로 더해 장부의 합계와 ==로 비교합니다. 0.1과 0.2는 2진수로 정확히
    // 나타낼 수 없어 더한 값이 0.3과 미세하게 다릅니다.
    #[expect(
        clippy::float_cmp,
        reason = "비권장 판: 기본으로 꺼진 린트가 잡는 버그를 보이려고 남김"
    )]
    pub fn balanced_v1(parts: &[f64], total: f64) -> bool {
        parts.iter().sum::<f64>() == total
    }

    // 권장: 린트의 help는 오차 범위 안에서 비교하라고 제안합니다. 금액이라면 한 단계 더 나아가
    // 가장 작은 단위를 1로 세는 정수(여기서는 0.1을 1로 셈)로 표현을 바꾸면 ==가 정확해집니다.
    pub fn balanced_v2(parts: &[u64], total: u64) -> bool {
        parts.iter().sum::<u64>() == total
    }
}

// ── 2. clippy::needless_for_each: for 반복문으로 충분한 for_each ───────────────────────
mod scan {
    // 비권장: 처음 나온 음수에서 멈추려고 클로저 안에 return을 썼습니다. 그러나 return은
    // 클로저 한 번만 끝내므로, 음수를 건너뛰고 끝까지 더합니다(ch9 2.5절).
    #[expect(
        clippy::needless_for_each,
        reason = "비권장 판: 기본으로 꺼진 린트가 잡는 버그를 보이려고 남김"
    )]
    pub fn sum_until_negative_v1(xs: &[i32]) -> i32 {
        let mut total = 0;
        xs.iter().for_each(|&x| {
            if x < 0 {
                return;
            }
            total += x;
        });
        total
    }

    // 권장: 린트의 help는 for 반복문으로 바꾸면서 "return을 continue로 바꾸라"고 덧붙입니다.
    // 그 문장을 읽으면 v1이 실제로 무엇을 했는지 드러납니다. 뜻대로 멈추려면 break를 씁니다.
    pub fn sum_until_negative_v2(xs: &[i32]) -> i32 {
        let mut total = 0;
        for &x in xs {
            if x < 0 {
                break;
            }
            total += x;
        }
        total
    }
}

fn main() {
    println!("1. clippy::float_cmp (pedantic)");
    println!(
        "  비권장 v1: 0.1 + 0.2 == 0.3 → {}",
        money::balanced_v1(&[0.1, 0.2], 0.3)
    ); // 출력: false
    println!(
        "  권장 v2  : 1 + 2 == 3 (10분의 1 단위 정수) → {}",
        money::balanced_v2(&[1, 2], 3)
    ); // 출력: true

    println!("2. clippy::needless_for_each (pedantic)");
    let xs = [3, 4, -1, 10];
    println!(
        "  {xs:?}에서 처음 음수 전까지의 합: 비권장 v1 {}, 권장 v2 {}",
        scan::sum_until_negative_v1(&xs),
        scan::sum_until_negative_v2(&xs)
    ); // 출력: 비권장 v1 17, 권장 v2 7
}
