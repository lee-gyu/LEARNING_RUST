// 정적 디스패치와 동적 디스패치, 그리고 트레이트 객체를 가리키는 두 포인터(&dyn, Box<dyn>)를 비교합니다.
// - 제네릭(T: Shape)은 컴파일할 때 타입마다 함수를 따로 만들어 부를 메서드를 정해 둡니다(정적 디스패치).
// - 트레이트 객체(dyn Shape)는 실행 중에 vtable(메서드 표)을 보고 부를 메서드를 찾습니다(동적 디스패치).
// - &dyn Shape와 Box<dyn Shape>는 둘 다 "값의 주소 + vtable의 주소" 두 칸짜리 포인터입니다.
// - 트레이트 객체로 부를 수 없는 메서드는 `where Self: Sized`로 따로 빼 두면,
//   트레이트 전체는 계속 dyn으로 쓸 수 있습니다(dyn 호환성).
//
// 실행: cargo run -p p0-ch17-trait-objects-oop --example 03_dynamic_dispatch

use std::f64::consts::PI;

trait Shape {
    fn name(&self) -> String;
    fn area(&self) -> f64;

    // Self를 돌려주는 메서드입니다. dyn Shape 뒤에 있는 값은 실제 타입을 모르므로
    // 돌려받을 값의 크기도 알 수 없어서, 이 메서드는 트레이트 객체로 부를 수 없습니다.
    // `where Self: Sized`는 "크기를 아는 구체 타입에서만 부를 수 있다"는 표시입니다.
    // 이 메서드는 vtable에서 빠지고, 덕분에 Shape를 dyn Shape로 쓸 수 있습니다.
    // where 절을 지우면 E0038입니다(compile_fail/e0038_dyn_incompatible.rs).
    fn scaled(&self, factor: f64) -> Self
    where
        Self: Sized;
}

struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn name(&self) -> String {
        format!("반지름 {}인 원", self.radius)
    }

    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }

    fn scaled(&self, factor: f64) -> Self {
        Circle {
            radius: self.radius * factor,
        }
    }
}

impl Shape for Rectangle {
    fn name(&self) -> String {
        format!("{}x{} 사각형", self.width, self.height)
    }

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

// 정적 디스패치: 컴파일러가 쓰인 타입마다 이 함수를 한 벌씩 만듭니다(ch10의 단형성화).
// describe_static::<Circle> 안의 shape.area()는 처음부터 Circle의 area를 부르도록 정해져 있습니다.
fn describe_static<T: Shape>(shape: &T) -> String {
    format!("{}: 넓이 {:.2}", shape.name(), shape.area())
}

// 동적 디스패치: 이 함수는 한 벌뿐입니다. shape.area()를 만나면 실행 중에
// shape에 붙어 있는 vtable에서 area 칸을 찾아, 거기 적힌 함수를 부릅니다.
fn describe_dynamic(shape: &dyn Shape) -> String {
    format!("{}: 넓이 {:.2}", shape.name(), shape.area())
}

// Box<dyn Shape>는 값을 소유하므로 함수 밖으로 돌려줄 수 있습니다.
// 지역 변수를 빌린 &dyn Shape는 함수가 끝나면 댕글링 참조가 되므로 이렇게 돌려줄 수 없습니다.
fn make_shapes() -> Vec<Box<dyn Shape>> {
    vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle {
            width: 2.0,
            height: 3.0,
        }),
    ]
}

