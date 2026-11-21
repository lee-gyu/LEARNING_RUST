// Q3.4 — 메모리 누수는 안전한 코드에서도 만들 수 있다
//
// 버려질 때 메시지를 출력하는 `Tracked` 값을 세 가지 방법으로 "잃어버립니다". 세 방법 모두
// `unsafe` 없이 컴파일되고, 세 값 모두 소멸자가 실행되지 않습니다. 누수는 메모리 안전성
// 위반이 아니기 때문입니다(해제되지 않은 메모리를 잘못 읽거나 쓰는 일은 생기지 않습니다).
//
//   1. `mem::forget`: 값을 버리지 않고 잊어버림
//   2. `Box::leak`: 힙의 값을 프로그램이 끝날 때까지 두고 `&'static mut` 참조를 얻음
//   3. `Rc` 순환 참조: 서로를 가리키는 두 값의 참조 수가 0이 되지 않음(Part 0 ch14 2.10절)
//
// 실행: cargo run -p faq-ch3-safety-limits --example 01_leaks_are_safe

use std::cell::RefCell;
use std::rc::Rc;

/// 버려질 때 이름을 출력합니다.
struct Tracked {
    name: &'static str,
    next: RefCell<Option<Rc<Tracked>>>,
}

impl Tracked {
    fn new(name: &'static str) -> Tracked {
        Tracked {
            name,
            next: RefCell::new(None),
        }
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        println!("  {} 버려짐", self.name);
    }
}

fn main() {
    println!("0. 비교: 보통은 스코프가 끝날 때 버려짐");
    {
        let _normal = Tracked::new("보통 값");
    }
    // 출력:   보통 값 버려짐

    println!("1. mem::forget");
    let forgotten = Tracked::new("잊은 값");
    std::mem::forget(forgotten);
    println!("  (forget 뒤에 아무것도 출력되지 않음)");
    // 출력:   (forget 뒤에 아무것도 출력되지 않음)

    println!("2. Box::leak");
    let leaked: &'static mut Tracked = Box::leak(Box::new(Tracked::new("새어 나간 값")));
    println!("  새어 나간 값을 계속 쓸 수 있음: {}", leaked.name);
    // 출력:   새어 나간 값을 계속 쓸 수 있음: 새어 나간 값

    println!("3. Rc 순환 참조");
    {
        let a = Rc::new(Tracked::new("A"));
        let b = Rc::new(Tracked::new("B"));
        *a.next.borrow_mut() = Some(Rc::clone(&b)); // A → B
        *b.next.borrow_mut() = Some(Rc::clone(&a)); // B → A
        println!(
            "  A의 참조 수 {}, B의 참조 수 {}",
            Rc::strong_count(&a),
            Rc::strong_count(&b)
        );
        // 출력:   A의 참조 수 2, B의 참조 수 2
    } // 지역 변수 a, b가 사라져도 서로가 서로를 붙잡고 있어 참조 수가 1에서 멈춥니다.
    println!("  (스코프가 끝났지만 A도 B도 버려지지 않음)");
    // 출력:   (스코프가 끝났지만 A도 B도 버려지지 않음)

    println!("끝: 잊은 값, 새어 나간 값, A, B는 프로그램이 끝날 때까지 버려지지 않습니다");
    // 출력: 끝: 잊은 값, 새어 나간 값, A, B는 프로그램이 끝날 때까지 버려지지 않습니다
    // 프로그램이 끝나면 운영체제가 프로세스의 메모리를 통째로 거두지만, 소멸자는 실행되지 않습니다.
}
