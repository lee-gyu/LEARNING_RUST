// 기대하는 에러: E0277 — cannot add `Option<i8>` to `i8`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_add_option_to_int.rs
//
// Option<i8>은 i8과 다른 타입입니다. y는 값이 없을(None) 수도 있으므로,
// 그 경우를 정하지 않은 채 i8처럼 더할 수 없습니다. null이 있는 언어에서라면
// 실행 중에 터졌을 실수를 컴파일할 때 막아 주는 것입니다.

fn main() {
    let x: i8 = 5;
    let y: Option<i8> = Some(5);

    let sum = x + y;
    println!("{sum}");
}
