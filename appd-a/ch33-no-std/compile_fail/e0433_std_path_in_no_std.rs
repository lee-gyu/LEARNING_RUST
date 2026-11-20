// 기대하는 에러: E0433 cannot find module or crate `std` in this scope (두 번)
//   help: you might be missing a crate named `std`
//   help: consider importing this module / struct
//         use core::time;  /  use core::time::Duration;
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0433_std_path_in_no_std.rs
//
// `#![no_std]` 크레이트에서는 `std`라는 이름 자체가 없습니다. `std::time::Duration`은 core의 타입을
// 다시 내보낸 것이므로 `core::time::Duration`으로 쓰면 컴파일됩니다(examples/01_where_items_live.rs).
// 이 경우는 컴파일러가 core의 경로를 찾아 제안합니다. std에만 있는 항목(예: `std::time::Instant`)에는
// 그런 제안이 없습니다.
// `no_std` 실행 파일은 패닉 처리기가 필요하므로 라이브러리(`--crate-type lib`)로 컴파일합니다.

#![no_std]

/// 측정 주기(밀리초)를 `Duration`으로 바꿉니다.
pub fn period(millis: u64) -> std::time::Duration {
    std::time::Duration::from_millis(millis)
}
