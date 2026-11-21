// Q4.2 · Q4.4 · Q4.7 — "두 변수가 같은 객체를 가리키는" 코드를 Rust로 옮기는 네 가지 방법
//
// Java나 Python에서 `b = a`는 같은 객체를 가리키는 이름을 하나 더 만듭니다. b로 고치면 a로도 보입니다.
// Rust에는 이 한 가지 동작이 없고, 하려는 일에 따라 네 가지 중에서 고릅니다.
//
//   1. 이동: 소유자를 바꾼다. 이전 이름은 더 쓸 수 없다.
//   2. 빌려 주기: 소유자는 그대로 두고 잠깐 고치게 한다.
//   3. 복제: 독립된 사본을 만든다. 고쳐도 서로 보이지 않는다.
//   4. Rc<RefCell<T>>: 여러 곳이 함께 소유하고 함께 고친다. Java의 참조에 가장 가깝다.
//
// 실행: cargo run -p faq-ch4-memory-without-gc --example 01_java_style_sharing

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct Account {
    owner: String,
    balance: i64,
}

fn open(owner: &str) -> Account {
    Account {
        owner: owner.to_string(),
        balance: 100,
    }
}

/// 소유권을 가져가지 않고 잠깐 빌려서 고칩니다.
fn deposit(account: &mut Account, amount: i64) {
    account.balance += amount;
}

fn main() {
    println!("1. 이동: let b = a;");
    let a = open("하준");
    let b = a; // 소유자가 b로 바뀝니다. 이 뒤에 a를 쓰면 E0382(Part 0 ch4 4.1절)
    println!("  b: {} {}원", b.owner, b.balance);
    // 출력:   b: 하준 100원

    println!("2. 빌려 주기: deposit(&mut account, 50)");
    let mut account = open("서아");
    deposit(&mut account, 50); // 함수가 끝나면 빌린 것이 돌아옵니다.
    println!("  account: {} {}원", account.owner, account.balance);
    // 출력:   account: 서아 150원

    println!("3. 복제: let mut copy = original.clone();");
    let original = open("도윤");
    let mut copy = original.clone(); // 힙의 이름 문자열까지 새로 만듭니다.
    copy.balance += 50;
    println!("  original {}원, copy {}원", original.balance, copy.balance);
    // 출력:   original 100원, copy 150원

    println!("4. Rc<RefCell<Account>>: 함께 소유하고 함께 고치기");
    let shared = Rc::new(RefCell::new(open("지우")));
    let other = Rc::clone(&shared); // 계좌를 복제하지 않고 참조 수만 늘립니다.
    other.borrow_mut().balance += 50; // other로 고치면
    println!(
        "  shared로 읽은 잔액 {}원, 소유자 수 {}",
        shared.borrow().balance, // shared로도 보입니다.
        Rc::strong_count(&shared)
    );
    // 출력:   shared로 읽은 잔액 150원, 소유자 수 2
    // Java의 `Account other = shared;`와 같은 동작이지만, 비용(참조 수 세기, 실행 중 대여 검사)이
    // 타입에 드러납니다. 필요한 곳에서만 씁니다.
}
