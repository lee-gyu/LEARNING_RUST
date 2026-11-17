// 기대하는 에러: E0277 — `Rectangle` doesn't implement `Debug`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_debug_not_derived.rs
//
// `{:?}`(디버그 형식)로 출력하려면 그 타입이 Debug 기능을 갖고 있어야 합니다.
// 직접 만든 구조체는 기본으로 갖고 있지 않으므로 `#[derive(Debug)]`를 붙여 컴파일러에게 만들어 달라고 해야 합니다.

struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    let rect = Rectangle {
        width: 30,
        height: 50,
    };

    println!("넓이: {}", rect.width * rect.height);
    println!("rect = {rect:?}");
}
