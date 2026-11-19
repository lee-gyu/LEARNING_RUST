// 기대하는 에러: E0782
// 핵심 메시지: expected a type, found a trait
// 에디션: 2021, 2024에서 실패 / 2018에서는 경고(bare_trait_objects)와 함께 통과
//
// 2018 에디션까지는 dyn 없이 트레이트 이름만 써도 트레이트 객체로 받아 주고 경고만 냈습니다.
// 2021 에디션부터는 이 경고가 에러가 되었습니다. Box<Shape>를 Box<dyn Shape>로 고칩니다.
//
// 확인: rustc --edition 2024 compile_fail/e0782_bare_trait_object.rs --out-dir /tmp/rust-p2-ch3
//       rustc --edition 2018 compile_fail/e0782_bare_trait_object.rs --out-dir /tmp/rust-p2-ch3

trait Shape {
    fn area(&self) -> f64;
}

struct Square(f64);

impl Shape for Square {
    fn area(&self) -> f64 {
        self.0 * self.0
    }
}

fn total_area(shapes: &[Box<Shape>]) -> f64 { // → E0782
    shapes.iter().map(|s| s.area()).sum()
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Square(2.0)), Box::new(Square(3.0))];
    println!("{}", total_area(&shapes));
}
