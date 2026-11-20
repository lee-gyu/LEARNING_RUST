// 기대하는 에러: E0433
//   error[E0433]: cannot find module or crate `itoa` in this scope
//
// `Cargo.toml`의 `[dependencies]`에 적지 않은 크레이트를 코드에서 쓴 실수입니다.
// Cargo는 `[dependencies]`에 적힌 크레이트에 대해서만 `rustc`에 `--extern`을 넘깁니다.
// 그래서 다음 세 경우에 똑같은 에러가 납니다.
//
//   1. 크레이트를 아예 적지 않았다.
//   2. `[dev-dependencies]`에만 적고 `src/`의 코드에서 썼다
//      (dev-dependencies는 테스트, 예제, 벤치마크를 컴파일할 때만 연결됩니다).
//   3. 내 의존성이 쓰는 크레이트(전이 의존성)를 직접 쓰려고 했다.
//
// `rustc`로 이 파일만 컴파일하면 `--extern`이 하나도 없으므로 같은 상황이 됩니다.
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0433_crate_not_in_dependencies.rs

fn main() {
    let mut buffer = itoa::Buffer::new();
    println!("{}", buffer.format(2024_u32));
}
