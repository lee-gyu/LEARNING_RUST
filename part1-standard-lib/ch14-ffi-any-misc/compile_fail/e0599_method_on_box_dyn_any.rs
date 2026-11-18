// 기대하는 에러: E0599
// 핵심 메시지: the method `len` exists for struct `Box<(dyn Any + 'static)>`, but its trait bounds were not satisfied
//
// Box<dyn Any> 안에 String이 들어 있어도 컴파일러가 아는 것은 "어떤 'static 타입"뿐입니다.
// dyn Any로는 Any의 메서드(type_id)와 다운캐스트 메서드만 부를 수 있습니다.
// String의 메서드를 쓰려면 먼저 downcast_ref::<String>()으로 꺼냅니다.
//
// 확인: rustc --edition 2024 compile_fail/e0599_method_on_box_dyn_any.rs --out-dir /tmp/rust-p1-ch14

use std::any::Any;

fn main() {
    let value: Box<dyn Any> = Box::new(String::from("러스트"));
    println!("{}", value.len()); // → E0599
}
