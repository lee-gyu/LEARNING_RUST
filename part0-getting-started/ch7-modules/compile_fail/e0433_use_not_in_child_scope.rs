// 기대하는 에러: E0433 — cannot find module or crate `hosting` in this scope
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0433_use_not_in_child_scope.rs --out-dir /tmp/rust-ch7
//
// `use`는 그 use를 쓴 스코프(여기서는 크레이트 루트)에서만 바로 가기를 만듭니다.
// customer 모듈은 크레이트 루트와 다른 스코프이므로 그 안에서는 `hosting`이라는 이름을 모릅니다.
// (크레이트 루트의 use가 쓰이지 않았다는 unused_imports 경고도 함께 나옵니다.)

mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting;

mod customer {
    pub fn eat_at_restaurant() {
        hosting::add_to_waitlist();
    }
}

fn main() {
    customer::eat_at_restaurant();
}
