//! receipt 1.1.0이 재내보낸 money(0.3)로 receipt와 주고받는 값을 만들고,
//! 앱 자신의 일에는 앱이 의존하는 money 0.4를 쓰는 카페 앱입니다.
//! cafe-mixed-fails에서 receipt로 넘어가는 값과 트레이트의 `money::`를 `receipt::money::`로 바꿨습니다.

use std::fmt;

struct Latte;

impl fmt::Display for Latte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("카페라테")
    }
}

// 재내보낸 경로의 트레이트이므로 receipt가 요구하는 바로 그 버전의 Priced입니다.
impl receipt::money::Priced for Latte {
    fn price(&self) -> receipt::money::Money {
        receipt::money::won(5000)
    }
}

fn main() {
    println!(
        "[cafe-re-exported] receipt {}, receipt의 money {}, 앱의 money {}",
        receipt::VERSION,
        receipt::money::VERSION,
        money::VERSION
    );
    println!("  관광객 메뉴판의 아메리카노: {}", money::usd(350));

    let mut bill = receipt::Receipt::new();
    bill.add_won("아메리카노", 4500);
    bill.add("마들렌", receipt::money::won(3200));
    bill.add_item(&Latte);
    print!("{}", bill.render());
}
