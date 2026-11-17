// 기대하는 에러: E0369 — binary operation `==` cannot be applied to type `Point`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0369_assert_eq_without_partial_eq.rs --out-dir /tmp/rust-ch11
//
// assert_eq!(a, b)는 안에서 a == b로 두 값을 비교합니다.
// Point에는 Debug만 있고 PartialEq가 없어서 ==를 쓸 수 없습니다.
// 테스트 함수 안이든 main 안이든 같은 에러가 나므로, rustc만으로 확인할 수 있게 main에 썼습니다.

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn origin() -> Point {
    Point { x: 0, y: 0 }
}

fn main() {
    assert_eq!(origin(), Point { x: 0, y: 0 });
}
