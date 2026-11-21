// 기대하는 에러: E0507
// 핵심 메시지: cannot move out of `account.owner` which is behind a shared reference
//
// Java라면 `String name = account.owner;`는 같은 문자열 객체를 가리키는 참조를 하나 더 만들 뿐입니다.
// Rust에서 `let name = account.owner;`는 `String`의 소유권을 옮기는(move) 코드인데, `account`는
// 빌린 것(`&Account`)이라 그 안의 필드를 가져갈 수 없습니다. 가져가면 빌려 준 쪽의 `Account`에
// 이름이 빠진 구멍이 남기 때문입니다.
//
// 고치는 법: 읽기만 한다면 빌립니다(`let name = &account.owner;`). 따로 가지고 있어야 한다면
// 복제합니다(`let name = account.owner.clone();`).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch4-memory-without-gc compile_fail/e0507_move_field_out_of_reference.rs

struct Account {
    owner: String,
    balance: i64,
}

fn describe(account: &Account) -> String {
    let name = account.owner; // E0507
    format!("{name}: {}원", account.balance)
}

fn main() {
    let account = Account {
        owner: String::from("하준"),
        balance: 100,
    };
    println!("{}", describe(&account));
}
