// 기대 에러: E0277
//   the size for values of type `str` cannot be known at compilation time
//
// `&str`은 이미 주소와 길이를 실은 넓은 포인터입니다. `&dyn Display`로 바꾸려면 같은 포인터에
// vtable까지 실어야 하는데, 포인터 하나에는 추가 정보를 하나만 실을 수 있습니다.
// 그래서 트레이트 객체로 바꿀 수 있는 것은 크기가 정해진(`Sized`) 타입의 참조뿐입니다.
// 한 겹 더 감싼 `&&str`(얇은 포인터)을 넘기면 됩니다(예제 02의 4번 구역).
//
// 확인: rustc --edition 2024 compile_fail/e0277_str_to_dyn_display.rs --out-dir /tmp/er-ch8

use std::fmt::Display;

fn framed(item: &dyn Display) -> String {
    format!("[{item}]")
}

fn main() {
    let station: &str = "제주 관측소";
    println!("{}", framed(station)); // E0277
}
