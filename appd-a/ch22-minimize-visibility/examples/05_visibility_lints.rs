//! 지침 5: "never used" 경고를 공개 범위의 신호로 읽고, 린트로 공개 범위를 점검한다.
//! (`unreachable_pub`과 `private_interfaces`는 책에 없는 린트입니다.)
//!
//! 이 파일은 기본값이 allow인 `unreachable_pub`을 첫 줄에서 켭니다. 비권장 `stats_loose`에서
//! 나는 경고 세 가지를 `#[expect(…)]`로 표시했습니다. 속성을 지우면 각각 경고가 납니다(README 2.6절).
//!   1. `math::median`: `pub`이지만 비공개 모듈 안에 있고 아무도 부르지 않음
//!      → dead_code("never used")와 unreachable_pub. 공개하려던 함수인데 길이 닫혀 있다는 신호입니다.
//!   2. `math::mean`: 비공개 모듈 안의 `pub`. 크레이트 안에서만 쓰임 → unreachable_pub
//!   3. `summarize`: `pub` 함수가 비공개 타입 `Summary`를 돌려줌 → private_interfaces.
//!      바깥에서는 이 함수를 불러도 결과를 쓸 수 없습니다.
//!
//! 권장 `stats`: median은 `pub use`로 공개 경로를 열고, mean은 쓰는 곳까지만(`pub(super)`),
//! `Summary`는 함수와 같은 수준으로 공개합니다.
//!
//! 실행: cargo run -p er-ch22-minimize-visibility --example 05_visibility_lints
#![warn(unreachable_pub)]

const TEMPS: [i32; 7] = [3, -3, 7, 12, 5, 1, 5];

// ---- 비권장: 선언한 가시성과 실제로 닿는 범위가 어긋남 ----

pub mod stats_loose {
    mod math {
        #[expect(
            unreachable_pub,
            reason = "비권장 예: 비공개 모듈 안이라 pub이 크레이트 밖까지 닿지 않음"
        )]
        pub fn mean(xs: &[i32]) -> f64 {
            let sum: i32 = xs.iter().sum();
            f64::from(sum) / xs.len() as f64
        }

        #[expect(
            dead_code,
            unreachable_pub,
            reason = "비권장 예: 공개하려던 함수지만 경로가 닫혀 있어 아무도 부를 수 없음"
        )]
        pub fn median(xs: &[i32]) -> Option<i32> {
            let mut sorted = xs.to_vec();
            sorted.sort();
            sorted.get(sorted.len() / 2).copied()
        }
    }

    struct Summary {
        min: i32,
        max: i32,
        mean: f64,
    }

    #[expect(
        private_interfaces,
        reason = "비권장 예: pub 함수가 비공개 타입을 돌려줌"
    )]
    pub fn summarize(xs: &[i32]) -> Summary {
        Summary {
            min: xs.iter().copied().min().unwrap_or_default(),
            max: xs.iter().copied().max().unwrap_or_default(),
            mean: math::mean(xs),
        }
    }

    pub fn describe(xs: &[i32]) -> String {
        let s = summarize(xs);
        format!(
            "{}개: 최저 {}, 최고 {}, 평균 {:.1}",
            xs.len(),
            s.min,
            s.max,
            s.mean
        )
    }
}

// ---- 권장: 선언한 가시성과 실제로 닿는 범위를 맞춤 ----

pub mod stats {
    // 공개하려던 함수는 공개 경로를 엽니다. 이제 크레이트 밖에서 닿으므로 경고가 없습니다.
    pub use math::median;

    mod math {
        // 쓰는 곳은 stats뿐입니다. unreachable_pub의 help도 실제로 닿는 범위인 pub(super)를 제안합니다.
        pub(super) fn mean(xs: &[i32]) -> f64 {
            let sum: i32 = xs.iter().sum();
            f64::from(sum) / xs.len() as f64
        }

        pub fn median(xs: &[i32]) -> Option<i32> {
            let mut sorted = xs.to_vec();
            sorted.sort();
            sorted.get(sorted.len() / 2).copied()
        }
    }

    /// 공개 함수가 돌려주는 타입이므로 함께 공개합니다. 필드는 비공개로 두고 메서드로 엽니다.
    pub struct Summary {
        min: i32,
        max: i32,
        mean: f64,
    }

    impl Summary {
        pub fn min(&self) -> i32 {
            self.min
        }

        pub fn max(&self) -> i32 {
            self.max
        }

        pub fn mean(&self) -> f64 {
            self.mean
        }
    }

    pub fn summarize(xs: &[i32]) -> Summary {
        Summary {
            min: xs.iter().copied().min().unwrap_or_default(),
            max: xs.iter().copied().max().unwrap_or_default(),
            mean: math::mean(xs),
        }
    }

    pub fn describe(xs: &[i32]) -> String {
        let s = summarize(xs);
        format!(
            "{}개: 최저 {}, 최고 {}, 평균 {:.1}",
            xs.len(),
            s.min(),
            s.max(),
            s.mean()
        )
    }
}

fn main() {
    println!("1. 비권장: 경고 세 개를 #[expect]로 표시한 판");
    println!("    {}", stats_loose::describe(&TEMPS));
    // stats_loose::summarize(&TEMPS); // 에러 코드 없는 에러: type `stats_loose::Summary` is private
    // stats_loose::math::median(&TEMPS); // E0603: module `math` is private

    println!("2. 권장: 닿아야 할 곳까지만, 닿아야 할 곳에는 반드시");
    println!("    {}", stats::describe(&TEMPS));
    println!("    stats::median → {:?}", stats::median(&TEMPS)); // 출력: Some(5)
    let s = stats::summarize(&TEMPS);
    println!("    stats::summarize → 최고 {}", s.max()); // 출력: 최고 12
}
