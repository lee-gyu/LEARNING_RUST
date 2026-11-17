// 트레이트 객체(`dyn Trait`)로 서로 다른 타입을 한 컬렉션에 담는 GUI 예제입니다.
// - Vec<Box<dyn Draw>>에는 Draw를 구현한 타입이면 무엇이든 섞어 담을 수 있습니다.
// - 라이브러리(gui 모듈) 밖에서 만든 타입도 Draw만 구현하면 Screen에 들어갑니다.
// - 트레이트의 기본 구현은 상속에서 부모 클래스의 메서드를 물려받는 것처럼 코드를 재사용하게 해 줍니다.
// - 제네릭 버전 GenericScreen<T: Draw>는 한 가지 타입만 담을 수 있습니다.
//
// 실행: cargo run -p p0-ch17-trait-objects-oop --example 02_gui_trait_objects

// 라이브러리 역할을 하는 모듈입니다. 실제로는 별도의 크레이트로 배포된다고 생각하세요.
mod gui {
    pub trait Draw {
        // 구현하는 타입이 반드시 작성하는 메서드입니다.
        // 실제로 화면에 그리는 대신, 그린 모양을 글자로 돌려줍니다.
        fn draw(&self) -> String;

        // 기본 구현: 대부분의 컴포넌트는 키보드 포커스를 받지 않습니다.
        // 구현하는 타입은 아무것도 쓰지 않아도 이 메서드를 얻고, 필요하면 재정의합니다.
        fn focusable(&self) -> bool {
            false
        }
    }

    pub struct Screen {
        // Box<dyn Draw>는 "Draw를 구현한 어떤 타입의 값"을 힙에 담은 Box입니다.
        // 원소마다 실제 타입이 달라도 Vec 입장에서는 모두 같은 타입(Box<dyn Draw>)입니다.
        pub components: Vec<Box<dyn Draw>>,
    }

    impl Screen {
        pub fn run(&self) {
            // run은 각 컴포넌트의 실제 타입을 모릅니다. 아는 것은 Draw의 메서드뿐입니다.
            // 어느 타입의 draw를 부를지는 실행 중에 정해집니다(동적 디스패치, 03번 예제).
            for component in self.components.iter() {
                let mark = if component.focusable() { "*" } else { " " };
                println!("    {mark} {}", component.draw());
            }
        }
    }

    pub struct Button {
        pub width: u32,
        pub height: u32,
        pub label: String,
    }

    impl Draw for Button {
        fn draw(&self) -> String {
            format!("[ {} ] ({}x{})", self.label, self.width, self.height)
        }

        // 버튼은 포커스를 받으므로 기본 구현을 재정의합니다.
        fn focusable(&self) -> bool {
            true
        }
    }

    pub struct Label {
        pub text: String,
    }

    impl Draw for Label {
        fn draw(&self) -> String {
            self.text.clone()
        }
        // focusable을 쓰지 않았으므로 기본 구현(false)을 그대로 씁니다.
    }

    // 비교용: 트레이트 바운드를 단 제네릭 버전입니다.
    // T는 Screen 하나에 대해 한 가지 구체 타입으로 정해집니다.
    pub struct GenericScreen<T: Draw> {
        pub components: Vec<T>,
    }

    impl<T: Draw> GenericScreen<T> {
        pub fn run(&self) {
            for component in self.components.iter() {
                println!("    {}", component.draw());
            }
        }
    }
}

// 여기부터는 gui 라이브러리를 가져다 쓰는 사용자 코드입니다.
use gui::{Button, Draw, GenericScreen, Label, Screen};

// 라이브러리를 만든 사람은 알지 못하는, 사용자가 새로 만든 컴포넌트입니다.
// gui 모듈은 한 글자도 고치지 않았는데 Screen에 넣을 수 있습니다.
struct SelectBox {
    width: u32,
    height: u32,
    options: Vec<String>,
}

impl Draw for SelectBox {
    fn draw(&self) -> String {
        format!(
            "<{}> ({}x{})",
            self.options.join(" | "),
            self.width,
            self.height
        )
    }

    fn focusable(&self) -> bool {
        true
    }
}

fn main() {
    println!("1. 서로 다른 세 타입을 Vec<Box<dyn Draw>> 하나에 담아 그립니다 (*는 포커스 가능)");
    let screen = Screen {
        components: vec![
            Box::new(Label {
                text: String::from("배송 방법을 고르세요"),
            }),
            Box::new(SelectBox {
                width: 75,
                height: 10,
                options: vec![
                    String::from("택배"),
                    String::from("퀵"),
                    String::from("방문 수령"),
                ],
            }),
            Box::new(Button {
                width: 50,
                height: 10,
                label: String::from("OK"),
            }),
        ],
    };
    screen.run();
    // 출력:
    //       배송 방법을 고르세요
    //     * <택배 | 퀵 | 방문 수령> (75x10)
    //     * [ OK ] (50x10)
    // vec![..., Box::new(String::from("Hi"))]처럼 Draw를 구현하지 않은 타입을 넣으면
    // E0277로 거부됩니다(compile_fail/e0277_not_draw.rs).

    println!("2. 트레이트 객체로는 트레이트의 메서드만 부를 수 있습니다");
    let focusable = screen
        .components
        .iter()
        .filter(|component| component.focusable())
        .count();
    println!(
        "    컴포넌트 {}개 중 포커스를 받는 것 {}개",
        screen.components.len(),
        focusable
    ); // 출력: 컴포넌트 3개 중 포커스를 받는 것 2개
    // screen.components[0].text처럼 실제 타입의 필드를 쓰면 E0609입니다.
    // 첫 원소가 Label이라는 사실은 Box<dyn Draw>라는 타입에 남아 있지 않기 때문입니다.

    println!("3. 어떤 타입을 담을지 실행 중에 데이터를 보고 정할 수 있습니다");
    let layout = "label:로그인이 필요합니다,button:로그인,button:취소";
    let mut login = Screen {
        components: Vec::new(),
    };
    for item in layout.split(',') {
        let Some((kind, text)) = item.split_once(':') else {
            continue;
        };
        // 타입 표기 `Box<dyn Draw>`가 있어야 두 갈래의 Box<Button>, Box<Label>이 모두
        // Box<dyn Draw>로 바뀝니다. 표기를 지우면 첫 갈래가 타입을 Box<Button>으로 정해 버려서
        // E0308(`match` arms have incompatible types)이 납니다.
        let component: Box<dyn Draw> = match kind {
            "button" => Box::new(Button {
                width: 40,
                height: 10,
                label: text.to_string(),
            }),
            _ => Box::new(Label {
                text: text.to_string(),
            }),
        };
        login.components.push(component);
    }
    login.run();
    // 출력:
    //       로그인이 필요합니다
    //     * [ 로그인 ] (40x10)
    //     * [ 취소 ] (40x10)

    println!("4. 제네릭 버전 GenericScreen<T>는 한 가지 타입만 담습니다");
    let buttons = GenericScreen {
        components: vec![
            Button {
                width: 30,
                height: 10,
                label: String::from("예"),
            },
            Button {
                width: 30,
                height: 10,
                label: String::from("아니요"),
            },
        ],
    };
    buttons.run();
    // 출력:
    //     [ 예 ] (30x10)
    //     [ 아니요 ] (30x10)
    // 여기에 SelectBox를 섞으면 E0308(expected `Button`, found `SelectBox`)입니다.
    // T가 Button으로 정해진 GenericScreen<Button>에는 Button만 들어갈 수 있습니다.
}
