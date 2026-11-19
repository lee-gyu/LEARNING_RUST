//! 기능 이름의 오타와 알리지 않은 cfg 이름을 `unexpected_cfgs` 린트가 잡게 합니다.
//! 지침: 기능 이름은 Cargo.toml에 적은 글자 그대로 쓰고, 사용자 정의 cfg 이름은 `[lints.rust]`에
//! 알려 둡니다. 경고를 끄지 않습니다.
//!
//! 실행: cargo run -p er-ch26-feature-creep --example 03_unexpected_cfgs
//!       cargo run -p er-ch26-feature-creep --example 03_unexpected_cfgs --features fast-hash
//!       RUSTFLAGS='--cfg expensive_checks' cargo run -p er-ch26-feature-creep --example 03_unexpected_cfgs

// 비권장: Cargo.toml의 기능 이름은 `fast-hash`(하이픈)인데 코드에는 `fast_hash`(밑줄)라고
// 적었습니다. 그런 기능은 없으므로 이 조건은 어떤 빌드에서도 거짓이고, 컴파일 에러도 나지
// 않습니다. Cargo는 선언된 기능 목록을 rustc에 넘기고(--check-cfg), rustc는 목록에 없는 값을
// unexpected_cfgs 경고로 알려 줍니다. 오타를 보여 주려고 모듈에 #[expect]를 붙여 경고를
// 막았습니다. 이 속성을 지우면 아래 두 곳에서 경고가 납니다.
//
// #[expect]를 오타 난 #[cfg]와 같은 항목에 붙이면 막지 못합니다. 조건이 거짓이라 그 항목이
// #[expect]째 지워지기 때문입니다. 그래서 감싸는 모듈에 붙였습니다.
#[expect(unexpected_cfgs, reason = "기능 이름의 오타를 일부러 보여 줍니다")]
mod v1 {
    /// 빠른 지문 함수가 이 빌드에 있는지.
    pub fn has_fast() -> bool {
        cfg!(feature = "fast_hash")
    }

    /// 기능이 켜졌을 때만 있을 상수. 오타 때문에 어떤 빌드에서도 생기지 않습니다.
    #[cfg(feature = "fast_hash")]
    pub const FAST_NAME: &str = "poly31";
}

// 권장: Cargo.toml과 같은 이름을 씁니다.
mod v2 {
    /// 빠른 지문 함수가 이 빌드에 있는지.
    pub fn has_fast() -> bool {
        cfg!(feature = "fast-hash")
    }

    /// `fast-hash` 기능이 켜졌을 때만 있는 상수.
    #[cfg(feature = "fast-hash")]
    pub const FAST_NAME: &str = "poly31";
}

/// 정렬된 목록에 값을 넣습니다. 돌려주는 값은 추가 검사를 했는지입니다.
///
/// `expensive_checks`는 Cargo 기능이 아니라 사용자 정의 cfg 이름입니다. 빌드하는 사람이
/// `RUSTFLAGS='--cfg expensive_checks'`로 켜고, 그러면 빌드되는 모든 크레이트에 함께 켜집니다.
/// 이 이름은 Cargo.toml의 `[lints.rust]`에 알려 두었으므로 경고가 나지 않습니다.
fn insert_sorted(list: &mut Vec<u32>, x: u32) -> bool {
    let at = list.partition_point(|&y| y < x);
    list.insert(at, x);
    if cfg!(expensive_checks) {
        // 넣을 때마다 목록 전체를 훑는 느린 검사라고 칩니다.
        assert!(list.is_sorted(), "정렬이 깨졌습니다: {list:?}");
        return true;
    }
    false
}

fn main() {
    println!("Cargo가 켠 기능 fast-hash: {}", cfg!(feature = "fast-hash")); // 출력: false

    println!("1. 비권장: 코드에 fast_hash(밑줄)라고 적음");
    println!("  v1::has_fast() = {}", v1::has_fast()); // 출력: false (기능을 켜도 false)

    println!("2. 권장: Cargo.toml과 같은 fast-hash(하이픈)");
    println!("  v2::has_fast() = {}", v2::has_fast()); // 출력: false (기능을 켜면 true)
    #[cfg(feature = "fast-hash")]
    println!("  v2::FAST_NAME = {}", v2::FAST_NAME);

    let on = cfg!(expensive_checks);
    println!("3. 사용자 정의 cfg expensive_checks: {on}"); // 출력: false
    let mut list = Vec::new();
    let mut checked = 0;
    for x in [21, 3, 42, 8, 15] {
        checked += usize::from(insert_sorted(&mut list, x));
    }
    println!("  정렬된 목록 {list:?}, 추가 검사 {checked}번"); // 출력: [3, 8, 15, 21, 42], 0번
}
