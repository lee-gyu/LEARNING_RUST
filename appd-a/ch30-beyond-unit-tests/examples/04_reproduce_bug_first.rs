// 지침: 버그를 고칠 때는 먼저 그 버그를 재현하는 테스트를 쓰고, 그 테스트가 실패하는 것을 본 뒤에 고칩니다.
//
// 실행: cargo run -p er-ch30-beyond-unit-tests --example 04_reproduce_bug_first
// 테스트: cargo test -p er-ch30-beyond-unit-tests --example 04_reproduce_bug_first
//
// 버그 신고: "글이 정확히 20개일 때 목록 아래에 빈 3페이지가 보입니다."
// 고치기 전 코드와 고친 코드를 함께 두고, 테스트를 어떤 순서로 돌리는지 따라가 봅니다.

use std::num::NonZeroUsize;

type PageFn = fn(usize, NonZeroUsize) -> usize;

const PER_PAGE: NonZeroUsize = NonZeroUsize::new(10).unwrap();

/// 고치기 전: 나머지가 있든 없든 한 페이지를 더합니다.
fn page_count_before(items: usize, per_page: NonZeroUsize) -> usize {
    items / per_page + 1
}

/// 고친 뒤: 올림 나눗셈.
/// `per_page`가 `NonZeroUsize`라 0으로 나누는 경우는 테스트할 필요가 없습니다(예제 01).
fn page_count_after(items: usize, per_page: NonZeroUsize) -> usize {
    items.div_ceil(per_page.get())
}

/// 신고 전부터 있던 테스트: 모두 나머지가 있는 경우라 버그를 지나쳤습니다.
const EXISTING: [(usize, usize); 4] = [(1, 1), (9, 1), (11, 2), (25, 3)];
/// 재현 테스트: 신고된 입력 그대로. 같은 원인(나누어떨어짐)을 가진 10개와 0개도 함께 정합니다.
const REPRO: [(usize, usize); 3] = [(20, 2), (10, 1), (0, 0)];
/// 재현하지 못한 테스트: 신고를 잘못 읽고 쓴 테스트입니다.
const NOT_A_REPRO: [(usize, usize); 1] = [(25, 3)];

fn check(f: PageFn, cases: &[(usize, usize)]) -> Result<(), String> {
    for &(items, expected) in cases {
        let got = f(items, PER_PAGE);
        if got != expected {
            return Err(format!("글 {items}개 → {got}페이지 (기대: {expected})"));
        }
    }
    Ok(())
}

fn step(label: &str, f: PageFn, cases: &[(usize, usize)]) {
    let verdict = match check(f, cases) {
        Ok(()) => "통과".to_string(),
        Err(why) => format!("실패: {why}"),
    };
    println!("{label} {verdict}");
}

fn main() {
    step("1. 고치기 전 + 기존 테스트:", page_count_before, &EXISTING);
    step("2. 고치기 전 + 재현 테스트:", page_count_before, &REPRO);
    step("3. 고친 뒤   + 재현 테스트:", page_count_after, &REPRO);
    step("4. 고친 뒤   + 기존 테스트:", page_count_after, &EXISTING);
    println!("(비교) 재현하지 못한 테스트");
    step("   고치기 전:", page_count_before, &NOT_A_REPRO);
    step("   고친 뒤:  ", page_count_after, &NOT_A_REPRO);
    // 출력:
    // 1. 고치기 전 + 기존 테스트: 통과
    // 2. 고치기 전 + 재현 테스트: 실패: 글 20개 → 3페이지 (기대: 2)
    // 3. 고친 뒤   + 재현 테스트: 통과
    // 4. 고친 뒤   + 기존 테스트: 통과
    // (비교) 재현하지 못한 테스트
    //    고치기 전: 통과
    //    고친 뒤:   통과
}

// 고친 뒤에 남기는 테스트: 기존 테스트와 재현 테스트(이제는 회귀 테스트)를 함께 둡니다.
// 회귀 테스트의 이름과 주석에 신고 내용을 적어 두면, 나중에 이 테스트가 실패했을 때
// 무엇이 다시 깨졌는지 압니다.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_cases() {
        assert_eq!(check(page_count_after, &EXISTING), Ok(()));
    }

    /// 버그 신고: 글이 정확히 20개일 때 빈 3페이지가 보임.
    #[test]
    fn regression_exact_multiple_has_no_empty_page() {
        assert_eq!(check(page_count_after, &REPRO), Ok(()));
    }
}
