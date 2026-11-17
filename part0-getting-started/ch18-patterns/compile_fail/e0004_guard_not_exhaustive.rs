// 기대하는 에러: E0004 non-exhaustive patterns: `i32::MIN..=i32::MAX` not covered
// 원인: 사람이 보기에는 t < 0과 t >= 0이 모든 정수를 덮지만, 컴파일러는 가드 조건을 따지지 않습니다.
//       가드가 붙은 갈래는 "맞지 않을 수도 있는 갈래"로 보므로, 이 match는 아무 값도 확실히 덮지 못합니다.
// 해결: 마지막 갈래의 가드를 지우고 가드 없는 패턴으로 나머지를 받습니다.
//       t if t < 0 => "영하",
//       _ => "영상",
//
// 확인: rustc --edition 2024 compile_fail/e0004_guard_not_exhaustive.rs

fn describe(temp: i32) -> &'static str {
    match temp {
        t if t < 0 => "영하",
        t if t >= 0 => "영상",
    }
}

fn main() {
    println!("{}", describe(-5));
}
