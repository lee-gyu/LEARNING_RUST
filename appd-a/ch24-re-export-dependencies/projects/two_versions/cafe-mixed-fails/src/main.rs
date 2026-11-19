//! 일부러 빌드에 실패하는 카페 앱입니다(E0308, E0277).
//! 앱이 의존하는 money 0.4의 값과 트레이트 구현을, money 0.3을 기대하는 receipt 1.0.0에 넘깁니다.

use std::fmt;

/// 카페라테. 앱이 의존하는 money(0.4)의 `Priced`를 구현합니다.
struct Latte;

impl fmt::Display for Latte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("카페라테")
    }
}

impl money::Priced for Latte {
    fn price(&self) -> money::Money {
        money::won(5000)
    }
}

fn main() {
    println!(
        "[cafe-mixed-fails] 관광객 메뉴판의 아메리카노: {}",
        money::usd(350)
    );

    let mut bill = receipt::Receipt::new();
    bill.add_won("아메리카노", 4500);
    bill.add("마들렌", money::won(3200)); // E0308: money 0.4의 Money를 넘김
    bill.add_item(&Latte); // E0277: money 0.4의 Priced만 구현함
    print!("{}", bill.render());
}
