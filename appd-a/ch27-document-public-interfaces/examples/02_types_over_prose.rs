//! 지침 2: 타입으로 말할 수 있는 조건은 문서 대신 타입에 맡깁니다.
//!
//! 실행: cargo run -p er-ch27-document-public-interfaces --example 02_types_over_prose
//! 문서: cargo doc -p er-ch27-document-public-interfaces --example 02_types_over_prose --no-deps
//!
//! `v1`은 "0이면 안 된다", "100 이하여야 한다"는 조건을 문서의 문장으로만 알립니다. 문장은
//! 컴파일러가 읽지 않으므로, 부르는 쪽이 놓치면 실행 중에 패닉하거나 틀린 값이 나옵니다.
//! `v2`는 같은 조건을 매개변수의 타입(`NonZero<usize>`, `Percent`)으로 옮겼습니다. 조건을 어긴
//! 값은 함수에 닿기 전에, 값을 만드는 자리에서 걸러집니다. 그리고 그 조건은 시그니처가
//! 말하므로(지침 1) 문서에서 `# Panics` 절이 사라집니다.
//!
//! 패닉을 붙잡아 출력하려고 패닉 훅을 바꿉니다. 그래서 이 예제는 표준 에러에 패닉 메시지를
//! 내지 않습니다.

use std::num::NonZero;
use std::panic;

/// 비권장: 조건을 문서의 문장으로만 알립니다.
pub mod v1 {
    /// `total`원을 `people`명이 낼 몫으로 나눕니다. 나머지는 앞사람부터 1원씩 더 냅니다.
    ///
    /// # Panics
    ///
    /// `people`이 0이면 패닉합니다.
    pub fn split(total: u64, people: usize) -> Vec<u64> {
        let n = people as u64;
        let (base, extra) = (total / n, total % n);
        (0..n)
            .map(|i| if i < extra { base + 1 } else { base })
            .collect()
    }

    /// `price`원에서 `percent`퍼센트를 깎은 값을 돌려줍니다. 1원 미만은 버립니다.
    ///
    /// `percent`는 100 이하여야 합니다. 100보다 크면 디버그 빌드에서는 패닉하고, 릴리스
    /// 빌드에서는 엉뚱하게 큰 값을 돌려줍니다.
    pub fn discount(price: u64, percent: u8) -> u64 {
        price * u64::from(100 - percent) / 100
    }
}

/// 권장: 조건을 타입으로 옮겼습니다. 문서에는 타입이 말하지 못하는 것만 남습니다.
pub mod v2 {
    use std::num::NonZero;

    /// `total`원을 `people`명이 낼 몫으로 나눕니다. 나머지는 앞사람부터 1원씩 더 냅니다.
    pub fn split(total: u64, people: NonZero<usize>) -> Vec<u64> {
        let n = people.get() as u64;
        let (base, extra) = (total / n, total % n);
        (0..n)
            .map(|i| if i < extra { base + 1 } else { base })
            .collect()
    }

    /// 0 이상 100 이하의 백분율입니다.
    ///
    /// 범위를 확인하는 [`Percent::new`]로만 만들 수 있으므로, 이 타입의 값은 늘 범위 안에 있습니다.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Percent(u8);

    impl Percent {
        /// `value`가 100 이하이면 `Some`, 아니면 `None`을 돌려줍니다.
        pub fn new(value: u8) -> Option<Percent> {
            (value <= 100).then_some(Percent(value))
        }
    }

    /// `price`원에서 `percent`만큼 깎은 값을 돌려줍니다. 1원 미만은 버립니다.
    pub fn discount(price: u64, percent: Percent) -> u64 {
        price * u64::from(100 - percent.0) / 100
    }
}

/// 클로저를 실행해 패닉하면 그 메시지를, 아니면 결과를 문자열로 돌려줍니다.
fn run<T: std::fmt::Debug>(f: impl FnOnce() -> T + panic::UnwindSafe) -> String {
    match panic::catch_unwind(f) {
        Ok(v) => format!("{v:?}"),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_default();
            format!("패닉: {msg}")
        }
    }
}

fn main() {
    // 패닉 메시지를 표준 에러에 내지 않도록 훅을 비워 둡니다(run이 메시지를 대신 출력).
    panic::set_hook(Box::new(|_| {}));

    // 바깥(설정 파일, 사용자 입력)에서 들어온 값이라고 칩니다.
    let inputs = [3usize, 0];
    let percents = [30u8, 120];

    println!("1. 비권장 v1: 조건이 문서의 문장에만 있음");
    // 출력: 1. 비권장 v1: 조건이 문서의 문장에만 있음
    for people in inputs {
        println!(
            "  split(10000, {people}) → {}",
            run(|| v1::split(10_000, people))
        );
    }
    // 출력:   split(10000, 3) → [3334, 3333, 3333]
    // 출력:   split(10000, 0) → 패닉: attempt to divide by zero
    for p in percents {
        println!("  discount(5000, {p}) → {}", run(|| v1::discount(5_000, p)));
    }
    // 출력:   discount(5000, 30) → 3500
    // 출력:   discount(5000, 120) → 패닉: attempt to subtract with overflow
    // (--release로 실행하면 마지막 줄은 패닉 대신 틀린 값이 나옵니다. README 3절 참고.)

    println!("2. 권장 v2: 조건을 어긴 값은 값을 만드는 자리에서 걸러짐");
    // 출력: 2. 권장 v2: 조건을 어긴 값은 값을 만드는 자리에서 걸러짐
    for people in inputs {
        match NonZero::new(people) {
            Some(n) => println!("  split(10000, {people}) → {:?}", v2::split(10_000, n)),
            None => println!("  NonZero::new({people}) → None: 함수를 부르기 전에 처리"),
        }
    }
    // 출력:   split(10000, 3) → [3334, 3333, 3333]
    // 출력:   NonZero::new(0) → None: 함수를 부르기 전에 처리
    for p in percents {
        match v2::Percent::new(p) {
            Some(pct) => println!("  discount(5000, {p}) → {}", v2::discount(5_000, pct)),
            None => println!("  Percent::new({p}) → None: 함수를 부르기 전에 처리"),
        }
    }
    // 출력:   discount(5000, 30) → 3500
    // 출력:   Percent::new(120) → None: 함수를 부르기 전에 처리
}
