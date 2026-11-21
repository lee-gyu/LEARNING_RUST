//! 콜라츠 수열 계산. 이 크레이트에 계산 시간이 몰려 있습니다.

/// 콜라츠 수열의 다음 값입니다.
fn next(n: u64) -> u64 {
    if n.is_multiple_of(2) {
        n / 2
    } else {
        3 * n + 1
    }
}

/// `n`에서 시작해 1에 닿을 때까지의 단계 수입니다.
fn steps(mut n: u64) -> u32 {
    let mut count = 0;
    while n != 1 {
        n = next(n);
        count += 1;
    }
    count
}

/// `1..=limit` 가운데 수열이 가장 긴 시작 값과 그 단계 수를 돌려줍니다.
///
/// 제네릭이 아니므로 이 크레이트를 컴파일할 때 기계어가 만들어집니다.
/// 그래서 이 크레이트에 준 `opt-level`이 그대로 적용됩니다.
pub fn longest(limit: u64) -> (u64, u32) {
    let mut best = (1, 0);
    for start in 1..=limit {
        let s = steps(start);
        if s > best.1 {
            best = (start, s);
        }
    }
    best
}

/// 같은 계산을, 다음 값을 구하는 함수를 받아서 하는 제네릭 판입니다.
///
/// 제네릭 함수는 구체 타입(여기서는 부르는 쪽의 클로저 타입)이 정해지는 곳에서 기계어가 만들어집니다.
/// 그래서 이 크레이트의 `opt-level`이 아니라 부르는 크레이트의 `opt-level`을 따릅니다.
pub fn longest_by<F: Fn(u64) -> u64>(limit: u64, next: F) -> (u64, u32) {
    let mut best = (1, 0);
    for start in 1..=limit {
        let mut n = start;
        let mut count = 0;
        while n != 1 {
            n = next(n);
            count += 1;
        }
        if count > best.1 {
            best = (start, count);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_under_ten() {
        // 9 → 28 → 14 → 7 → … → 1 은 19단계입니다.
        assert_eq!(longest(10), (9, 19));
    }

    #[test]
    fn generic_version_agrees() {
        let next = |n: u64| {
            if n.is_multiple_of(2) {
                n / 2
            } else {
                3 * n + 1
            }
        };
        assert_eq!(longest_by(1000, next), longest(1000));
    }
}
