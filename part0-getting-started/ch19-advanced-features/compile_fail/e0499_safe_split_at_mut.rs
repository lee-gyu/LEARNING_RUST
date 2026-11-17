// 기대하는 에러: E0499 — cannot borrow `*values` as mutable more than once at a time
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0499_safe_split_at_mut.rs
//
// 두 슬라이스 [..mid]와 [mid..]는 겹치지 않으므로 실제로는 안전합니다.
// 하지만 대여 검사기는 "같은 슬라이스를 두 번 가변으로 빌렸다"는 것까지만 알고,
// 범위가 겹치지 않는다는 사실은 알지 못합니다.
// 해결: 표준 라이브러리의 `values.split_at_mut(mid)`를 쓰거나,
//       examples/01_unsafe_basics.rs처럼 원시 포인터와 unsafe로 직접 구현해 안전한 함수로 감쌉니다.

fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();

    assert!(mid <= len);

    (&mut values[..mid], &mut values[mid..])
}

fn main() {
    let mut vector = vec![1, 2, 3, 4, 5, 6];
    let (left, right) = split_at_mut(&mut vector, 3);
    println!("{left:?} {right:?}");
}
