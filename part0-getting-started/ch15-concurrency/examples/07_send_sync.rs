// Send와 Sync가 무엇을 허락하는지 보여 줍니다.
// - Send: 값의 소유권을 다른 스레드로 넘겨도 안전하다.
// - Sync: 여러 스레드가 같은 값을 참조(&T)로 동시에 써도 안전하다.
// 두 트레이트 모두 직접 구현하지 않아도, 구성 요소가 모두 Send/Sync이면 컴파일러가 자동으로 붙여 줍니다.
//
// 실행: cargo run -p p0-ch15-concurrency --example 07_send_sync

use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;
use std::thread;

// 직접 만든 구조체입니다. Send나 Sync를 구현한다고 따로 적지 않았습니다.
#[derive(Debug)]
struct Order {
    id: u32,
    items: Vec<String>,
}

// 같은 값 하나를 Arc로 감싸 두 스레드가 함께 읽게 하고, 각 스레드가 읽은 내용을 돌려받습니다.
// 여러 스레드가 같은 값을 동시에 참조하므로 T는 Sync여야 합니다. Arc<T>를 다른 스레드로
// 보내려면 T가 Send + Sync여야 한다는 것이 표준 라이브러리의 규칙이기 때문입니다.
// 'static은 "T 안에 금방 사라질 수 있는 지역 변수의 참조가 들어 있지 않다"는 뜻입니다.
// String, Vec처럼 데이터를 직접 소유하는 타입은 모두 'static을 만족합니다.
fn read_from_two_threads<T>(value: T) -> Vec<String>
where
    T: Debug + Send + Sync + 'static,
{
    let shared = Arc::new(value);
    let handles: Vec<_> = (1..=2)
        .map(|id| {
            let shared = Arc::clone(&shared);
            thread::spawn(move || format!("[스레드 {id}] 읽은 값: {shared:?}"))
        })
        .collect();
    // 만든 순서대로 join하므로 결과의 순서는 항상 같습니다.
    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

fn main() {
    println!("1. 필드가 모두 Send인 구조체는 자동으로 Send입니다");
    // u32와 Vec<String>이 모두 Send이므로 Order도 Send입니다. 그래서 스레드로 옮길 수 있습니다.
    let order = Order {
        id: 7,
        items: vec![String::from("커피"), String::from("빵")],
    };
    let handle = thread::spawn(move || {
        println!("    [새 스레드] 주문 {}: {:?}", order.id, order.items);
    });
    handle.join().unwrap();
    // 출력:     [새 스레드] 주문 7: ["커피", "빵"]

    println!("2. RefCell은 Send라서 통째로 다른 스레드에 넘길 수는 있습니다");
    // 소유권을 넘기면 RefCell을 쓰는 스레드는 한 번에 하나뿐이므로 안전합니다.
    let cell = RefCell::new(vec![1, 2, 3]);
    let handle = thread::spawn(move || {
        cell.borrow_mut().push(4);
        cell // 다 쓴 뒤 소유권을 돌려줍니다.
    });
    let cell = handle.join().unwrap();
    println!("    돌려받은 RefCell 안의 값: {:?}", cell.borrow());
    // 출력:     돌려받은 RefCell 안의 값: [1, 2, 3, 4]

    println!("3. 여러 스레드가 함께 읽으려면 Sync여야 합니다");
    // String, Vec, 직접 만든 Order는 모두 Sync이므로 아래 호출이 컴파일됩니다.
    // (Mutex<T>도 Sync입니다. 06_shared_counter.rs에서 Arc<Mutex<i32>>를 여러 스레드가 함께 썼습니다.)
    for line in read_from_two_threads(String::from("문자열")) {
        println!("    {line}");
    }
    for line in read_from_two_threads(vec![1, 2, 3]) {
        println!("    {line}");
    }
    let order = Order {
        id: 8,
        items: vec![String::from("차")],
    };
    for line in read_from_two_threads(order) {
        println!("    {line}");
    }
    // 아래 두 호출은 컴파일되지 않습니다(E0277).
    // read_from_two_threads(RefCell::new(0));
    //     → RefCell<i32>는 Sync가 아닙니다(compile_fail/e0277_refcell_is_not_sync.rs).
    // read_from_two_threads(std::rc::Rc::new(0));
    //     → Rc<i32>는 Send도 Sync도 아닙니다(compile_fail/e0277_rc_is_not_send.rs).
    // 출력:     [스레드 1] 읽은 값: "문자열"
    // 출력:     [스레드 2] 읽은 값: "문자열"
    // 출력:     [스레드 1] 읽은 값: [1, 2, 3]
    // 출력:     [스레드 2] 읽은 값: [1, 2, 3]
    // 출력:     [스레드 1] 읽은 값: Order { id: 8, items: ["차"] }
    // 출력:     [스레드 2] 읽은 값: Order { id: 8, items: ["차"] }
}
