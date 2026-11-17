// 기대하는 에러: E0502 — cannot borrow `v` as mutable because it is also borrowed as immutable
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0502_push_while_borrowed.rs
//
// first는 v의 0번 요소를 빌린 참조이고, 마지막 println!에서 다시 쓰입니다.
// 그 사이에 push가 용량을 넘기면 벡터는 요소를 새 힙 공간으로 옮기고 예전 공간을 해제할 수 있으므로,
// first가 해제된 메모리를 가리키게 됩니다. 컴파일러는 이 가능성 자체를 막습니다.

fn main() {
    let mut v = vec![1, 2, 3, 4, 5];

    let first = &v[0];

    v.push(6);

    println!("첫 번째 요소: {first}");
}
