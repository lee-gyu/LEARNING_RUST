// 기대하는 에러: E0594 — cannot assign to `user.email`, as `user` is not declared as mutable
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0594_assign_field_of_immutable.rs
//
// 필드 하나만 바꾸더라도 인스턴스를 담은 변수가 `mut`여야 합니다.
// Rust에는 "이 필드만 가변"이라는 표시가 없고, 가변성은 인스턴스(를 담은 변수) 전체에 붙습니다.

struct User {
    username: String,
    email: String,
}

fn main() {
    let user = User {
        username: String::from("ferris"),
        email: String::from("old@example.com"),
    };

    user.email = String::from("new@example.com");

    println!("{} <{}>", user.username, user.email);
}
