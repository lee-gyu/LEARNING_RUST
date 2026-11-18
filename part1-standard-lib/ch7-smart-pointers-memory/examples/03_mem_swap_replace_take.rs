// 이 예제가 보여 주는 것: std::mem의 swap / replace / take로 &mut 뒤의 값을 옮기기,
//                        replace로 열거형 상태를 바꾸기, needs_drop
// 실행: cargo run -p p1-ch7-smart-pointers-memory --example 03_mem_swap_replace_take
//
// &mut로 빌린 자리에서 값을 그냥 꺼내 가면 그 자리에 "비어 있는 구멍"이 남습니다. 빌려준 쪽은 나중에
// 그 자리를 다시 읽거나 drop할 것이므로 Rust는 이것을 허락하지 않습니다(E0507, compile_fail 참고).
// swap, replace, take는 꺼내는 순간 다른 값을 채워 넣어 구멍을 남기지 않는 함수들입니다.

use std::mem;

// 화면에 보여 주는 버퍼(front)와 다음 장면을 그리는 버퍼(back)를 가진 이중 버퍼입니다.
struct DoubleBuffer {
    front: Vec<String>,
    back: Vec<String>,
}

impl DoubleBuffer {
    // 다 그린 back을 front로 올립니다. 원소를 하나도 복사하지 않고 두 Vec의 핸들만 맞바꿉니다.
    fn present(&mut self) {
        mem::swap(&mut self.front, &mut self.back);
        self.back.clear(); // 이전 장면을 지우되 용량은 남겨 다음 장면에 재사용합니다.
    }
}

enum Order {
    Cart(Vec<String>),
    Paid {
        items: Vec<String>,
        amount: u32,
    },
    Shipped {
        items: Vec<String>,
        tracking: String,
    },
}

impl Order {
    fn describe(&self) -> String {
        match self {
            Order::Cart(items) => format!("장바구니 {items:?}"),
            Order::Paid { items, amount } => format!("결제 완료 {items:?}, {amount}원"),
            Order::Shipped { items, tracking } => format!("배송 중 {items:?}, 송장 {tracking}"),
        }
    }
}

struct Shop {
    order: Order,
}

impl Shop {
    // `match self.order { Order::Cart(items) => ... }`라고 쓰면 E0507입니다.
    // &mut self 뒤의 self.order에서 items(Vec)를 꺼내 옮기려 하기 때문입니다.
    // mem::replace로 먼저 자리 표시 값(빈 장바구니)을 넣고, 꺼낸 예전 값을 소유한 채로 match합니다.
    // 빈 Vec은 힙을 할당하지 않으므로 자리 표시 값을 만드는 비용은 거의 없습니다.
    fn pay(&mut self, amount: u32) {
        let current = mem::replace(&mut self.order, Order::Cart(Vec::new()));
        self.order = match current {
            Order::Cart(items) => Order::Paid { items, amount },
            other => other, // 결제할 수 없는 상태면 꺼냈던 값을 그대로 되돌려 놓습니다.
        };
    }

    fn ship(&mut self, tracking: &str) {
        let current = mem::replace(&mut self.order, Order::Cart(Vec::new()));
        self.order = match current {
            Order::Paid { items, .. } => Order::Shipped {
                items,
                tracking: tracking.to_string(),
            },
            other => other,
        };
    }
}

// 쌓아 둔 로그를 한꺼번에 넘겨주는 기록기입니다.
struct Logger {
    pending: Vec<String>,
}

impl Logger {
    // mem::take는 값을 꺼내 돌려주고, 그 자리에 Default::default()를 남깁니다. Vec이면 빈 Vec입니다.
    fn flush(&mut self) -> Vec<String> {
        mem::take(&mut self.pending)
    }
}

struct Noisy;

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("[drop] Noisy");
    }
}

