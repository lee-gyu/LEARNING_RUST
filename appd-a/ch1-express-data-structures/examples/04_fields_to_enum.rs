// 지침: "이 필드는 저 필드가 X일 때만 의미가 있다"는 주석이 붙는 필드 조합 대신,
//       경우마다 필요한 데이터만 가진 열거형을 씁니다. 무효한 상태를 아예 만들 수 없게 됩니다.
// 비권장: DiscountFields { enabled, is_rate, rate_percent, amount_won }
// 권장:   enum Discount { None, Rate { percent }, Amount { won } }
//
// 실행: cargo run -p er-ch1-express-data-structures --example 04_fields_to_enum

const PRICE: u32 = 20_000;

// ---- 비권장 ----------------------------------------------------------------

#[derive(Debug)]
struct DiscountFields {
    enabled: bool,
    is_rate: bool,     // enabled일 때만 의미가 있습니다.
    rate_percent: u32, // enabled && is_rate일 때만 의미가 있습니다.
    amount_won: u32,   // enabled && !is_rate일 때만 의미가 있습니다.
}

// 화면에 보여 줄 설명입니다. 정액 할인이 먼저 눈에 띄도록 amount_won부터 확인했습니다.
fn describe_fields(d: &DiscountFields) -> String {
    if !d.enabled {
        String::from("할인 없음")
    } else if d.amount_won > 0 {
        format!("{}원 할인", d.amount_won)
    } else {
        format!("{}% 할인", d.rate_percent)
    }
}

// 실제 결제 금액입니다. 다른 사람이 작성해서 is_rate부터 확인합니다.
// 필드 조합이 올바르다면 두 함수는 같은 결론을 내지만, 올바르지 않은 조합에서는 갈라집니다.
fn pay_fields(price: u32, d: &DiscountFields) -> u32 {
    if !d.enabled {
        price
    } else if d.is_rate {
        price.saturating_sub(price * d.rate_percent / 100)
    } else {
        price.saturating_sub(d.amount_won)
    }
}

// ---- 권장 ------------------------------------------------------------------

// 경우마다 필요한 값만 담습니다. "정률인데 금액도 있는" 값은 만들 방법이 없습니다.
#[derive(Debug)]
enum Discount {
    None,
    Rate { percent: u32 },
    Amount { won: u32 },
}

impl Discount {
    fn describe(&self) -> String {
        match self {
            Discount::None => String::from("할인 없음"),
            Discount::Rate { percent } => format!("{percent}% 할인"),
            Discount::Amount { won } => format!("{won}원 할인"),
        }
    }

    fn pay(&self, price: u32) -> u32 {
        match self {
            Discount::None => price,
            // percent가 100을 넘는 값은 이 타입으로도 만들 수 있으므로 0 아래로 내려가지 않게 합니다.
            Discount::Rate { percent } => price.saturating_sub(price * percent / 100),
            Discount::Amount { won } => price.saturating_sub(*won),
        }
    }
}

// 같은 값을 두 함수가 어떻게 해석하는지 나란히 출력합니다.
fn show_fields(d: &DiscountFields) {
    let description = describe_fields(d);
    let paid = pay_fields(PRICE, d);
    println!("  {d:?}");
    println!("    설명: {description} / 결제: {paid}원");
}

fn show(discount: &Discount) {
    let description = discount.describe();
    let paid = discount.pay(PRICE);
    println!("  {discount:?}: {description} / 결제: {paid}원");
}

fn main() {
    println!("[비권장] 필드 조합 (정가 {PRICE}원)");
    // 관리 화면에서 3000원 정액 할인을 만들었습니다.
    let mut d = DiscountFields {
        enabled: true,
        is_rate: false,
        rate_percent: 0,
        amount_won: 3000,
    };
    show_fields(&d);
    // 출력:
    //   DiscountFields { enabled: true, is_rate: false, rate_percent: 0, amount_won: 3000 }
    //     설명: 3000원 할인 / 결제: 17000원

    // 정률 할인으로 바꾸려고 is_rate만 켜고, 비율을 넣기 전에 저장했습니다.
    // 남은 amount_won을 지워야 한다는 규칙은 주석에만 있으므로 아무도 막지 않습니다.
    d.is_rate = true;
    show_fields(&d);
    // 출력:
    //   DiscountFields { enabled: true, is_rate: true, rate_percent: 0, amount_won: 3000 }
    //     설명: 3000원 할인 / 결제: 20000원

    println!("[권장] 데이터를 가진 열거형");
    // 정률로 바꾸는 일은 값 전체를 새 배리언트로 바꾸는 것입니다. 옛 금액이 남을 자리가 없습니다.
    let mut discount = Discount::Amount { won: 3000 };
    show(&discount);
    discount = Discount::Rate { percent: 10 };
    show(&discount);
    discount = Discount::None;
    show(&discount);
    // 출력:
    //   Amount { won: 3000 }: 3000원 할인 / 결제: 17000원
    //   Rate { percent: 10 }: 10% 할인 / 결제: 18000원
    //   None: 할인 없음 / 결제: 20000원
}
