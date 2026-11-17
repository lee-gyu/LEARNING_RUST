// 기대하는 에러: E0072 recursive type `List` has infinite size
// 원인: List 안에 List를 직접 넣어서, List의 크기를 계산하려면 List의 크기가 필요합니다.
// 해결: 다음 칸을 Box<List>로 감쌉니다(examples/02_cons_list.rs).
//
// 확인: rustc --edition 2024 compile_fail/e0072_recursive_type.rs

enum List {
    Cons(i32, List),
    Nil,
}

use crate::List::{Cons, Nil};

fn main() {
    let _list = Cons(1, Cons(2, Cons(3, Nil)));
}
