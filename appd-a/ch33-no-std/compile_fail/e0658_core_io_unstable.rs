// 기대하는 에러: E0658 use of unstable library feature `core_io`
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0658_core_io_unstable.rs
//
// `std::io::Write`의 문서에서 "Source" 링크를 따라가면 `src/core/io/write.rs`가 나오고,
// `type_name::<dyn std::io::Write>()`도 `dyn core::io::write::Write`를 출력합니다
// (examples/01_where_items_live.rs). 그러나 1.100.0에서 `core::io` 모듈은 nightly 전용이라 stable의
// `no_std` 크레이트에서는 쓸 수 없습니다. 정의된 위치만 보고 판단하지 말고 core 문서의 해당 페이지에
// 안정화 표시가 있는지 확인합니다. stable에서 입출력이 필요한 부분은 std 기능 뒤에 둡니다
// (projects/tinyhex/src/io.rs).

#![no_std]

/// 측정값을 바이트로 내보냅니다.
pub fn dump<W: core::io::Write>(out: &mut W, readings: &[u8]) {
    let _ = out.write_all(readings);
}
