// 기대하는 에러: E0277 — the size for values of type `dyn PriceRule` cannot be known at compilation time
// 핵심 메시지: required by an implicit `Sized` bound in `preview`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_dyn_to_implicit_sized.rs --out-dir /tmp/er-ch2
//
// 타입 매개변수에는 따로 적지 않아도 `Sized` 바운드가 붙습니다. 그래서 `preview<R: PriceRule>`는
// `&R`로 참조를 받는데도 `R`이 크기를 아는 타입이어야 하고, 트레이트 객체 `dyn PriceRule`은
// 들어올 수 없습니다. 실행 중에 고른 규칙(`Box<dyn PriceRule>`)을 가진 부르는 쪽은 이 함수를
// 쓸 수 없게 됩니다.
// 해결: 받는 쪽에 `?Sized`를 붙여 `R: PriceRule + ?Sized`로 바꿉니다
//       (examples/05_trait_object_param.rs의 `preview`). 첫 번째 help(메서드 호출을 지우라는 것)를
//       따르면 `Box<dyn PriceRule>`을 `&R` 자리에 넘기게 되어 E0308이 나므로, 두 번째 help를 따릅니다.

trait PriceRule {
    fn apply(&self, price: u32) -> u32;
}

struct AmountOff(u32);

impl PriceRule for AmountOff {
    fn apply(&self, price: u32) -> u32 {
        price.saturating_sub(self.0)
    }
}

fn preview<R: PriceRule>(rule: &R, price: u32) -> u32 {
    rule.apply(price)
}

fn main() {
    let chosen: Box<dyn PriceRule> = Box::new(AmountOff(500));
    println!("{}", preview(chosen.as_ref(), 10_000)); // E0277
}
