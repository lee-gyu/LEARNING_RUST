// 기대하는 에러: error[E0599]: `Point` doesn't implement `std::fmt::Display`
// 핵심 메시지:   method cannot be called on `Point` due to unsatisfied trait bounds
//                `Point: std::fmt::Display` which is required by `Point: ToString`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0599_to_string_without_display.rs --out-dir /tmp/rust-p1-ch3
//
// to_string()은 `impl<T: Display + ?Sized> ToString for T`라는 포괄 구현에서 나옵니다(Part 0 ch10).
// Point는 Debug만 derive했고 Display는 구현하지 않았으므로 to_string()도 생기지 않습니다.
// 고치는 방법: impl fmt::Display for Point를 작성합니다. 디버그 형식의 문자열로 충분하다면
//             format!("{p:?}")를 씁니다.

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p = Point { x: 3, y: -2 };
    let text = p.to_string();
    println!("{text} ({}, {})", p.x, p.y);
}
