// Rc<T>(참조 카운팅)로 값 하나에 소유자를 여럿 둡니다.
// - Rc::clone은 데이터를 복사하지 않고 "소유자 수"(strong count)만 1 늘립니다.
// - Rc가 하나 버려질 때마다 카운트가 1 줄고, 0이 되는 순간 값이 drop됩니다.
// - Rc는 단일 스레드 전용입니다. 스레드 사이의 공유는 ch15의 Arc<T>가 맡습니다.
//
// 실행: cargo run -p p0-ch14-smart-pointers --example 05_rc_shared_list

use std::rc::Rc; // Rc는 프렐류드에 없으므로 가져와야 합니다.

// 02번 예제의 cons 리스트에서 Box를 Rc로 바꿨습니다.
// 이제 리스트의 꼬리 하나를 여러 리스트가 함께 소유할 수 있습니다.
enum List {
    Cons(i32, Rc<List>),
    Nil,
}

use crate::List::{Cons, Nil};

fn to_text(list: &List) -> String {
    match list {
        Cons(value, rest) => format!("{value} -> {}", to_text(rest)),
        Nil => String::from("Nil"),
    }
}

// 마지막 소유자가 사라질 때 drop되는 것을 보기 위한 관찰용 타입입니다(04번 예제).
struct Noisy {
    name: String,
}

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.name);
    }
}

fn main() {
    println!("1. 리스트 a를 b와 c가 함께 소유합니다");
    //   b: 3 --+
    //          +--> a: 5 -> 10 -> Nil
    //   c: 4 --+
    let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
    println!("    a를 만든 뒤 카운트 = {}", Rc::strong_count(&a)); // 출력: a를 만든 뒤 카운트 = 1
    let b = Cons(3, Rc::clone(&a));
    println!("    b를 만든 뒤 카운트 = {}", Rc::strong_count(&a)); // 출력: b를 만든 뒤 카운트 = 2
    {
        let c = Cons(4, Rc::clone(&a));
        println!("    c를 만든 뒤 카운트 = {}", Rc::strong_count(&a)); // 출력: c를 만든 뒤 카운트 = 3
        println!("    b = {}", to_text(&b)); // 출력: b = 3 -> 5 -> 10 -> Nil
        println!("    c = {}", to_text(&c)); // 출력: c = 4 -> 5 -> 10 -> Nil
    } // c가 버려지면서 c 안의 Rc도 버려집니다. Rc의 Drop 구현이 카운트를 1 줄입니다.
    println!("    c가 사라진 뒤 카운트 = {}", Rc::strong_count(&a)); // 출력: c가 사라진 뒤 카운트 = 2

    println!("2. Rc::clone은 깊은 복사가 아니라 카운트 증가입니다");
    let text = Rc::new(String::from("공유되는 문자열"));
    let text2 = Rc::clone(&text);
    // ptr_eq는 두 Rc가 같은 값을 가리키는지 비교합니다. 데이터가 복사되지 않았으므로 true입니다.
    println!("    같은 값을 가리키는가? {}", Rc::ptr_eq(&text, &text2)); // 출력: 같은 값을 가리키는가? true
    // 안의 String을 복제하면 힙 데이터까지 새로 만들어집니다. 이것이 깊은 복사입니다.
    let deep_copy: String = (*text).clone();
    println!(
        "    String 복제본은 같은 글자를 가리키는가? {}",
        deep_copy.as_ptr() == text.as_ptr()
    ); // 출력: String 복제본은 같은 글자를 가리키는가? false
    // text.clone()도 Rc::clone과 같은 일을 하지만, 깊은 복사와 구분하려고 Rc::clone(&text)로 씁니다.
    println!("    text의 카운트 = {}", Rc::strong_count(&text)); // 출력: text의 카운트 = 2

    println!("3. 마지막 소유자가 사라질 때 비로소 값이 drop됩니다");
    let first = Rc::new(Noisy {
        name: String::from("공유 자원"),
    });
    let second = Rc::clone(&first);
    println!("    카운트 = {}", Rc::strong_count(&first)); // 출력: 카운트 = 2
    drop(first); // 소유자 하나가 사라졌지만 아직 second가 남아 있으므로 drop되지 않습니다.
    println!("    first를 버린 뒤 카운트 = {}", Rc::strong_count(&second)); // 출력: first를 버린 뒤 카운트 = 1
    drop(second); // 마지막 소유자입니다. 카운트가 0이 되어 Noisy가 drop됩니다.
    println!("    second까지 버렸습니다");
    // 출력:
    //     [drop] 공유 자원
    //     second까지 버렸습니다

    println!("4. Rc는 안의 값을 읽기만 할 수 있습니다");
    // 여러 소유자가 저마다 값을 바꿀 수 있다면 대여 규칙이 깨집니다. 그래서 Rc는 &T만 내줍니다.
    // text.push_str("!"); // E0596: cannot borrow data in an `Rc` as mutable
    // 바꿔야 한다면 Rc<RefCell<T>>를 씁니다(06번 예제).
    println!("    text = {text}, 길이 = {}", text.len()); // 출력: text = 공유되는 문자열, 길이 = 22
}
