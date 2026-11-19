// 지침 5: 서로 다른 타입의 구현을 한 번에 섞어 받아야 하거나 실행 중에 골라야 한다면,
// 매개변수를 트레이트 객체(`&dyn Trait`, `&[Box<dyn Trait>]`, `&mut dyn FnMut(..)`)로 선언합니다.
//
// 제네릭 매개변수 `T: Trait`는 호출 한 번에 구체 타입 하나로 정해집니다. 그래서 `&[T]`로 받으면
// 부르는 쪽은 섞여 있는 규칙을 타입별로 나눠 따로 불러야 하고, 이 예제에서는 순서가 바뀌어
// 최종 가격이 달라집니다.
//
// 실행: cargo run -p er-ch2-express-common-behavior --example 05_trait_object_param

trait PriceRule {
    fn apply(&self, price: u32) -> u32;
    fn label(&self) -> String;
}

/// 몇 퍼센트 깎아 주는 규칙.
struct PercentOff(u32);
/// 정해진 금액을 깎아 주는 규칙.
struct AmountOff(u32);

impl PriceRule for PercentOff {
    fn apply(&self, price: u32) -> u32 {
        price * (100 - self.0) / 100
    }
    fn label(&self) -> String {
        format!("{}% 할인", self.0)
    }
}

impl PriceRule for AmountOff {
    fn apply(&self, price: u32) -> u32 {
        price.saturating_sub(self.0)
    }
    fn label(&self) -> String {
        format!("{}원 할인", self.0)
    }
}

/// 비권장: 제네릭 슬라이스로 받습니다. `rules`의 원소는 모두 같은 타입 `R`이어야 합니다.
fn apply_rules_generic<R: PriceRule>(price: u32, rules: &[R]) -> u32 {
    rules.iter().fold(price, |p, rule| rule.apply(p))
}

/// 권장: 트레이트 객체의 슬라이스로 받습니다. 타입이 다른 규칙을 정해진 순서대로 적용합니다.
/// 진행 상황을 알려 줄 콜백도 트레이트 객체 `&mut dyn FnMut`으로 받을 수 있습니다.
fn apply_rules(price: u32, rules: &[Box<dyn PriceRule>], report: &mut dyn FnMut(&str, u32)) -> u32 {
    let mut p = price;
    for rule in rules {
        p = rule.apply(p);
        report(&rule.label(), p);
    }
    p
}

/// 규칙 하나를 미리 보는 제네릭 함수. `?Sized`를 붙여서 구체 타입(`&AmountOff`)과
/// 트레이트 객체(`&dyn PriceRule`)를 모두 받습니다. 붙이지 않으면 트레이트 객체를 넘길 때
/// E0277이 납니다(compile_fail/e0277_dyn_to_implicit_sized.rs).
fn preview<R: PriceRule + ?Sized>(rule: &R, price: u32) -> String {
    format!("{} → {}원", rule.label(), rule.apply(price))
}

/// 설정 문자열에서 규칙을 만듭니다. 어떤 타입이 나올지는 실행 중에 정해집니다.
fn parse_rule(text: &str) -> Option<Box<dyn PriceRule>> {
    let (kind, value) = text.split_once(':')?;
    let value: u32 = value.parse().ok()?;
    match kind {
        "percent" => Some(Box::new(PercentOff(value))),
        "amount" => Some(Box::new(AmountOff(value))),
        _ => None,
    }
}

fn main() {
    let price = 20_000;
    // 운영자가 정한 순서: 3000원 쿠폰 → 10% 회원 할인 → 500원 포인트
    let config = "amount:3000,percent:10,amount:500";

    println!("1. 비권장: 타입별로 나눠서 두 번 부릅니다");
    // 한 슬라이스에 AmountOff와 PercentOff를 섞을 수 없으므로 타입별로 나눕니다.
    let amounts = [AmountOff(3000), AmountOff(500)];
    let percents = [PercentOff(10)];
    let total = apply_rules_generic(apply_rules_generic(price, &amounts), &percents);
    println!("    최종 가격 {total}원");
    // 출력:     최종 가격 14850원
    // 500원 할인이 10% 할인보다 먼저 적용되어 운영자가 정한 순서와 다른 값이 나왔습니다.

    println!("2. 권장: 트레이트 객체로 받아 정해진 순서대로 적용합니다");
    let rules: Vec<Box<dyn PriceRule>> = config.split(',').filter_map(parse_rule).collect();
    let mut steps = Vec::new();
    let total = apply_rules(price, &rules, &mut |label, p| {
        steps.push(format!("{label}: {p}원"));
    });
    for step in &steps {
        println!("    {step}");
    }
    println!("    최종 가격 {total}원");
    // 출력:     3000원 할인: 17000원
    // 출력:     10% 할인: 15300원
    // 출력:     500원 할인: 14800원
    // 출력:     최종 가격 14800원

    println!("3. ?Sized를 붙인 제네릭 함수는 구체 타입과 트레이트 객체를 모두 받습니다");
    println!("    구체 타입: {}", preview(&AmountOff(1000), price));
    println!("    트레이트 객체: {}", preview(rules[1].as_ref(), price));
    // 출력:     구체 타입: 1000원 할인 → 19000원
    // 출력:     트레이트 객체: 10% 할인 → 18000원
}
