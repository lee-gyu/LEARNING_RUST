// 기대하는 에러: E0277 — `Point` doesn't implement `Debug`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_assert_eq_without_debug.rs --out-dir /tmp/rust-ch11
//
// assert_eq!는 비교가 실패하면 두 값을 {:?}(디버그 형식)로 출력합니다.
// Point에는 PartialEq만 있고 Debug가 없어서, 실패했을 때 값을 보여 줄 방법이 없습니다.
// 비교가 실제로 실패하는지와 상관없이 컴파일 단계에서 거부됩니다.

#[derive(PartialEq)]
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
