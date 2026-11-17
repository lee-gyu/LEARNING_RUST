// `mod` 블록으로 모듈(module)을 만들면 크레이트 안에 모듈 트리(module tree)가 생깁니다.
// 이 파일 자체가 크레이트 루트(crate root)이고, 파일의 내용이 트리의 뿌리인 `crate` 모듈이 됩니다.
//
// 실행: cargo run -p p0-ch7-modules --example 01_module_tree
//
// 이 파일이 만드는 모듈 트리:
//
// crate                          ← 이 파일 (크레이트 루트)
//  ├── front_of_house            ← 손님을 맞는 홀
//  │   ├── hosting
//  │   │   ├── add_to_waitlist
//  │   │   └── seat_at_table
//  │   └── serving
//  │       ├── take_order
//  │       └── clean
//  ├── back_of_house             ← 주방
//  │   ├── cook_order
//  │   └── clean
//  └── main

// `pub`은 "바깥에서 써도 된다"는 표시입니다. 이 키워드가 없으면 main에서 부를 수 없는데,
// 그 이유는 03번 예제에서 다룹니다. 지금은 트리 모양에만 집중합니다.
mod front_of_house {
    // 모듈 안에 또 모듈을 둘 수 있습니다. hosting과 serving은 같은 부모를 둔 형제입니다.
    pub mod hosting {
        pub fn add_to_waitlist() {
            // module_path!()는 이 코드가 모듈 트리의 어디에 있는지를 문자열로 알려 주는 매크로입니다.
            println!("  대기 명단에 추가합니다    @ {}", module_path!());
        }

        pub fn seat_at_table() {
            println!("  자리로 안내합니다         @ {}", module_path!());
        }
    }

    pub mod serving {
        pub fn take_order() {
            println!("  주문을 받습니다           @ {}", module_path!());
        }

        // back_of_house에도 clean이라는 함수가 있지만 충돌하지 않습니다.
        // 모듈마다 이름 공간이 따로 있어서, 전체 경로가 다르면 다른 항목입니다.
        pub fn clean() {
            println!("  테이블을 치웁니다         @ {}", module_path!());
        }
    }
}

mod back_of_house {
    pub fn cook_order() {
        println!("  요리합니다                @ {}", module_path!());
    }

    pub fn clean() {
        println!("  주방을 청소합니다         @ {}", module_path!());
    }
}

fn main() {
    // main도 크레이트 루트에 정의된 함수이므로 트리의 맨 위(크레이트 이름)에 있습니다.
    // Cargo는 예제 파일 이름을 크레이트 이름으로 쓰므로 `01_module_tree`가 출력됩니다.
    println!("main의 위치: {}", module_path!()); // 출력: main의 위치: 01_module_tree

    // 트리를 따라 내려가는 경로를 `::`로 이어 써서 함수를 부릅니다(경로는 02번 예제에서 자세히 다룹니다).
    println!("손님이 왔습니다.");
    front_of_house::hosting::add_to_waitlist(); // 출력:   대기 명단에 추가합니다    @ 01_module_tree::front_of_house::hosting
    front_of_house::hosting::seat_at_table(); // 출력:   자리로 안내합니다         @ 01_module_tree::front_of_house::hosting
    front_of_house::serving::take_order(); // 출력:   주문을 받습니다           @ 01_module_tree::front_of_house::serving
    back_of_house::cook_order(); // 출력:   요리합니다                @ 01_module_tree::back_of_house

    println!("영업이 끝났습니다.");
    // 이름이 같은 두 clean 함수를 경로로 구분합니다.
    front_of_house::serving::clean(); // 출력:   테이블을 치웁니다         @ 01_module_tree::front_of_house::serving
    back_of_house::clean(); // 출력:   주방을 청소합니다         @ 01_module_tree::back_of_house
}
