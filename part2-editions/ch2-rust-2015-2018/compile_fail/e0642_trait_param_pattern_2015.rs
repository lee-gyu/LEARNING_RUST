// 기대하는 결과: 2015 에디션에서 E0642로 실패, 2018·2021·2024 에디션에서는 경고 없이 통과
// 핵심 메시지: error[E0642]: patterns aren't allowed in methods without bodies
//              help: give this argument a name or use an underscore to ignore it
// 함께 나오는 에러: E0425 두 개(cannot find value `w`/`h`). 컴파일러가 패턴을 버리고 나니 본문의 w, h가
//              가리킬 것이 없어졌기 때문입니다.
//
// 2018 에디션부터 본문이 있는 트레이트 메서드는 매개변수 자리에 반박 불가능한 패턴(여기서는 튜플
// 구조 분해)을 쓸 수 있습니다. 지금 문법에 익숙한 사람이 2015 에디션 크레이트를 고칠 때 만나는 에러입니다.
// 2015 에디션에서는 트레이트 메서드의 매개변수가 이름 없이 타입만 올 수도 있었기 때문에(익명 매개변수)
// 패턴을 받지 않았습니다. 메시지는 "methods without bodies"라고 하지만 이 메서드에는 본문이 있습니다.
// 고치는 방법(2015): 매개변수에는 이름을 주고 본문 안에서 구조 분해합니다.
//   fn rect(&self, size: (u32, u32)) -> u32 { let (w, h) = size; w * h }
// 이렇게 고친 코드는 모든 에디션에서 통과합니다.
//
// 확인(실패): rustc --edition 2015 compile_fail/e0642_trait_param_pattern_2015.rs --out-dir /tmp/rust-p2-ch2
// 확인(통과): rustc --edition 2024 compile_fail/e0642_trait_param_pattern_2015.rs --out-dir /tmp/rust-p2-ch2

trait Area {
    fn rect(&self, (w, h): (u32, u32)) -> u32 {
        // → 2015에서 E0642
        w * h
    }
}

struct Calc;

impl Area for Calc {}

fn main() {
    println!("{}", Calc.rect((3, 4)));
}
