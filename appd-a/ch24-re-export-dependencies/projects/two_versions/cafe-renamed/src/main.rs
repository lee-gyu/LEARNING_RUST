//! receipt 1.0.0을 고치지 않고 cafe-mixed-fails를 빌드되게 만든 카페 앱입니다.
//! `Cargo.toml`에서 money 0.3을 `money03`이라는 이름으로 직접 의존하고,
//! receipt와 주고받는 값은 `money03`으로, 앱 자신의 일은 `money`(0.4)로 합니다.

use std::fmt;

struct Latte;

impl fmt::Display for Latte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("카페라테")
    }
}

// receipt가 요구하는 0.3의 Priced를 구현합니다.
impl money03::Priced for Latte {
    fn price(&self) -> money03::Money {
        money03::won(5000)
    }
}

fn main() {
    println!(
        "[cafe-renamed] receipt {}, money03 {}, money {}",
        receipt::VERSION,
        money03::VERSION,
        money::VERSION
    );
    println!("  관광객 메뉴판의 아메리카노: {}", money::usd(350));

    let mut bill = receipt::Receipt::new();
    bill.add_won("아메리카노", 4500);
    bill.add("마들렌", money03::won(3200));
    bill.add_item(&Latte);
    print!("{}", bill.render());
}
