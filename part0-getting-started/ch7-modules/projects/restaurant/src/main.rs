// 바이너리 크레이트의 루트입니다 (src/main.rs).
// 이 패키지에는 src/lib.rs도 있으므로 크레이트가 두 개입니다. 둘 다 이름이 restaurant입니다.
//
// 같은 패키지의 라이브러리 크레이트는 Cargo.toml에 의존성으로 적지 않아도 패키지 이름으로 쓸 수 있습니다.
// 바이너리 안에서 `crate`는 바이너리 자신(이 파일)을 뜻하므로, 라이브러리의 항목은
// crate::가 아니라 restaurant::로 시작하는 경로로 가리킵니다.
// 바이너리는 바깥 사용자와 똑같이 라이브러리의 공개 API만 쓸 수 있습니다.

use restaurant::back_of_house::{Appetizer, Breakfast};
use restaurant::hosting;

fn main() {
    println!("main의 위치: {}", module_path!());

    println!("1. 재내보낸 경로 restaurant::hosting으로 부르기");
    hosting::add_to_waitlist("김민준");

    println!("2. 공개 구조체: 공개 필드는 바꾸고, 비공개 필드는 메서드로 읽기");
    let mut meal = Breakfast::summer("호밀빵");
    meal.toast = String::from("통밀빵");
    println!("  {} 토스트, 곁들임: {}", meal.toast, meal.fruit());

    println!("3. 공개 열거형: 배리언트를 모두 쓸 수 있음");
    let appetizers = [Appetizer::Soup, Appetizer::Salad];
    println!("  전채 메뉴: {appetizers:?}");

    println!("4. 라이브러리의 공개 함수 부르기");
    restaurant::eat_at_restaurant();

    // front_of_house는 라이브러리의 비공개 모듈이므로 바이너리에서는 쓸 수 없습니다.
    // 아래 줄의 주석을 풀면 E0603(module `front_of_house` is private) 에러가 납니다.
    // restaurant::front_of_house::serving::take_order("김민준", "호밀빵");
}
