// 기대하는 에러: E0616 — field `seasonal_fruit` of struct `Breakfast` is private
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0616_private_field.rs --out-dir /tmp/rust-ch7
//
// `pub struct`는 구조체 이름만 공개합니다. 필드는 하나하나 pub을 붙여야 공개되며,
// pub이 없는 seasonal_fruit는 back_of_house 바깥에서 읽을 수도, 바꿀 수도 없습니다.

mod back_of_house {
    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String,
    }

    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("복숭아"),
            }
        }
    }
}

fn main() {
    let mut meal = back_of_house::Breakfast::summer("호밀빵");
    meal.toast = String::from("통밀빵"); // 공개 필드라서 괜찮습니다.
    meal.seasonal_fruit = String::from("블루베리"); // 비공개 필드라서 에러입니다.
    println!("{} 토스트", meal.toast);
}
