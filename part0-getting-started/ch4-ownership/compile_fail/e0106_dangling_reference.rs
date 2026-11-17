// 기대하는 에러: E0106 — missing lifetime specifier
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0106_dangling_reference.rs
//
// s는 dangle 안에서 만든 값이므로 함수가 끝날 때 drop됩니다.
// 그 참조를 돌려주면 이미 해제된 메모리를 가리키는 댕글링 참조가 됩니다.
// 에러 메시지에 나오는 라이프타임(lifetime)은 ch10에서 배웁니다.

fn main() {
    let reference_to_nothing = dangle();
    println!("{reference_to_nothing}");
}

fn dangle() -> &String {
    let s = String::from("hello");

    &s
}
