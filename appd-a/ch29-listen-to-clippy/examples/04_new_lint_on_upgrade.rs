//! 지침 3: 경고 0개를 유지합니다. 그래야 코드를 고치거나 Clippy를 올렸을 때 생긴 새 경고가
//! 바로 눈에 띕니다.
//!
//! 실행: cargo run -p er-ch29-listen-to-clippy --example 04_new_lint_on_upgrade
//!
//! `v1::bits_needed`의 버그는 Clippy 1.98.1이 경고하지 않고, 1.100.0에 새로 생긴
//! `clippy::mismatched_bit_width_type`(suspicious 그룹, 기본 warn)이 잡습니다. 같은 버그를
//! 리터럴 `32`로 쓴 `v1_literal`은 1.100.0에서 pedantic·nursery까지 켜도 아무 린트도 잡지
//! 못했습니다. 두 판의 결과가 같다는 것이 출력으로 보입니다(README 2.4절).

// 비권장: `x`는 u64인데 u32의 비트 수에서 뺍니다. x < 2^32이면 뺄셈이 넘쳐서 디버그 빌드에서는
// 패닉하고 릴리스 빌드에서는 엉뚱한 큰 값이 됩니다. 그래서 main은 2^32 이상의 값만 넣습니다.
mod v1 {
    #[expect(
        clippy::mismatched_bit_width_type,
        reason = "비권장 판: Clippy 1.100.0에 새로 생긴 린트가 잡는 버그를 보이려고 남김"
    )]
    pub fn bits_needed(x: u64) -> u32 {
        u32::BITS - x.leading_zeros()
    }
}

// 비권장: 같은 버그를 리터럴로 썼습니다. 린트는 정해진 모양(`T::BITS - x.leading_zeros()`)을
// 찾기 때문에 이 모양은 지나칩니다. 이런 버그는 테스트가 잡아야 합니다.
mod v1_literal {
    pub fn bits_needed(x: u64) -> u32 {
        32 - x.leading_zeros()
    }
}

// 권장: 1.97.0에서 안정화된 `bit_width`를 씁니다. 0이면 0을 돌려주고 넘침이 없습니다.
mod v2 {
    pub fn bits_needed(x: u64) -> u32 {
        x.bit_width()
    }
}

fn main() {
    println!(
        "{:>22}  {:>10}  {:>10}  {:>10}",
        "x", "v1 BITS판", "v1 32판", "v2"
    );
    for x in [5_000_000_000, 1 << 40, u64::MAX] {
        println!(
            "{x:>22}  {:>10}  {:>10}  {:>10}",
            v1::bits_needed(x),
            v1_literal::bits_needed(x),
            v2::bits_needed(x)
        );
    }
    // 출력: 5000000000 → 1, 1, 33 / 1099511627776 → 9, 9, 41 / 18446744073709551615 → 32, 32, 64
    println!(
        "v2는 작은 값도 됩니다: 0 → {}, 5 → {}",
        v2::bits_needed(0),
        v2::bits_needed(5)
    ); // 출력: 0 → 0, 5 → 3
}
