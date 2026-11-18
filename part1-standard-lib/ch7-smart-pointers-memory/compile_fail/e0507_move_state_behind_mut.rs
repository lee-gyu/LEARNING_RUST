// 기대하는 에러: error[E0507]: cannot move out of `self.order` as enum variant `Cart` which is behind a mutable reference
// 핵심 메시지: data moved here / ...and here
//             help: consider borrowing the pattern binding (ref items)
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0507_move_state_behind_mut.rs --out-dir /tmp/rust-p1-ch7
//
// match self.order는 self.order 자리를 그대로 들여다봅니다. 그런데 갈래의 패턴 Order::Cart(items)는
// 안의 Vec을 items로 옮겨 가려 합니다. self는 &mut로 빌린 것이라, 옮겨 가면 호출한 쪽의 Shop에
// 비어 있는 구멍이 남습니다(바로 다음 줄에서 다시 채운다 해도 컴파일러는 그 사이를 허락하지 않습니다).
// help:가 제안하는 ref items는 Vec을 빌리기만 하므로 Order::Paid { items, .. }에 넣을 수 없어 E0308이 됩니다.
// 고치는 방법: mem::replace로 자리 표시 값을 먼저 넣고, 꺼낸 예전 값을 소유한 채로 match합니다.
//   let current = std::mem::replace(&mut self.order, Order::Cart(Vec::new()));
//   self.order = match current { Order::Cart(items) => Order::Paid { items, amount }, other => other };
// Order가 Default를 구현했다면 std::mem::take(&mut self.order)도 됩니다(examples/03_mem_swap_replace_take.rs).

enum Order {
    Cart(Vec<String>),
    Paid { items: Vec<String>, amount: u32 },
}

struct Shop {
    order: Order,
}

impl Shop {
    fn pay(&mut self, amount: u32) {
        self.order = match self.order {
            Order::Cart(items) => Order::Paid { items, amount },
            other => other,
        };
    }
}

fn main() {
    let mut shop = Shop {
        order: Order::Cart(vec![String::from("사과")]),
    };
    shop.pay(1000);
    if let Order::Paid { items, amount } = &shop.order {
        println!("{items:?}, {amount}원");
    }
}
