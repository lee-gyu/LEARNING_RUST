// 기대하는 에러: error[E0283]: type annotations needed
// 핵심 메시지: = note: the type must implement `From<&str>`
//             = note: required for `&str` to implement `Into<_>`
//
// "민수"를 무엇으로 바꿀지 알 수 없습니다. From<&str>를 구현한 타입은
// String, Box<str>, Box<dyn Error>, Vec<u8>, Rc<str> 등 여럿이고, println!의 {}는 그중 하나를 고를 단서가 되지 못합니다.
// 해결: let name: String = "민수".into(); 처럼 목적 타입을 적거나, String::from("민수")처럼 쓰는 쪽을 바꿉니다.
//
// 확인: rustc --edition 2024 compile_fail/e0283_into_unknown_target.rs --out-dir <임시 디렉터리>

fn main() {
    let name = "민수".into();
    println!("{name}");
}
