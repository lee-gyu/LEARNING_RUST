// 기대하는 에러: E0596 cannot borrow data in an `Rc` as mutable
// 원인: Rc는 여러 소유자가 함께 쓰는 값이라 &T(불변 참조)만 내줍니다. Rc는 DerefMut를 구현하지 않습니다.
//       push_str은 &mut String이 필요하므로 거부됩니다.
// 해결: 바꿔야 하는 값은 Rc<RefCell<T>>로 감싸고 borrow_mut()로 바꿉니다(examples/06_refcell.rs).
//
// 확인: rustc --edition 2024 compile_fail/e0596_mutate_inside_rc.rs

use std::rc::Rc;

fn main() {
    let shared = Rc::new(String::from("hello"));
    let other = Rc::clone(&shared);

    shared.push_str(", world");

    println!("{other}");
}
