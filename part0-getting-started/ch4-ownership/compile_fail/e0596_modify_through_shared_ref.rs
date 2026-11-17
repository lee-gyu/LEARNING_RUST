// 기대하는 에러: E0596 — cannot borrow `*some_string` as mutable, as it is behind a `&` reference
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0596_modify_through_shared_ref.rs
//
// change는 불변 참조(&String)를 받았으므로 빌려 온 값을 읽을 수만 있습니다.
// push_str은 String을 바꾸는 메서드라서 가변 참조가 필요합니다.

fn main() {
    let s = String::from("hello");

    change(&s);
    println!("{s}");
}

fn change(some_string: &String) {
    some_string.push_str(", world");
}
