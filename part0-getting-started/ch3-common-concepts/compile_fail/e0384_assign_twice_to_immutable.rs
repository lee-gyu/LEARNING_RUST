// 기대하는 에러: E0384 — cannot assign twice to immutable variable `x`
// 확인: rustc --edition 2024 compile_fail/e0384_assign_twice_to_immutable.rs
//
// `let`으로 만든 변수는 기본이 불변입니다. 한 번 값을 묶은 뒤 다시 대입할 수 없습니다.
// 고치는 방법: 값이 바뀌어야 한다면 `let mut x = 5;`로 선언합니다.
//              새 값으로 "다시 시작"하려는 것이라면 `let x = 6;`으로 섀도잉합니다.

fn main() {
    let x = 5;
    println!("x = {x}");
    x = 6;
    println!("x = {x}");
}
