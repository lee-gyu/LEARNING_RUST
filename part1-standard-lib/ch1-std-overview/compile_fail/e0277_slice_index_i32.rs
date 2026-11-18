// 기대하는 에러: error[E0277]: the type `[{integer}]` cannot be indexed by `i32`
//               (slice indices are of type `usize` or ranges of `usize`)
//
// 원인: 배열과 슬라이스의 인덱스는 `usize`와 `usize`의 범위만 됩니다(`SliceIndex<[T]>`를 구현한 타입).
//       `i32`는 음수일 수 있으므로 Rust는 인덱스로 쓰기 전에 직접 변환하게 합니다.
// 해결: 처음부터 `usize`로 선언하거나, `usize::try_from(i)`로 변환해 음수인 경우를 처리합니다.
//       값이 음수가 아님을 확신할 때만 `i as usize`를 씁니다(음수면 아주 큰 수로 바뀝니다).
//
// 확인: rustc --edition 2024 compile_fail/e0277_slice_index_i32.rs --out-dir /tmp/rust-p1-ch1

fn main() {
    let scores = [90, 85, 77];
    let i: i32 = 1;
    println!("{}", scores[i]);
}
