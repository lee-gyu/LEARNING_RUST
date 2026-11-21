// 기대하는 에러: E0382
// 핵심 메시지: use of moved value: `buffer`
//
// C의 이중 해제(compare/02_double_free.c)를 그대로 옮긴 코드입니다. C에서 free(p)를 두 번 부르면 컴파일은
// 되고, 실행 중에 할당기의 장부가 망가질 수 있습니다(미정의 동작). Rust의 drop(buffer)는 buffer의 소유권을 받아 가는 함수라서,
// 한 번 넘긴 buffer는 다시 쓸 수 없습니다. 두 번째 drop은 "이미 옮겨 간 값을 쓴다"는 에러가 됩니다.
//
// 고치는 법: 두 번째 drop을 지웁니다. 컴파일러의 help는 clone을 제안하지만, 사본을 만들어 버리는 것은
// 이중 해제를 고치는 길이 아닙니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch9-c-memory-errors compile_fail/e0382_drop_twice.rs

fn main() {
    let buffer = vec![0u8; 64];
    drop(buffer); // 해제
    drop(buffer); // E0382: 이중 해제를 시도
}
