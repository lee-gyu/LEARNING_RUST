// `pub use`(재내보내기, re-export)는 다른 곳에 정의된 항목을 "이 모듈의 항목인 것처럼" 바깥에 공개합니다.
// 내부 구조는 만드는 사람에게 편한 대로 두고, 바깥에는 쓰는 사람에게 편한 경로를 보여 줄 수 있습니다.
//
// 실행: cargo run -p p0-ch7-modules --example 07_pub_use
//
// 실제 모듈 트리:                         바깥(main)에서 보이는 경로:
//
// crate                                   restaurant::hosting::add_to_waitlist
//  ├── restaurant                         restaurant::order_food
//  │   ├── front_of_house      (비공개)   restaurant::eat_at_restaurant
//  │   │   └── hosting
//  │   │       └── add_to_waitlist
//  │   ├── back_of_house       (비공개)
//  │   │   └── cook_order
//  │   └── eat_at_restaurant
//  └── main

mod restaurant {
    // 식당을 운영하는 사람은 "홀(front of house)"과 "주방(back of house)"으로 나눠 생각합니다.
    // 이 구분은 내부 사정이므로 두 모듈 모두 비공개로 둡니다.
    mod front_of_house {
        pub mod hosting {
            pub fn add_to_waitlist() {
                println!("  대기 명단에 추가합니다 @ {}", module_path!());
            }
        }
    }

    mod back_of_house {
        pub fn cook_order() {
            println!("  요리합니다             @ {}", module_path!());
        }
    }

    // 손님은 홀/주방 구분을 모릅니다. 그래서 hosting을 restaurant 바로 아래에 있는 것처럼 공개합니다.
    // 이제 바깥에서는 restaurant::hosting으로 부를 수 있습니다.
    pub use self::front_of_house::hosting;

    // `as`와 함께 쓰면 이름까지 바꿔서 공개할 수 있습니다.
    pub use self::back_of_house::cook_order as order_food;

    pub fn eat_at_restaurant() {
        // pub use도 use이므로, 이 모듈 안에서는 짧은 이름 hosting을 그대로 쓸 수 있습니다.
        hosting::add_to_waitlist();
        order_food();
    }
}

fn main() {
    println!("1. 재내보낸 경로 restaurant::hosting으로 부르기");
    restaurant::hosting::add_to_waitlist();
    // 출력:   대기 명단에 추가합니다 @ 07_pub_use::restaurant::front_of_house::hosting
    // module_path!()의 결과를 보면, 함수가 정의된 위치는 여전히 front_of_house::hosting입니다.
    // pub use는 항목을 옮기거나 복사하지 않고, 같은 항목을 가리키는 새 경로를 하나 더 공개할 뿐입니다.

    println!("2. 이름을 바꿔 재내보낸 restaurant::order_food 부르기");
    restaurant::order_food(); // 출력:   요리합니다             @ 07_pub_use::restaurant::back_of_house

    println!("3. 모듈 안에서도 pub use로 만든 이름을 씁니다");
    restaurant::eat_at_restaurant();
    // 출력:   대기 명단에 추가합니다 @ 07_pub_use::restaurant::front_of_house::hosting
    // 출력:   요리합니다             @ 07_pub_use::restaurant::back_of_house

    // 원래 경로는 쓸 수 없습니다. front_of_house가 restaurant의 비공개 모듈이기 때문입니다.
    // 아래 줄의 주석을 풀면 E0603(module `front_of_house` is private) 에러가 납니다.
    // restaurant::front_of_house::hosting::add_to_waitlist();
}
