// 모듈 back_of_house의 내용입니다 (src/back_of_house.rs).
// 하위 모듈이 없으므로 src/back_of_house/ 디렉터리는 필요 없습니다.

// 공개 구조체지만 seasonal_fruit 필드는 비공개입니다.
// 바깥에서는 생성자 summer로만 만들 수 있고, 과일은 fruit 메서드로 읽기만 할 수 있습니다.
#[derive(Debug)]
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

    pub fn fruit(&self) -> &str {
        &self.seasonal_fruit
    }
}

// 공개 열거형은 모든 배리언트가 공개됩니다.
#[derive(Debug)]
pub enum Appetizer {
    Soup,
    Salad,
}
