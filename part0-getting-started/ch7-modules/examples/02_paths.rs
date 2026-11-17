// 경로(path)는 모듈 트리에서 항목을 찾아가는 주소입니다.
// `crate`로 시작하는 절대 경로와, 현재 모듈에서 출발하는 상대 경로(`self`, `super`, 이름)가 있습니다.
//
// 실행: cargo run -p p0-ch7-modules --example 02_paths
//
// 이 파일의 모듈 트리:
//
// crate
//  ├── deliver_order
//  ├── front_of_house
//  │   └── hosting
//  │       ├── add_to_waitlist
//  │       └── seat_at_table
//  ├── back_of_house
//  │   ├── cook_order
//  │   ├── fix_incorrect_order
//  │   └── dishwashing
//  │       └── wash_dishes
//  └── main

// 크레이트 루트에 있는 함수입니다. 절대 경로로 쓰면 `crate::deliver_order`입니다.
fn deliver_order() {
    println!("    -> 음식을 손님에게 가져갑니다 @ {}", module_path!());
}

mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {
            println!("    -> 대기 명단에 추가합니다     @ {}", module_path!());
        }

        pub fn seat_at_table() {
            // 같은 모듈(hosting)에 있는 함수는 이름만 써도 됩니다.
            // `self::`를 붙이면 "현재 모듈에서 출발한다"는 뜻을 드러낼 수 있습니다. 둘은 같은 함수입니다.
            add_to_waitlist();
            self::add_to_waitlist();
        }
    }
}

mod back_of_house {
    // `pub`이 없는 함수는 이 모듈 바깥에서는 부를 수 없습니다(03번 예제).
    // 같은 모듈 안에서 부르는 것은 상관없습니다.
    fn cook_order() {
        println!("    -> 다시 요리합니다            @ {}", module_path!());
    }

    pub fn fix_incorrect_order() {
        cook_order();
        // `super`는 부모 모듈을 가리킵니다. 파일 시스템의 `..`와 같습니다.
        // back_of_house의 부모는 크레이트 루트이므로 super::deliver_order는 crate::deliver_order와 같은 함수입니다.
        super::deliver_order();
    }

    pub mod dishwashing {
        pub fn wash_dishes() {
            println!("    -> 설거지합니다               @ {}", module_path!());
            // super를 두 번 쓰면 두 단계 위로 올라갑니다: dishwashing → back_of_house → crate
            super::super::deliver_order();
            // 같은 함수를 절대 경로로 부를 수도 있습니다.
            crate::deliver_order();
        }
    }
}

fn main() {
    println!("1. 절대 경로: crate::front_of_house::hosting::add_to_waitlist()");
    // `crate`는 이 크레이트의 루트 모듈입니다. 파일 시스템의 `/`처럼 어디에서 쓰든 같은 곳을 가리킵니다.
    crate::front_of_house::hosting::add_to_waitlist(); // 출력:     -> 대기 명단에 추가합니다     @ 02_paths::front_of_house::hosting

    println!("2. 상대 경로: front_of_house::hosting::add_to_waitlist()");
    // main은 크레이트 루트에 있으므로, 같은 곳에 정의된 front_of_house에서 바로 출발할 수 있습니다.
    front_of_house::hosting::add_to_waitlist(); // 출력:     -> 대기 명단에 추가합니다     @ 02_paths::front_of_house::hosting

    println!("3. 같은 모듈 안에서 이름만, 또는 self::로 부르기");
    front_of_house::hosting::seat_at_table();
    // 출력:     -> 대기 명단에 추가합니다     @ 02_paths::front_of_house::hosting
    // 출력:     -> 대기 명단에 추가합니다     @ 02_paths::front_of_house::hosting

    println!("4. super::로 부모 모듈의 함수 부르기");
    back_of_house::fix_incorrect_order();
    // 출력:     -> 다시 요리합니다            @ 02_paths::back_of_house
    // 출력:     -> 음식을 손님에게 가져갑니다 @ 02_paths

    println!("5. super::super::와 crate::는 같은 곳을 가리킵니다");
    back_of_house::dishwashing::wash_dishes();
    // 출력:     -> 설거지합니다               @ 02_paths::back_of_house::dishwashing
    // 출력:     -> 음식을 손님에게 가져갑니다 @ 02_paths
    // 출력:     -> 음식을 손님에게 가져갑니다 @ 02_paths

    // 코드 안에서는 자기 크레이트를 `crate`라고 부르지만, module_path!()는 크레이트의 실제 이름(02_paths)을 보여 줍니다.
}
