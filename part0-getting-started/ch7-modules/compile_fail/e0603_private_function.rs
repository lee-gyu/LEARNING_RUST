// 기대하는 에러: E0603 — function `add_to_waitlist` is private
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0603_private_function.rs --out-dir /tmp/rust-ch7
//
// `pub mod hosting`으로 모듈은 공개했지만, 그 안의 함수에는 pub을 붙이지 않았습니다.
// 모듈의 pub은 "모듈 이름을 바깥에서 부를 수 있다"는 뜻일 뿐, 안의 항목까지 공개하지는 않습니다.

mod front_of_house {
    pub mod hosting {
        fn add_to_waitlist() {}
    }
}

fn main() {
    front_of_house::hosting::add_to_waitlist();
}
