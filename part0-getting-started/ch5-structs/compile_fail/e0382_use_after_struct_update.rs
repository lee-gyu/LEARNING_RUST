// 기대하는 에러: E0382 — borrow of partially moved value: `user1`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0382_use_after_struct_update.rs
//
// 구조체 갱신 문법 `..user1`은 나머지 필드를 user1에서 "대입"으로 가져옵니다.
// String인 username은 Copy가 아니므로 user2로 이동하고, user1은 일부 필드가 빠진 부분 이동 상태가 됩니다.
// 그 뒤에 user1 전체를 쓰려 하면(여기서는 디버그 형식으로 출력) 빠져나간 필드 때문에 거부됩니다.

#[derive(Debug)]
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

fn main() {
    let user1 = User {
        email: String::from("ferris@example.com"),
        username: String::from("ferris"),
        active: true,
        sign_in_count: 1,
    };

    let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };

    println!("{user2:?}");
    println!("{user1:?}");
}
