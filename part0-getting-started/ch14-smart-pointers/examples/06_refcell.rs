// RefCell<T>로 대여 규칙을 컴파일 시점이 아니라 실행 시점에 검사합니다(내부 가변성).
// - RefCell은 불변으로 묶여 있어도 안의 값을 바꿀 수 있게 해 줍니다.
// - borrow()는 Ref<T>, borrow_mut()는 RefMut<T>를 돌려주고, 이 값이 버려질 때 대여가 끝납니다.
// - 규칙을 어기면 컴파일 에러 대신 실행 중에 패닉합니다. try_borrow_mut()로는 패닉 없이 확인할 수 있습니다.
// - Rc<RefCell<T>>: 여러 소유자가 함께 가지면서 바꿀 수도 있는 값입니다.
//
// 실행: cargo run -p p0-ch14-smart-pointers --example 06_refcell
// 패닉까지 보기: cargo run -p p0-ch14-smart-pointers --example 06_refcell -- panic
//   (5번 구역에서 일부러 패닉하며 끝납니다. 종료 코드 101)

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

// 05번 예제의 리스트에서, 각 칸의 값을 Rc<RefCell<i32>>로 바꿨습니다.
// Rc로 여러 리스트가 같은 값을 소유하고, RefCell로 그 값을 바꿀 수 있습니다.
#[derive(Debug)]
enum List {
    Cons(Rc<RefCell<i32>>, Rc<List>),
    Nil,
}

use crate::List::{Cons, Nil};

// 각 칸의 값을 borrow()로 읽어 "3 -> 15 -> Nil" 모양의 문자열로 만듭니다.
fn to_text(list: &List) -> String {
    match list {
        Cons(value, rest) => format!("{} -> {}", value.borrow(), to_text(rest)),
        Nil => String::from("Nil"),
    }
}

fn main() {
    println!("1. RefCell 안의 값은 바인딩이 불변이어도 바꿀 수 있습니다");
    let notes = RefCell::new(vec![String::from("첫 메모")]); // mut가 없습니다.
    // borrow_mut()가 돌려준 RefMut는 이 문장이 끝날 때 버려지고, 그때 가변 대여도 끝납니다.
    notes.borrow_mut().push(String::from("둘째 메모"));
    println!("    notes = {:?}", notes.borrow()); // 출력: notes = ["첫 메모", "둘째 메모"]

    println!("2. 대여 규칙은 같습니다: 불변 대여는 여러 개, 가변 대여는 하나만");
    {
        let r1 = notes.borrow(); // Ref<Vec<String>>. Deref 덕분에 &Vec<String>처럼 씁니다.
        let r2 = notes.borrow(); // 불변 대여는 동시에 여러 개 있어도 됩니다.
        println!("    r1.len() = {}, r2[0] = {}", r1.len(), r2[0]); // 출력: r1.len() = 2, r2[0] = 첫 메모
    } // r1, r2가 버려지며 불변 대여가 끝납니다.
    {
        let mut w = notes.borrow_mut(); // 다른 대여가 없으므로 가변 대여에 성공합니다.
        w.push(String::from("셋째 메모"));
        println!("    w.len() = {}", w.len()); // 출력: w.len() = 3
    } // w가 버려지며 가변 대여가 끝납니다.

    println!("3. 규칙 위반은 실행 중에 드러납니다 (try_ 메서드로 패닉 없이 확인)");
    let reader = notes.borrow();
    // 읽는 중(reader가 살아 있음)에 가변 대여를 시도하면 Err가 돌아옵니다.
    if let Err(e) = notes.try_borrow_mut() {
        println!("    읽는 중에 가변 대여 → 실패: {e}"); // 출력: 읽는 중에 가변 대여 → 실패: RefCell already borrowed
    }
    drop(reader); // 대여를 일찍 끝냅니다(04번 예제의 std::mem::drop).
    if let Ok(mut w) = notes.try_borrow_mut() {
        w.push(String::from("넷째 메모"));
        println!("    reader를 버린 뒤 가변 대여 → 성공, 길이 {}", w.len()); // 출력: reader를 버린 뒤 가변 대여 → 성공, 길이 4
    }
    let writer = notes.borrow_mut();
    // 반대로 쓰는 중에 불변 대여를 시도해도 실패합니다.
    if let Err(e) = notes.try_borrow() {
        println!("    쓰는 중에 불변 대여 → 실패: {e}"); // 출력: 쓰는 중에 불변 대여 → 실패: RefCell already mutably borrowed
    }
    drop(writer);

    println!("4. Rc<RefCell<T>>: 여러 소유자가 같은 값을 바꿀 수 있습니다");
    let value = Rc::new(RefCell::new(5));
    let a = Rc::new(Cons(Rc::clone(&value), Rc::new(Nil)));
    let b = Cons(Rc::new(RefCell::new(3)), Rc::clone(&a));
    let c = Cons(Rc::new(RefCell::new(4)), Rc::clone(&a));
    // value, a 속의 값, b와 c가 공유하는 꼬리의 값은 모두 같은 RefCell입니다.
    *value.borrow_mut() += 10;
    println!("    a = {}", to_text(&a)); // 출력: a = 15 -> Nil
    println!("    b = {}", to_text(&b)); // 출력: b = 3 -> 15 -> Nil
    println!("    c = {}", to_text(&c)); // 출력: c = 4 -> 15 -> Nil
    // 디버그 형식으로 출력하면 Rc는 드러나지 않고 RefCell은 RefCell { value: .. }로 보입니다.
    println!("    b = {b:?}"); // 출력: b = Cons(RefCell { value: 3 }, Cons(RefCell { value: 15 }, Nil))

    // 명령줄 인자에 panic이 있을 때만 실행합니다(인자는 ch12, any와 클로저는 ch13).
    if env::args().any(|arg| arg == "panic") {
        println!("5. 가변 대여를 두 번 하면 컴파일은 되지만 실행 중에 패닉합니다");
        let mut first = notes.borrow_mut();
        let mut second = notes.borrow_mut(); // 여기서 패닉합니다. first의 가변 대여가 살아 있습니다.
        first.push(String::from("닿지 않는 코드"));
        second.push(String::from("닿지 않는 코드"));
    } else {
        println!("5. (건너뜀) `-- panic`을 붙여 실행하면 가변 대여를 두 번 해서 패닉합니다");
    }
}
