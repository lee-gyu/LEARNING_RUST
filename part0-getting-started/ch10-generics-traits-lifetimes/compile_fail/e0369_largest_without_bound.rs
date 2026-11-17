// 기대하는 에러: E0369 — binary operation `>` cannot be applied to type `&T`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0369_largest_without_bound.rs
//
// 제네릭 함수의 본문은 T 자리에 올 수 있는 "모든" 타입에 대해 올바라야 합니다.
// 그런데 `>`로 비교할 수 없는 타입도 있으므로(예: 직접 만든 구조체), 컴파일러는 이 함수를 거부합니다.
// 해결: `fn largest<T: PartialOrd>(...)`처럼 "T는 비교할 수 있는 타입이어야 한다"는 트레이트 바운드를 답니다.

fn largest<T>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    println!("가장 큰 수: {}", largest(&numbers));
}
