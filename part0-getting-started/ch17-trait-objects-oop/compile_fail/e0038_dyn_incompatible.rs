// 기대하는 에러: E0038 — the trait `Shape` is not dyn compatible
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0038_dyn_incompatible.rs
//
// 트레이트 객체로 메서드를 부르려면 vtable(메서드 표)에 메서드마다 함수 하나를 적어 둘 수 있어야 합니다.
// scaled는 Self를 돌려주므로, dyn Shape 뒤의 실제 타입을 모르는 호출자는 돌려받을 값의 크기조차 알 수 없습니다.
// 이런 메서드가 있는 트레이트는 "dyn 호환(dyn compatible)"이 아니어서 dyn Shape를 만들 수 없습니다.
// 해결: scaled에 `where Self: Sized`를 붙여 트레이트 객체로는 부를 수 없는 메서드로 표시합니다
//       (examples/03_dynamic_dispatch.rs). 또는 scaled를 다른 트레이트로 옮기거나,
//       컴파일러의 제안처럼 열거형으로 도형을 표현합니다(examples/04_box_dyn_and_enum.rs).

trait Shape {
    fn area(&self) -> f64;
    fn scaled(&self, factor: f64) -> Self;
}

struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn scaled(&self, factor: f64) -> Self {
        Circle {
            radius: self.radius * factor,
        }
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn scaled(&self, factor: f64) -> Self {
        Rectangle {
            width: self.width * factor,
            height: self.height * factor,
        }
    }
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle {
            width: 2.0,
            height: 3.0,
        }),
    ];
    for shape in &shapes {
        println!("{:.2}", shape.area());
    }
}
