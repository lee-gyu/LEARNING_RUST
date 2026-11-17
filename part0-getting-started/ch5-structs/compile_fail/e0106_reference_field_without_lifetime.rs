// 기대하는 에러: E0106 — missing lifetime specifier
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0106_reference_field_without_lifetime.rs
//
// 구조체 필드에 참조(&str)를 넣으면, 구조체가 살아 있는 동안 그 참조가 가리키는 값도 살아 있어야 합니다.
// 컴파일러가 이 관계를 확인하려면 라이프타임(lifetime) 표기가 필요합니다(ch10에서 배웁니다).
// 지금 단계의 해결책은 &str 대신 데이터를 소유하는 String을 쓰는 것입니다.

struct User {
    active: bool,
    username: &str,
    email: &str,
    sign_in_count: u64,
}

fn main() {
    let user1 = User {
        active: true,
        username: "ferris",
        email: "ferris@example.com",
        sign_in_count: 1,
    };

    println!(
        "{} <{}>, 활성: {}, 로그인 {}회",
        user1.username, user1.email, user1.active, user1.sign_in_count
    );
}
