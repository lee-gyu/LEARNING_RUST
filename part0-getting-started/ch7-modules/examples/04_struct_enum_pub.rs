// 구조체와 열거형은 `pub`의 효과가 다릅니다.
// - `pub struct`는 구조체 이름만 공개합니다. 필드는 하나하나 `pub`을 붙여야 공개됩니다.
// - `pub enum`은 열거형을 공개하면 모든 배리언트가 함께 공개됩니다.
//
// 실행: cargo run -p p0-ch7-modules --example 04_struct_enum_pub

mod back_of_house {
    // 손님은 빵 종류(toast)를 고를 수 있지만, 곁들이는 과일은 주방이 그날 재고를 보고 정합니다.
    // 그래서 toast만 공개하고 seasonal_fruit는 비공개로 둡니다.
    #[derive(Debug)]
    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String,
    }

    impl Breakfast {
        // 비공개 필드가 있는 구조체는 바깥에서 `Breakfast { ... }`로 직접 만들 수 없습니다.
        // 비공개 필드에 넣을 값을 바깥에서 정할 수 없기 때문입니다.
        // 그래서 값을 만들어 주는 공개 연관 함수(생성자)를 제공해야 합니다.
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("복숭아"),
            }
        }

        // 비공개 필드를 바깥에서 "읽기만" 하게 하려면 공개 메서드를 둡니다.
        // 바깥 코드는 과일을 볼 수는 있어도 바꿀 수는 없습니다.
        pub fn fruit(&self) -> &str {
            &self.seasonal_fruit
        }
    }

    // 모든 필드가 pub이면 바깥에서도 구조체 리터럴로 바로 만들 수 있습니다.
    pub struct Drink {
        pub name: String,
        pub size_ml: u32,
    }

    // 열거형은 pub 하나로 모든 배리언트가 공개됩니다.
    // 배리언트를 쓸 수 없는 열거형은 쓸모가 없으므로 이렇게 정해져 있습니다.
    #[derive(Debug)]
    pub enum Appetizer {
        Soup,
        Salad,
    }
}

fn main() {
    println!("1. 공개 필드는 바깥에서 읽고 바꿀 수 있습니다");
    let mut meal = back_of_house::Breakfast::summer("호밀빵");
    println!("  주문: {} 토스트", meal.toast); // 출력:   주문: 호밀빵 토스트
    meal.toast = String::from("통밀빵"); // 손님이 마음을 바꿨습니다.
    println!("  변경: {} 토스트", meal.toast); // 출력:   변경: 통밀빵 토스트

    println!("2. 비공개 필드는 공개 메서드를 통해서만 읽을 수 있습니다");
    println!("  곁들임 과일: {}", meal.fruit()); // 출력:   곁들임 과일: 복숭아
    // 아래 줄의 주석을 풀면 E0616(필드가 비공개) 에러가 납니다. compile_fail/e0616_private_field.rs 참고.
    // meal.seasonal_fruit = String::from("블루베리");

    // {:?}로 출력하는 것은 derive(Debug)가 만든 코드라서 비공개 필드도 보입니다.
    println!("  {meal:?}"); // 출력:   Breakfast { toast: "통밀빵", seasonal_fruit: "복숭아" }

    println!("3. 모든 필드가 공개된 구조체는 리터럴로 만들 수 있습니다");
    let drink = back_of_house::Drink {
        name: String::from("아이스티"),
        size_ml: 350,
    };
    println!("  음료: {} {}ml", drink.name, drink.size_ml); // 출력:   음료: 아이스티 350ml

    println!("4. 공개 열거형의 배리언트는 모두 공개입니다");
    let order1 = back_of_house::Appetizer::Soup;
    let order2 = back_of_house::Appetizer::Salad;
    for order in [order1, order2] {
        let price = match order {
            back_of_house::Appetizer::Soup => 4000,
            back_of_house::Appetizer::Salad => 5500,
        };
        println!("  {order:?}: {price}원");
    }
    // 출력:   Soup: 4000원
    // 출력:   Salad: 5500원
}
