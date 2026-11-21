// 기대하는 에러: E0308
// 핵심 메시지: expected `&Node`, found `*const _`
//
// C에서 "못 찾으면 NULL"을 돌려주는 함수의 결과를 확인 없이 역참조하면 널 포인터 역참조가 됩니다
// (compare/05_null_deref.c). 그 습관으로 Rust의 참조 자리에 널 포인터를 넣으려 한 코드입니다. Rust의 참조
// `&Node`는 언제나 살아 있는 값을 가리키므로 널이 될 수 없고, std::ptr::null()이 만드는 것은 참조가 아니라
// 원시 포인터(`*const _`)입니다. 원시 포인터를 역참조하려면 unsafe가 필요합니다.
//
// 고치는 법: 없을 수 있는 값은 Option<&Node>로 돌려주고, 쓰기 전에 None인 경우를 처리합니다(ch5 Q5.1).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch9-c-memory-errors compile_fail/e0308_null_into_reference.rs

struct Node {
    value: i32,
}

fn main() {
    let node: &Node = std::ptr::null(); // E0308
    println!("{}", node.value);
}
