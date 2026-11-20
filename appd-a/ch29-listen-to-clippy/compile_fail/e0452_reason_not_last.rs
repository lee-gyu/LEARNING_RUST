// 기대하는 에러: E0452 — malformed lint attribute input
//   (reason in lint attribute must come last)
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0452_reason_not_last.rs
//
// 린트 속성의 `reason`은 린트 이름들 **뒤에**, 한 번만, 문자열 리터럴로 적어야 합니다.
// 이유를 먼저 쓰고 싶은 마음에 앞에 두면 속성 자체가 잘못된 것으로 거부됩니다(README 4.2절).

#[expect(reason = "다음 단계에서 쓸 함수", dead_code)] // E0452
fn next_step() -> u32 {
    42
}

fn main() {
    println!("아직 next_step을 부르지 않습니다");
}
