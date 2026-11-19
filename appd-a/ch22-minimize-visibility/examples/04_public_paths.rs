//! 지침 4: 모듈 구조는 감추고 바깥에 보일 경로는 `pub use`로 정한다. 만드는 함수는 `pub(in …)`으로 묶는다.
//!
//! 표준 라이브러리의 반복자 어댑터와 같은 배치를 작게 따라 합니다. 1.100.0의 core 소스에서
//! `Map`은 비공개 모듈 `iter::adapters::map`에 정의되고, `pub use`로 `core::iter::Map`
//! (= `std::iter::Map`)에 올라옵니다. 만드는 함수 `Map::new`는 `pub(in crate::iter)`라서
//! `Iterator::map` 같은 `iter` 안의 코드만 부를 수 있습니다.
//!
//! - 비권장 `order_loose`: 하위 모듈이 모두 `pub mod`이고 상태 타입의 `new`도 `pub`입니다.
//!   사용자 코드가 내부 경로(`order_loose::states::paid::Paid`)를 적고, 결제 절차 없이
//!   "결제 완료" 주문을 만들어 배송합니다.
//! - 권장 `order`: 하위 모듈은 비공개이고, 쓰는 사람에게 필요한 타입만 `pub use`로
//!   `order::Cart`, `order::Paid`, `order::Shipped`에 올립니다. `Paid::new`와 `Shipped::new`는
//!   `pub(in crate::order)`라서 `Cart::checkout`과 `Paid::ship`만 부를 수 있습니다.
//!
//! 실행: cargo run -p er-ch22-minimize-visibility --example 04_public_paths

use std::any::type_name;

// ---- 비권장: 내부 모듈 구조와 생성자를 모두 공개 ----

pub mod order_loose {
    pub mod states {
        pub mod paid {
            use super::shipped::Shipped;

            pub struct Paid {
                pub items: usize,
                pub total: u32,
            }

            impl Paid {
                pub fn new(items: usize, total: u32) -> Paid {
                    Paid { items, total }
                }

                pub fn ship(self, tracking: &str) -> Shipped {
                    Shipped::new(self.total, tracking)
                }
            }
        }

        pub mod shipped {
            pub struct Shipped {
                pub total: u32,
                pub tracking: String,
            }

            impl Shipped {
                pub fn new(total: u32, tracking: &str) -> Shipped {
                    Shipped {
                        total,
                        tracking: tracking.to_string(),
                    }
                }
            }
        }
    }
}

// ---- 권장: 하위 모듈은 비공개, 공개 경로는 pub use로 ----

pub mod order {
    // 바깥에서 보이는 경로는 이 세 줄이 정합니다. 하위 모듈을 나누거나 합쳐도 이 경로는 그대로입니다.
    pub use cart::Cart;
    pub use states::{Paid, Shipped};

    mod cart {
        use super::states::Paid;

        #[derive(Default)]
        pub struct Cart {
            items: Vec<(String, u32)>,
        }

        impl Cart {
            pub fn new() -> Cart {
                Cart::default()
            }

            pub fn add(&mut self, name: &str, price: u32) {
                self.items.push((name.to_string(), price));
            }

            /// 결제 완료 상태(`Paid`)를 만드는 유일한 길입니다.
            pub fn checkout(&self, paid: u32) -> Result<Paid, String> {
                let total: u32 = self.items.iter().map(|(_, price)| price).sum();
                if paid < total {
                    return Err(format!("{total}원이 필요합니다"));
                }
                Ok(Paid::new(self.items.len(), total))
            }
        }
    }

    mod states {
        // 비공개 모듈 안의 pub use: 한 단계 위(order)에서 states::Paid로 쓸 수 있게 합니다.
        pub use paid::Paid;
        pub use shipped::Shipped;

        mod paid {
            use super::Shipped;

            pub struct Paid {
                items: usize,
                total: u32,
            }

            impl Paid {
                // order 안(cart, states)에서만 부를 수 있습니다. 바깥에서는 Cart::checkout을 거칩니다.
                pub(in crate::order) fn new(items: usize, total: u32) -> Paid {
                    Paid { items, total }
                }

                pub fn items(&self) -> usize {
                    self.items
                }

                pub fn total(&self) -> u32 {
                    self.total
                }

                pub fn ship(self, tracking: &str) -> Shipped {
                    Shipped::new(self.total, tracking)
                }
            }
        }

        mod shipped {
            pub struct Shipped {
                total: u32,
                tracking: String,
            }

            impl Shipped {
                pub(in crate::order) fn new(total: u32, tracking: &str) -> Shipped {
                    Shipped {
                        total,
                        tracking: tracking.to_string(),
                    }
                }

                pub fn total(&self) -> u32 {
                    self.total
                }

                pub fn tracking(&self) -> &str {
                    &self.tracking
                }
            }
        }
    }
}

/// 사용자 코드는 공개 경로 `order::Paid`로 타입을 적습니다. 정의된 하위 모듈은 몰라도 됩니다.
fn receipt(p: &order::Paid) -> String {
    format!("상품 {}개, {}원 결제 완료", p.items(), p.total())
}

fn main() {
    println!("1. 비권장: 내부 경로와 생성자가 공개");
    // 사용자 코드가 내부 모듈 구조(states::paid)를 경로로 적습니다.
    use order_loose::states::paid::Paid as LoosePaid;
    let fake = LoosePaid::new(1, 0); // 장바구니도 결제도 없이 "결제 완료"를 만듦
    let (items, total) = (fake.items, fake.total);
    println!("    결제 없이 만든 Paid: 상품 {items}개, {total}원");
    let shipped = fake.ship("FAKE-0001");
    println!(
        "    그대로 배송됨: {}원, 송장 {}",
        shipped.total, shipped.tracking
    );

    println!("2. 권장: 하위 모듈은 비공개, 타입은 pub use로 올림");
    let mut cart = order::Cart::new();
    cart.add("키보드", 59000);
    cart.add("마우스", 21000);
    for paid in [70000, 80000] {
        match cart.checkout(paid) {
            Ok(p) => {
                println!("    {paid}원 냄: {}", receipt(&p));
                let s = p.ship("KR-2026-1119");
                println!("    배송: {}원, 송장 {}", s.total(), s.tracking());
            }
            Err(e) => println!("    {paid}원 냄: 거절 - {e}"),
        }
    }
    // order::Paid::new(1, 0); // E0624: associated function `new` is private
    // fn show(_: &order::states::paid::Paid) {} // E0603: module `states` is private

    println!("3. 공개 경로와 정의된 곳");
    println!("    order::Paid     → {}", type_name::<order::Paid>());
    println!("    order::Shipped  → {}", type_name::<order::Shipped>());
    let std_map = type_name::<std::iter::Map<std::vec::IntoIter<u32>, fn(u32) -> u32>>();
    println!("    std::iter::Map  → {std_map}");
}
