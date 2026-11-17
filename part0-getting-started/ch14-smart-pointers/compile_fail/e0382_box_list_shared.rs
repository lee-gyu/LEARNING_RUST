// 기대하는 에러: E0382 use of moved value: `a`
// 원인: Box는 안의 값을 소유합니다. Box::new(a)로 a가 b 쪽으로 이동했으므로 c를 만들 때 다시 쓸 수 없습니다.
// 해결: 꼬리를 여러 리스트가 함께 소유하도록 Box 대신 Rc를 쓰고 Rc::clone(&a)를 넘깁니다
//       (examples/05_rc_shared_list.rs).
//
// 확인: rustc --edition 2024 compile_fail/e0382_box_list_shared.rs

enum List {
    Cons(i32, Box<List>),
    Nil,
}

use crate::List::{Cons, Nil};

fn main() {
    let a = Cons(5, Box::new(Cons(10, Box::new(Nil))));
    let _b = Cons(3, Box::new(a));
    let _c = Cons(4, Box::new(a));
}
