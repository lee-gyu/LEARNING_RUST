//! 같은 코드를 2021 에디션과 2024 에디션으로 컴파일해서, never 타입 폴백이 어떤 타입을
//! 고르는지 비교합니다. rustc 1.100.0부터는 두 에디션의 결과가 같습니다.
//!
//! 실행(이 프로젝트 디렉터리에서):
//!   cargo run -q --bin never_fallback_2021
//!   cargo run -q --bin never_fallback_2024

use std::any::type_name;

/// 클로저 `f`의 반환 타입 `R`이 무엇으로 정해졌는지 이름으로 알려 줍니다.
/// 클로저를 부르지는 않으므로, 안에 든 `panic!()`은 실행되지 않습니다.
fn return_type_of<R, F: FnOnce() -> R>(_f: F) -> &'static str {
    type_name::<R>()
}

fn main() {
    println!("1. 반환 타입을 정할 단서가 없는 클로저");
    // `panic!()`의 타입은 `!`이고, R을 정해 줄 다른 단서가 없습니다.
    // 이때 컴파일러가 고르는 타입이 "폴백 타입"입니다.
    println!(
        "    || panic!()          → R = {}",
        return_type_of(|| panic!())
    );
    // 출력(두 에디션, rustc 1.100.0): R = !

    println!("2. 단서가 있으면 폴백이 쓰이지 않습니다");
    let flag = std::env::args().count() > 100; // 실행할 때만 알 수 있는 조건
    println!(
        "    if 갈래 하나가 u8     → R = {}",
        return_type_of(move || if flag { 5u8 } else { panic!() })
    );
    println!(
        "    R을 ()로 직접 지정    → R = {}",
        return_type_of::<(), _>(|| panic!())
    );
    // 출력(두 에디션): R = u8, R = ()
}
