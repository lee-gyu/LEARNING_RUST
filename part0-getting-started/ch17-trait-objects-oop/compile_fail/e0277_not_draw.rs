// 기대하는 에러: E0277 — the trait bound `String: Draw` is not satisfied
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_not_draw.rs
//
// Vec<Box<dyn Draw>>에는 Draw를 구현한 타입의 값만 넣을 수 있습니다.
// Box<String>을 Box<dyn Draw>로 바꾸려면 String이 Draw를 구현해야 하는데, 그런 구현이 없습니다.
// 동적 타입 언어라면 실행 중에 draw를 부르는 순간 실패했을 코드를, Rust는 컴파일할 때 거부합니다.
// 해결: Draw를 구현한 타입(예: Label)으로 감싸 넣거나, 정말 필요하다면 impl Draw for String을 작성합니다.

mod gui {
    pub trait Draw {
        fn draw(&self) -> String;
    }

    pub struct Screen {
        pub components: Vec<Box<dyn Draw>>,
    }

    impl Screen {
        pub fn run(&self) {
            for component in self.components.iter() {
                println!("{}", component.draw());
            }
        }
    }

    pub struct Button {
        pub label: String,
    }

    impl Draw for Button {
        fn draw(&self) -> String {
            format!("[ {} ]", self.label)
        }
    }
}

use gui::{Button, Screen};

fn main() {
    let screen = Screen {
        components: vec![
            Box::new(Button {
                label: String::from("OK"),
            }),
            Box::new(String::from("Hi")), // String은 Draw를 구현하지 않았습니다.
        ],
    };
    screen.run();
}
