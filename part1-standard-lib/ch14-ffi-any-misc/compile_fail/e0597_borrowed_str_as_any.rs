// 기대하는 에러: E0597
// 핵심 메시지: `line` does not live long enough ... coercion requires that `line` is borrowed for `'static`
//
// dyn Any에는 'static인 타입만 담을 수 있습니다. name의 타입은 지역 변수 line을 빌린 &'a str이라서
// 'static이 아니고, &name을 &dyn Any로 바꾸려면 line을 'static 동안 빌려야 하는데
// line은 main이 끝나면 사라지므로 불가능합니다.
//
// 확인: rustc --edition 2024 compile_fail/e0597_borrowed_str_as_any.rs --out-dir /tmp/rust-p1-ch14

use std::any::Any;

fn describe(value: &dyn Any) -> &'static str {
    if value.is::<&str>() { "문자열 조각" } else { "모르는 타입" }
}

fn main() {
    let line = String::from("이름=러스트");
    let name: &str = &line[7..]; // line을 빌린 조각
    println!("{}", describe(&name)); // → E0597
}
