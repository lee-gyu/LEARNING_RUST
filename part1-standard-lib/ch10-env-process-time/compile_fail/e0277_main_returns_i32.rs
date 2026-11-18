// 기대하는 에러: E0277 — `main` has invalid return type `i32`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_main_returns_i32.rs --out-dir /tmp/rust-p1-ch10
//
// C처럼 main에서 정수를 돌려주면 그것이 종료 코드가 되리라 기대했습니다.
// 하지만 Rust의 main은 Termination 트레이트를 구현한 타입만 돌려줄 수 있고,
// 표준 라이브러리는 (), ExitCode, Result<T: Termination, E: Debug>, ! 에만 구현해 두었습니다.
// i32를 허락하지 않는 이유는 플랫폼마다 종료 코드로 쓸 수 있는 범위가 달라서입니다(유닉스는 하위 8비트만 전달).
// 해결: fn main() -> ExitCode로 바꾸고 ExitCode::from(2)를 돌려줍니다(From<u8>, 0~255).
//       (examples/02_exit_code.rs)

fn main() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("사용법: 프로그램 <이름>");
        return 2;
    }
    println!("안녕하세요, {}님", args[1]);
    0
}
