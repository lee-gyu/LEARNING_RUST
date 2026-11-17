// 기대하는 에러: E0308 — mismatched types (expected `&str`, found `usize`)
// 확인: rustc --edition 2024 compile_fail/e0308_mut_cannot_change_type.rs
//
// `mut`는 "같은 타입의 다른 값"으로 바꿀 수 있게 해 줄 뿐, 변수의 타입까지 바꾸지는 못합니다.
// `spaces`는 첫 값 "   " 때문에 `&str`(문자열) 타입으로 정해졌으므로 `usize`(숫자)를 넣을 수 없습니다.
// 고치는 방법: `let spaces = spaces.len();`처럼 섀도잉으로 새 변수를 만듭니다.

fn main() {
    let mut spaces = "   ";
    spaces = spaces.len();
    println!("spaces = {spaces}");
}