fn main() {
    println!("== 1. swap: 두 자리의 값을 맞바꾸기 ==");
    let mut a = String::from("왼쪽");
    let mut b = String::from("오른쪽");
    mem::swap(&mut a, &mut b);
    println!("a = {a}, b = {b}"); // 출력: a = 오른쪽, b = 왼쪽

    let mut screen = DoubleBuffer {
        front: vec![String::from("장면 1")],
        back: Vec::with_capacity(8),
    };
    screen.back.push(String::from("장면 2"));
    let back_heap = screen.back.as_ptr(); // 주소 값 자체는 실행마다 다르므로 비교에만 씁니다.
    screen.present();
    println!("front = {:?}, back = {:?}", screen.front, screen.back); // 출력: front = ["장면 2"], back = []
    println!(
        "front가 예전 back의 힙 버퍼를 그대로 쓰는가? {}",
        screen.front.as_ptr() == back_heap
    ); // 출력: front가 예전 back의 힙 버퍼를 그대로 쓰는가? true

    println!("\n== 2. replace: 새 값을 넣고 예전 값을 돌려받기 ==");
    let mut status = String::from("대기");
    let old = mem::replace(&mut status, String::from("실행 중"));
    println!("예전 = {old}, 지금 = {status}"); // 출력: 예전 = 대기, 지금 = 실행 중

    println!("\n== 3. replace로 상태 전이 (&mut self 뒤의 열거형) ==");
    let mut shop = Shop {
        order: Order::Cart(vec![String::from("사과"), String::from("배")]),
    };
    println!("{}", shop.order.describe()); // 출력: 장바구니 ["사과", "배"]
    shop.ship("A-1"); // 아직 결제 전이라 아무 일도 일어나지 않습니다.
    println!("{}", shop.order.describe()); // 출력: 장바구니 ["사과", "배"]
    shop.pay(5000);
    println!("{}", shop.order.describe()); // 출력: 결제 완료 ["사과", "배"], 5000원
    shop.ship("A-1");
    println!("{}", shop.order.describe()); // 출력: 배송 중 ["사과", "배"], 송장 A-1
    // Vec<String>은 한 번도 복제되지 않고 Cart → Paid → Shipped로 옮겨 다녔습니다.

    println!("\n== 4. take: 꺼내고 기본값 남기기 ==");
    let mut logger = Logger {
        pending: Vec::with_capacity(16),
    };
    logger.pending.push(String::from("시작"));
    logger.pending.push(String::from("완료"));
    let batch = logger.flush();
    println!(
        "꺼낸 것: {batch:?} (용량 {}), 남은 것: {:?} (용량 {})",
        batch.capacity(),
        logger.pending,
        logger.pending.capacity()
    ); // 출력: 꺼낸 것: ["시작", "완료"] (용량 16), 남은 것: [] (용량 0)
    // 힙 버퍼가 통째로 batch에게 넘어갔습니다. 버퍼를 남겨 재사용하고 싶다면 drain(..)을 씁니다.

    // Default를 구현한 타입이면 무엇이든 됩니다. 정수는 0, String은 빈 문자열, Option은 None입니다.
    let mut count = 42;
    let mut title = String::from("제목");
    let mut slot = Some(7);
    let (c, t, s) = (
        mem::take(&mut count),
        mem::take(&mut title),
        mem::take(&mut slot),
    );
    println!("꺼낸 값: {c}, {t:?}, {s:?}"); // 출력: 꺼낸 값: 42, "제목", Some(7)
    println!("남은 값: {count}, {title:?}, {slot:?}"); // 출력: 남은 값: 0, "", None
    // Option에 쓴 mem::take는 slot.take()와 같은 일을 합니다(Part 1 ch2).

    println!("\n== 5. needs_drop: 버릴 때 할 일이 있는 타입인가 ==");
    println!("i32: {}", mem::needs_drop::<i32>()); // 출력: i32: false
    println!("(u8, &str): {}", mem::needs_drop::<(u8, &str)>()); // 출력: (u8, &str): false
    println!("String: {}", mem::needs_drop::<String>()); // 출력: String: true
    println!("Vec<i32>: {}", mem::needs_drop::<Vec<i32>>()); // 출력: Vec<i32>: true
    println!("Noisy: {}", mem::needs_drop::<Noisy>()); // 출력: Noisy: true
    // 참조와 정수는 버릴 때 아무 일도 하지 않습니다. 힙을 가진 타입과 Drop을 구현한 타입만 true입니다.
    let noisy = Noisy;
    drop(noisy); // 출력: [drop] Noisy
    println!("끝"); // 출력: 끝
}