fn main() {
    let circle = Circle { radius: 1.0 };
    let rect = Rectangle {
        width: 2.0,
        height: 3.0,
    };

    println!("1. 같은 본문, 다른 디스패치: 결과는 같습니다");
    println!("    정적: {}", describe_static(&circle)); // 출력: 정적: 반지름 1인 원: 넓이 3.14
    println!("    정적: {}", describe_static(&rect)); // 출력: 정적: 2x3 사각형: 넓이 6.00
    // &Circle, &Rectangle을 &dyn Shape 자리에 넘기면 컴파일러가 vtable을 붙여 트레이트 객체로 바꿉니다.
    println!("    동적: {}", describe_dynamic(&circle)); // 출력: 동적: 반지름 1인 원: 넓이 3.14
    println!("    동적: {}", describe_dynamic(&rect)); // 출력: 동적: 2x3 사각형: 넓이 6.00

    println!("2. 트레이트 객체 포인터는 두 칸입니다 (64비트 환경)");
    println!("    &Circle:         {}바이트", size_of::<&Circle>()); // 출력: &Circle:         8바이트
    println!("    &dyn Shape:      {}바이트", size_of::<&dyn Shape>()); // 출력: &dyn Shape:      16바이트
    println!("    Box<Circle>:     {}바이트", size_of::<Box<Circle>>()); // 출력: Box<Circle>:     8바이트
    println!("    Box<dyn Shape>:  {}바이트", size_of::<Box<dyn Shape>>()); // 출력: Box<dyn Shape>:  16바이트
    // 늘어난 8바이트가 vtable을 가리키는 주소입니다.

    println!("3. 포인터의 크기는 같아도, 가리키는 값의 크기는 타입마다 다릅니다");
    let shapes = make_shapes();
    for shape in &shapes {
        // shape는 &Box<dyn Shape>이고, shape.as_ref()는 Box 안의 값을 빌린 &dyn Shape입니다.
        // size_of_val은 값의 실제 크기를 알려 줍니다. dyn Shape의 크기는 vtable에 적혀 있습니다.
        println!(
            "    {}: 포인터 {}바이트, 값 {}바이트",
            shape.name(),
            size_of_val(shape),
            size_of_val(shape.as_ref())
        );
    }
    // 출력:
    //     반지름 1인 원: 포인터 16바이트, 값 8바이트
    //     2x3 사각형: 포인터 16바이트, 값 16바이트
    // dyn Shape 자체는 크기가 정해지지 않은 타입이라 Vec<dyn Shape>는 만들 수 없습니다(E0277).
    // 그래서 트레이트 객체는 언제나 &나 Box 같은 포인터 뒤에 둡니다.

    println!("4. &dyn은 빌려서, Box<dyn>은 소유해서 가리킵니다");
    // 힙 할당 없이 스택의 값을 빌려 트레이트 객체 배열을 만들 수 있습니다.
    // 이 배열은 circle과 rect가 살아 있는 동안만 쓸 수 있습니다.
    let borrowed: [&dyn Shape; 2] = [&circle, &rect];
    let total: f64 = borrowed.iter().map(|shape| shape.area()).sum();
    println!("    빌린 도형 넓이 합계: {total:.2}"); // 출력: 빌린 도형 넓이 합계: 9.14
    // shapes는 make_shapes가 만든 값을 소유합니다. 원래 만든 함수는 이미 끝났습니다.
    let owned_total: f64 = shapes.iter().map(|shape| shape.area()).sum();
    println!("    소유한 도형 넓이 합계: {owned_total:.2}"); // 출력: 소유한 도형 넓이 합계: 9.14
    // Box 안의 트레이트 객체를 &dyn Shape로 빌려줄 때는 as_ref()를 씁니다.
    // &shapes[1]을 그대로 넘기면 컴파일러가 Box<dyn Shape> 자체를 트레이트 객체로 바꾸려다
    // E0277(the trait `Shape` is not implemented for `Box<dyn Shape>`)을 냅니다.
    println!("    {}", describe_dynamic(shapes[1].as_ref())); // 출력: 2x3 사각형: 넓이 6.00

    println!("5. `where Self: Sized` 메서드는 구체 타입에서만 부를 수 있습니다");
    let bigger = circle.scaled(2.0); // circle의 타입은 Circle이므로 문제없습니다.
    println!("    {}", describe_static(&bigger)); // 출력: 반지름 2인 원: 넓이 12.57
    // shapes[0].scaled(2.0)은 거부됩니다: the `scaled` method cannot be invoked on a trait object
}
