// 부록 B ch12 · Q12.2 — 콜백이 상태를 빌리지 않고 소유하게 하기
//
// compare/03의 C++ 코드는 람다가 [this]로 Counter를 가리킨 채 Button에 등록되고, Counter가 먼저 소멸해
// 해제 후 사용이 됩니다. 같은 구조를 Rust로 쓰면 compile_fail/e0597_callback_borrows_local.rs처럼
// 컴파일되지 않습니다. 구조체 필드와 함수 시그니처에 쓴 Box<dyn FnMut()>는 Box<dyn FnMut() + 'static>과 같아서,
// 등록하는 클로저가 빌린 것을 품을 수 없기 때문입니다.
// 이 예제는 컴파일되는 두 가지 방법을 보여 줍니다.
//   1. Rc<RefCell<Counter>>를 클로저 안으로 옮김(move): 콜백이 살아 있는 동안 Counter도 산다.
//   2. Weak<RefCell<Counter>>를 옮김: Counter의 수명은 원래 소유자가 정하고, 콜백은 쓸 때마다 살아 있는지 묻는다.
// 2번은 C++에서 weak_ptr을 캡처하는 관용구와 같지만, C++에서는 [this]를 캡처해도 컴파일러가 막지 않습니다.
// Rc·Weak·RefCell 자체와, 소유자를 버린 뒤 upgrade가 None이 되는 것은 Part 0 ch14 예제 08(트리의 부모 링크)에서
// 봤습니다. 여기서 더하는 것은 'static 콜백을 등록하는 자리에서 두 방법 가운데 무엇을 고르는가입니다.
//
// 실행: cargo run -p faq-ch12-modern-cpp-enough --example 01_callbacks_own_state

use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Default)]
struct Counter {
    clicks: u32,
}

#[derive(Default)]
struct Button {
    // 필드 타입의 Box<dyn FnMut()>는 Box<dyn FnMut() + 'static>입니다(Reference "Default trait object lifetimes").
    // 함수 본문의 let 타입 표기에서는 라이프타임이 추론되므로 이 규칙이 적용되지 않습니다.
    handlers: Vec<Box<dyn FnMut()>>,
}

impl Button {
    fn on_click(&mut self, f: Box<dyn FnMut()>) {
        self.handlers.push(f);
    }

    fn click(&mut self) {
        for f in &mut self.handlers {
            f();
        }
    }
}

fn main() {
    println!("1. Rc를 옮겨 콜백이 공동 소유");
    let mut button = Button::default();
    {
        let counter = Rc::new(RefCell::new(Counter::default()));
        let shared = Rc::clone(&counter);
        button.on_click(Box::new(move || {
            let mut c = shared.borrow_mut();
            c.clicks += 1;
            println!("  클릭 수: {}", c.clicks);
        }));
        println!("  블록 안의 참조 수: {}", Rc::strong_count(&counter)); // 출력:   블록 안의 참조 수: 2
    } // counter 변수는 사라지지만 콜백이 든 Rc가 남아 Counter는 살아 있습니다.
    button.click(); // 출력:   클릭 수: 1
    button.click(); // 출력:   클릭 수: 2

    println!("2. Weak를 옮겨 살아 있는지 묻기");
    let mut button = Button::default();
    {
        let counter = Rc::new(RefCell::new(Counter::default()));
        let weak: Weak<RefCell<Counter>> = Rc::downgrade(&counter);
        button.on_click(Box::new(move || match weak.upgrade() {
            Some(c) => {
                let mut c = c.borrow_mut();
                c.clicks += 1;
                println!("  클릭 수: {}", c.clicks);
            }
            None => println!("  Counter가 이미 사라져 무시"),
        }));
        button.click(); // 출력:   클릭 수: 1
    } // 강한 참조가 counter 하나뿐이었으므로 Counter가 여기서 해제됩니다.
    button.click(); // 출력:   Counter가 이미 사라져 무시
}
