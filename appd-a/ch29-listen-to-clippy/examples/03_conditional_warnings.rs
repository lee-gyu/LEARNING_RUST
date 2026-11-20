//! 지침 2의 예외: 빌드 설정에 따라 나기도 하고 안 나기도 하는 경고에는 `#[expect]`를 쓰지 않습니다.
//!
//! 실행: cargo run -p er-ch29-listen-to-clippy --example 03_conditional_warnings
//!       cargo run -p er-ch29-listen-to-clippy --example 03_conditional_warnings --release
//!
//! `#[cfg(debug_assertions)]`가 붙은 문장은 릴리스 빌드에서 지워집니다. 그러면 그 문장만 쓰던
//! `mut`이 릴리스 빌드에서만 `unused_mut` 경고를 받습니다. 이런 자리에 `#[expect(unused_mut)]`를
//! 붙이면 디버그 빌드에서 "기대가 충족되지 않았다"는 경고가 납니다(README 2.3절, 실제 출력은 3절).
//! 그래서 세 가지 대안을 나란히 둡니다. 셋 다 디버그와 릴리스 양쪽에서 경고가 없습니다.

/// 대안 1: `#[allow]`에 이유를 적습니다. 어느 빌드에서나 조용하지만, 언젠가 경고가 아예 나지
/// 않게 되어도(예: cfg 문장을 지움) 속성이 낡았다는 것을 알려 주지 않습니다.
fn checks_allow() -> Vec<&'static str> {
    #[allow(
        unused_mut,
        reason = "릴리스 빌드에서는 아래 push가 지워져 mut이 쓰이지 않음"
    )]
    let mut checks = vec!["형식 검사"];
    #[cfg(debug_assertions)]
    checks.push("불변식 검사");
    checks
}

/// 대안 2: 경고가 나는 빌드에서만 `#[expect]`가 붙도록 `cfg_attr`로 조건을 똑같이 겁니다.
/// 정확하지만, 아래 `#[cfg]`의 조건을 바꿀 때 이 조건도 함께 바꿔야 합니다.
fn checks_cfg_attr() -> Vec<&'static str> {
    #[cfg_attr(
        not(debug_assertions),
        expect(
            unused_mut,
            reason = "릴리스 빌드에서는 아래 push가 지워져 mut이 쓰이지 않음"
        )
    )]
    let mut checks = vec!["형식 검사"];
    #[cfg(debug_assertions)]
    checks.push("불변식 검사");
    checks
}

/// 대안 3(권장): 경고가 생기지 않게 고칩니다. `cfg!`는 `true`/`false` 리터럴이 될 뿐 문장을
/// 지우지 않으므로, 어느 빌드에서나 `push`가 남아 `mut`이 쓰입니다(ch26 2.2절).
fn checks_cfg_macro() -> Vec<&'static str> {
    let mut checks = vec!["형식 검사"];
    if cfg!(debug_assertions) {
        checks.push("불변식 검사");
    }
    checks
}

fn main() {
    let build = if cfg!(debug_assertions) {
        "디버그"
    } else {
        "릴리스"
    };
    println!(
        "빌드: {build} (debug_assertions = {})",
        cfg!(debug_assertions)
    );
    // 출력(디버그): 빌드: 디버그 (debug_assertions = true)
    // 출력(릴리스): 빌드: 릴리스 (debug_assertions = false)
    println!("  대안 1 allow     : {:?}", checks_allow());
    println!("  대안 2 cfg_attr  : {:?}", checks_cfg_attr());
    println!("  대안 3 cfg! 매크로: {:?}", checks_cfg_macro());
    // 출력(디버그): 세 줄 모두 ["형식 검사", "불변식 검사"]
    // 출력(릴리스): 세 줄 모두 ["형식 검사"]
}
