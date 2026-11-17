// 기대하는 에러: E0308 — mismatched types (expected `i32`, found `()`)
// 확인: rustc --edition 2024 compile_fail/e0308_tail_expression_semicolon.rs
//
// 함수 본문의 마지막 줄 `x + 1;`에 세미콜론이 붙어서 표현식이 구문이 되었습니다.
// 구문은 값을 내지 않으므로 함수 본문에는 마지막 표현식이 없고, 본문의 값은 유닛 `()`가 됩니다.
// 반환 타입은 `i32`라고 약속했는데 `()`를 돌려주게 되어 타입이 맞지 않습니다.
// 고치는 방법: 마지막 줄의 세미콜론을 지워 `x + 1`이 본문의 값이 되게 합니다.

fn plus_one(x: i32) -> i32 {
    x + 1;
}

fn main() {
    let y = plus_one(5);
    println!("y = {y}");
}
