//! receipt 1.0.0과 money 0.4.0을 함께 쓰는 카페 앱입니다.
//! 프로그램에 money가 두 버전 들어 있지만, money의 값이 receipt와의 경계를 넘지 않으므로 빌드됩니다.

use std::any::{type_name, type_name_of_val};

fn main() {
    println!(
        "[cafe-separate] receipt {}, 앱의 money {}",
        receipt::VERSION,
        money::VERSION
    );

    // 앱이 직접 쓰는 money 0.4: 0.4에서 생긴 달러 금액
    let tourist_price = money::usd(350);
    println!("  관광객 메뉴판의 아메리카노: {tourist_price}");

    // receipt에는 원 단위 정수만 넘깁니다. money 0.3의 값은 receipt 안에서 만들어집니다.
    let mut bill = receipt::Receipt::new();
    bill.add_won("아메리카노", 4500);
    bill.add_won("마들렌", 3200);
    print!("{}", bill.render());

    // receipt가 돌려준 합계는 money 0.3의 Money이고, 앱의 money::Money는 0.4의 것입니다.
    // 둘은 다른 타입이지만 type_name은 버전을 보여 주지 않으므로 같은 문자열이 나옵니다.
    let total = bill.total();
    println!("  합계의 타입:     {}", type_name_of_val(&total));
    println!("  앱의 Money 타입: {}", type_name::<money::Money>());
}
