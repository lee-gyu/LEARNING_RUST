// 기대하는 에러: E0308 — mismatched types
// 핵심 메시지: expected `&[Box<dyn Widget>]`, found `&Vec<Box<Button>>`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0308_no_coercion_inside_vec.rs --out-dir /tmp/er-ch5
//
// `Box<Button>` 하나는 `Box<dyn Widget>` 자리에서 강제 변환됩니다. `&Vec<T>`도 `&[T]` 자리에서
// 강제 변환됩니다. 그런데 두 변환을 겹쳐 `&Vec<Box<Button>>`을 `&[Box<dyn Widget>]`로 바꾸는 일은
// 일어나지 않습니다. 강제 변환은 바깥 한 겹의 모양만 바꾸고 컨테이너 안쪽 원소의 타입은 바꾸지
// 않습니다. 원소마다 바꾸려면 새 컨테이너를 만들어야 하는데, 그것은 컴파일러가 몰래 해 줄 수 있는
// 일이 아닙니다.
// 해결: 처음부터 `Vec<Box<dyn Widget>>`로 만들거나, 원소를 하나씩 바꿔 새 `Vec`을 만듭니다
//       (examples/05_coercion_sites.rs의 3번 구역).

trait Widget {
    fn draw(&self) -> String;
}

struct Button(&'static str);

impl Widget for Button {
    fn draw(&self) -> String {
        format!("[{}]", self.0)
    }
}

fn draw_all(widgets: &[Box<dyn Widget>]) -> String {
    widgets.iter().map(|w| w.draw()).collect::<Vec<_>>().join(" ")
}

fn main() {
    let buttons: Vec<Box<Button>> = vec![Box::new(Button("저장")), Box::new(Button("취소"))];
    println!("{}", draw_all(&buttons)); // E0308
}
