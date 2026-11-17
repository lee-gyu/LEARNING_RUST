// 라이브러리 크레이트의 루트입니다 (src/lib.rs).
// Cargo는 src/lib.rs가 있으면 패키지 이름과 같은 이름(restaurant)의 라이브러리 크레이트를 만듭니다.
//
// 이 크레이트의 모듈 트리와 파일:
//
// crate (restaurant)          src/lib.rs
//  ├── front_of_house         src/front_of_house.rs          (비공개)
//  │   ├── hosting            src/front_of_house/hosting.rs
//  │   └── serving            src/front_of_house/serving.rs
//  ├── back_of_house          src/back_of_house.rs           (공개)
//  ├── hosting                ← pub use로 재내보낸 이름
//  └── eat_at_restaurant

// `mod 이름;`은 "이 이름의 모듈이 여기 있고, 내용은 다른 파일에 있다"는 선언입니다.
// 중괄호 블록 대신 세미콜론으로 끝나면 컴파일러가 src/front_of_house.rs에서 내용을 찾습니다.
// 다른 언어의 include처럼 "파일을 붙여 넣는" 명령이 아니라, 모듈 트리에 모듈을 하나 등록하는 선언입니다.
// 그래서 한 파일은 모듈 트리 전체에서 딱 한 번만 mod로 선언합니다.
mod front_of_house;

// back_of_house는 pub mod라서 바깥(바이너리 크레이트 등)에서 restaurant::back_of_house로 쓸 수 있습니다.
pub mod back_of_house;

// front_of_house는 비공개지만, 그 안의 hosting만 restaurant::hosting이라는 경로로 공개합니다.
// 내용이 여러 파일로 나뉘어도 경로는 모듈 트리를 따르므로, 이 줄은 파일을 나누기 전과 똑같습니다.
pub use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    println!("  [{}] eat_at_restaurant 시작", module_path!());
    hosting::add_to_waitlist("최유나");
    hosting::seat_at_table("최유나");
    // serving은 재내보내지 않았으므로 라이브러리 안에서만 쓸 수 있습니다.
    let meal = front_of_house::serving::take_order("최유나", "호밀빵");
    front_of_house::serving::serve_order(&meal);
}
