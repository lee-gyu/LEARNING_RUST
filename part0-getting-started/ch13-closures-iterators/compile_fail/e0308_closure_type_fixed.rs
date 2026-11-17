// 기대하는 에러: error[E0308]: mismatched types
// 핵심 메시지: expected `String`, found integer
//             note: expected because the closure was earlier called with an argument of type `String`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0308_closure_type_fixed.rs --out-dir /tmp/rust-ch13
//
// 타입 표기가 없는 클로저는 처음 호출된 모습을 보고 매개변수와 반환값의 타입을 하나로 정합니다.
// 첫 호출에서 String을 넘겼으므로 x의 타입은 String으로 고정되고, 다음 호출의 정수는 맞지 않습니다.
// 클로저는 여러 타입을 받는 제네릭 함수가 아닙니다.
// 고치는 방법: 타입마다 클로저를 따로 만들거나, 정말로 여러 타입에 쓰려면 제네릭 함수(ch10)로 만듭니다.

fn main() {
    let identity = |x| x;

    let s = identity(String::from("hello"));
    let n = identity(5);

    println!("{s} {n}");
}
