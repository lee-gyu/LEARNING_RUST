// 기대하는 에러: E0596 — cannot borrow `account` as mutable, as it is not declared as mutable
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0596_mut_method_on_immutable.rs
//
// `account.deposit(500)`은 `Account::deposit(&mut account, 500)`과 같습니다.
// 점(.) 표기가 `&mut`를 자동으로 붙여 줘도, 가변으로 빌리려면 변수 자체가 `mut`여야 한다는 규칙은 그대로입니다.

struct Account {
    balance: u64,
}

impl Account {
    fn deposit(&mut self, amount: u64) {
        self.balance += amount;
    }
}

fn main() {
    let account = Account { balance: 1000 };

    account.deposit(500);

    println!("잔액: {}", account.balance);
}
