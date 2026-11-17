// 기대하는 에러: E0277 — `Point` doesn't implement `std::fmt::Display`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_missing_supertrait.rs
//
// `trait OutlinePrint: fmt::Display`는 "OutlinePrint를 구현하려면 Display부터 구현해야 한다"는 뜻입니다.
// OutlinePrint의 기본 구현이 to_string()(Display가 있어야 생기는 메서드)을 쓰기 때문입니다.
// Point는 Display를 구현하지 않았으므로 OutlinePrint를 구현할 수 없습니다.
// 해결: `impl fmt::Display for Point { ... }`를 먼저 작성합니다.

use std::fmt;

trait OutlinePrint: fmt::Display {
    fn outline_print(&self) {
        let output = self.to_string();
        let len = output.len();
        println!("{}", "*".repeat(len + 4));
        println!("* {output} *");
        println!("{}", "*".repeat(len + 4));
    }
}

struct Point {
    x: i32,
    y: i32,
}

impl OutlinePrint for Point {}

fn main() {
    let p = Point { x: 1, y: 3 };
    println!("{} {}", p.x, p.y);
    p.outline_print();
}
