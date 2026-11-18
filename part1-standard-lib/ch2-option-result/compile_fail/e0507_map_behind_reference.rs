// 기대하는 에러: error[E0507]: cannot move out of `user.nickname` which is behind a shared reference
// 핵심 메시지: `user.nickname` moved due to this method call
//             note: `Option::<T>::map` takes ownership of the receiver `self`, which moves `user.nickname`
//             help: consider calling `.as_ref()` to borrow the value's contents
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0507_map_behind_reference.rs --out-dir /tmp/rust-p1-ch2
//
// Option::map의 시그니처는 fn map<U, F>(self, f: F) -> Option<U>입니다. self를 받으므로 Option을 통째로 가져갑니다.
// 그런데 이 함수는 &User만 빌려 왔으므로 user.nickname의 소유권을 가져갈 수 없습니다. 빌린 것에서 값을 옮기면
// 원래 주인(호출한 쪽의 User)에 비어 있는 필드가 남기 때문입니다. unwrap(), unwrap_or(), and_then() 등 self를 받는
// 다른 콤비네이터도 똑같이 거부됩니다.
// 고치는 방법: as_ref()로 Option<&String>을 만들어 그것을 소비하게 합니다.
//   user.nickname.as_ref().map(|n| n.chars().count()).unwrap_or(0)
// &str이 필요하면 as_deref()로 Option<&str>을 만듭니다(examples/05_references.rs).

struct User {
    nickname: Option<String>,
}

fn nickname_len(user: &User) -> usize {
    user.nickname.map(|n| n.chars().count()).unwrap_or(0)
}

fn main() {
    let user = User {
        nickname: Some(String::from("하나둘")),
    };
    println!("{}", nickname_len(&user));
}
