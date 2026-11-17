// 기대하는 에러: E0790 — cannot call associated function on trait without specifying the corresponding `impl` type
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0790_ambiguous_associated_fn.rs
//
// baby_name에는 self 매개변수가 없습니다. 메서드라면 인자의 타입을 보고 어느 구현인지 정할 수 있지만,
// self가 없는 연관 함수는 `Animal::baby_name()`만으로는 어느 타입의 구현을 부를지 알 수 없습니다.
// (Animal을 구현한 타입이 Dog 하나뿐이어도 마찬가지입니다.)
// 해결: 완전 정규화 문법 `<Dog as Animal>::baby_name()`으로 타입과 트레이트를 모두 적습니다.

trait Animal {
    fn baby_name() -> String;
}

struct Dog;

impl Dog {
    fn baby_name() -> String {
        String::from("바둑이")
    }
}

impl Animal for Dog {
    fn baby_name() -> String {
        String::from("강아지")
    }
}

fn main() {
    println!("타입 자신의 함수: {}", Dog::baby_name());
    println!("새끼 개를 부르는 말: {}", Animal::baby_name());
}
