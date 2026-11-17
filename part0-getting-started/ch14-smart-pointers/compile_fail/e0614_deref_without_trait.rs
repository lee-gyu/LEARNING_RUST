// 기대하는 에러: E0614 type `MyBox<{integer}>` cannot be dereferenced
// 원인: MyBox는 Deref를 구현하지 않았으므로 컴파일러가 `*`를 어떻게 처리할지 모릅니다.
// 해결: impl Deref for MyBox<T>를 추가합니다(examples/03_deref_mybox.rs).
//
// 확인: rustc --edition 2024 compile_fail/e0614_deref_without_trait.rs

struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

fn main() {
    let x = 5;
    let y = MyBox::new(x);

    assert_eq!(5, x);
    assert_eq!(5, *y);
}
