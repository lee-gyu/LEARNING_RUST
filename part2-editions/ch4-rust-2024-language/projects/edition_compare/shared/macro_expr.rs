//! 같은 매크로를 2021 에디션과 2024 에디션으로 컴파일해서, 프래그먼트 지정자 `expr`가
//! `const { .. }` 블록과 `_`를 표현식으로 받아들이는지 비교합니다.
//!
//! 실행(이 프로젝트 디렉터리에서):
//!   cargo run -q --bin macro_expr_2021
//!   cargo run -q --bin macro_expr_2024

/// 입력이 어느 규칙에 맞았는지 문자열로 알려 주는 매크로입니다.
/// 규칙은 위에서부터 차례로 시도하고, 처음 맞은 규칙을 씁니다.
macro_rules! which_rule {
    ($e:expr) => {
        "1번 규칙($e:expr)"
    };
    (const $b:block) => {
        "2번 규칙(const 블록)"
    };
    (_) => {
        "3번 규칙(밑줄)"
    };
}

/// 1번 규칙만 `expr_2021`로 바꾼 매크로입니다. `expr_2021`은 어느 에디션에서든
/// 2021 에디션까지의 `expr`처럼 `const` 블록과 `_`를 받지 않습니다.
macro_rules! which_rule_2021 {
    ($e:expr_2021) => {
        "1번 규칙($e:expr_2021)"
    };
    (const $b:block) => {
        "2번 규칙(const 블록)"
    };
    (_) => {
        "3번 규칙(밑줄)"
    };
}

fn main() {
    println!("1. $e:expr");
    println!("    1 + 1           → {}", which_rule!(1 + 1));
    println!("    const {{ 1 + 1 }} → {}", which_rule!(const { 1 + 1 }));
    println!("    _               → {}", which_rule!(_));
    // 출력(2021): 1번, 2번, 3번 규칙
    // 출력(2024): 1번, 1번, 1번 규칙
    // 2024에서는 expr가 const 블록과 _까지 받으므로 세 입력 모두 1번 규칙에 맞습니다.

    println!("2. $e:expr_2021 (두 에디션이 같음)");
    println!("    1 + 1           → {}", which_rule_2021!(1 + 1));
    println!(
        "    const {{ 1 + 1 }} → {}",
        which_rule_2021!(const { 1 + 1 })
    );
    println!("    _               → {}", which_rule_2021!(_));
    // 출력(두 에디션): 1번, 2번, 3번 규칙
}
