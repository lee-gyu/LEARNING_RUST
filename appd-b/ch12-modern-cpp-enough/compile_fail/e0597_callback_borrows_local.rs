// 기대하는 에러: E0597
// 핵심 메시지: `counter` does not live long enough
//              coercion requires that `counter` is borrowed for `'static`
//              note: due to object lifetime defaults, `Box<dyn FnMut()>` actually means `Box<(dyn FnMut() + 'static)>`
//
// compare/03_lambda_captures_this.cpp를 같은 구조로 옮긴 코드입니다. counter.handler()가 돌려주는 클로저는
// counter를 빌립니다(&mut self). Button::on_click의 매개변수와 Button의 필드는 Box<dyn FnMut()>인데, 구조체
// 필드나 함수 시그니처에 쓴 Box<dyn FnMut()>는 Box<dyn FnMut() + 'static>과 같습니다(Reference "Default trait
// object lifetimes". 함수 본문의 let 타입 표기에서는 라이프타임이 추론되어 이 요구가 생기지 않음).
// 'static이라고 어디에도 적지 않았는데 등록하는 순간 "counter를 'static 동안 빌려야 한다"는 요구가 생기고,
// 블록 끝에서 drop되는 counter는 그 요구를 채울 수 없습니다. C++의 std::function은 이 정보를 타입에 남기지 않아
// 같은 코드가 컴파일되고, 실행하면 해제 후 사용이 됩니다.
// 고치는 방법: 콜백이 상태를 소유하게 합니다(Rc를 옮기거나 Weak를 옮김). examples/01_callbacks_own_state.rs.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch12-modern-cpp-enough compile_fail/e0597_callback_borrows_local.rs

#[derive(Default)]
struct Counter {
    clicks: u32,
}

impl Counter {
    fn handler(&mut self) -> impl FnMut() + '_ {
        || {
            self.clicks += 1;
            println!("clicks: {}", self.clicks);
        }
    }
}

#[derive(Default)]
struct Button {
    handlers: Vec<Box<dyn FnMut()>>,
}

impl Button {
    fn on_click(&mut self, f: Box<dyn FnMut()>) {
        self.handlers.push(f);
    }

    fn click(&mut self) {
        for f in &mut self.handlers {
            f();
        }
    }
}

fn main() {
    let mut button = Button::default();
    {
        let mut counter = Counter::default();
        button.on_click(Box::new(counter.handler())); // E0597
    } // counter가 여기서 drop됩니다.
    button.click();
}
