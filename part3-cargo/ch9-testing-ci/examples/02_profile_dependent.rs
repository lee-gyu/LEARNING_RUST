//! 같은 테스트가 고른 프로필에 따라 통과하기도 하고 실패하기도 하는 모습을 봅니다.
//!
//! 실행:
//!   cargo test -q -p p3-ch9-testing-ci --example 02_profile_dependent
//!   cargo test -q -p p3-ch9-testing-ci --example 02_profile_dependent --release   (테스트 1개 실패, 종료 상태 101)

/// 점수를 더합니다. 일부러 `+`를 써서 오버플로 동작을 프로필에 맡깁니다.
fn add_scores(a: u8, b: u8) -> u8 {
    a + b
}

fn main() {
    println!("200 + 50 = {}", add_scores(200, 50));
    // 출력: 200 + 50 = 250
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_sum() {
        assert_eq!(add_scores(2, 3), 5);
    }

    /// 오버플로가 패닉한다고 가정한 테스트입니다.
    /// test 프로필(dev를 물려받음)은 overflow-checks가 켜져 있어 통과하지만,
    /// `--release`(release 프로필)에서는 래핑되어 패닉하지 않으므로 실패합니다.
    #[test]
    #[should_panic(expected = "overflow")]
    fn overflow_panics() {
        add_scores(200, 100);
    }

    #[test]
    fn report_profile() {
        // `-- --no-capture`로 실행하면 보입니다.
        println!("debug_assertions = {}", cfg!(debug_assertions));
    }
}
