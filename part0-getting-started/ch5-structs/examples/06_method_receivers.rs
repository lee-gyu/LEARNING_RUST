// 이 예제가 보여 주는 것: 메서드의 첫 매개변수 `&self`, `&mut self`, `self`의 차이와,
//   메서드를 호출할 때 Rust가 `&`, `&mut`, `*`를 알아서 붙여 주는 자동 참조·역참조
// 실행: cargo run -p p0-ch5-structs --example 06_method_receivers

#[derive(Debug)]
struct Account {
    owner: String,
    balance: u64,
}

impl Account {
    // &self (= self: &Self): 인스턴스를 불변으로 빌립니다. 읽기만 하는 메서드는 이것을 씁니다.
    // `Self`는 이 impl 블록의 대상 타입, 여기서는 Account를 가리키는 별칭입니다.
    // 필드와 이름이 같은 메서드도 만들 수 있습니다. 괄호가 있으면 메서드, 없으면 필드입니다.
    fn balance(&self) -> u64 {
        self.balance
    }

    // &mut self (= self: &mut Self): 인스턴스를 가변으로 빌립니다. 필드를 바꾸는 메서드는 이것을 씁니다.
    // 빌려 오기만 하므로 메서드가 끝나도 인스턴스는 호출한 쪽에 그대로 남습니다.
    fn deposit(&mut self, amount: u64) {
        self.balance += amount;
    }

    // self (= self: Self): 인스턴스의 소유권을 가져옵니다(이동).
    // 계좌를 해지하면 계좌는 사라지고 영수증(String)만 남아야 하므로, 호출한 뒤에는 원래 계좌를 못 쓰게 하는 것이 맞습니다.
    // `format!`은 println!과 같은 서식 문법으로 출력 대신 String을 만들어 돌려줍니다(ch1).
    fn close(self) -> String {
        format!("{}님 계좌 해지, {}원 반환", self.owner, self.balance)
    } // self가 여기서 스코프를 벗어나 drop됩니다. 필요한 정보는 영수증에 옮겨 담았습니다.
}

// 가변 참조를 받는 일반 함수. 안에서 메서드를 부를 때 `(*account).balance()`처럼 쓰지 않습니다.
fn pay_interest(account: &mut Account) {
    // balance는 &Account가 필요한데 account는 &mut Account입니다.
    // Rust가 *로 참조를 따라간 뒤 &를 다시 붙여 Account::balance(&*account)로 호출합니다.
    let interest = account.balance() / 10;
    // deposit은 &mut Account가 필요하고 account가 바로 그 타입이므로 그대로 넘깁니다.
    account.deposit(interest);
}

fn main() {
    println!("1. &self 메서드는 읽기만 합니다");
    let mut account = Account {
        owner: String::from("페리스"),
        balance: 1000,
    };
    // account.balance()는 메서드, account.balance는 필드입니다.
    println!(
        "    메서드: {}, 필드: {}",
        account.balance(),
        account.balance
    );
    // 출력: 메서드: 1000, 필드: 1000

    println!("2. &mut self 메서드는 인스턴스를 바꿉니다");
    // deposit은 &mut self를 받으므로 account가 `let mut`이어야 합니다.
    // `let account`였다면 E0596 에러가 납니다(compile_fail 참고).
    account.deposit(500);
    println!("    {account:?}");
    // 출력: Account { owner: "페리스", balance: 1500 }

    println!("3. 점(.)으로 호출하면 Rust가 &나 &mut를 자동으로 붙입니다");
    // 메서드도 `타입::메서드(인스턴스, 인자)` 형태로 부를 수 있습니다.
    // 이렇게 쓰면 self 자리에 무엇이 들어가는지가 그대로 보입니다.
    Account::deposit(&mut account, 100); // account.deposit(100)과 같습니다.
    println!(
        "    Account::balance(&account) = {}, account.balance() = {}",
        Account::balance(&account),
        account.balance()
    );
    // 출력: Account::balance(&account) = 1600, account.balance() = 1600
    // 점 표기에서는 메서드의 self 모양(&self, &mut self, self)을 보고 Rust가 &, &mut를 정해 붙입니다.
    // 그래서 C/C++처럼 포인터용 `->`와 값용 `.`를 구분할 필요가 없습니다.

    println!("4. 참조를 통해 호출할 때도 *를 쓰지 않습니다");
    // pay_interest 안에서는 Rust가 필요한 만큼 역참조하고 다시 빌립니다(위 함수의 주석 참고).
    pay_interest(&mut account);
    let shared = &account; // shared의 타입은 &Account
    // balance가 원하는 타입이 바로 &Account이므로 shared를 그대로 넘깁니다. &를 한 번 더 붙이지 않습니다.
    println!("    이자 지급 후: {}", shared.balance());
    // 출력: 이자 지급 후: 1760

    println!("5. self 메서드는 인스턴스의 소유권을 가져갑니다");
    let receipt = account.close(); // account가 close의 self로 이동합니다.
    println!("    {receipt}");
    // 출력: 페리스님 계좌 해지, 1760원 반환
    // 아래 줄의 주석을 풀면 E0382가 납니다. close가 account의 소유권을 가져갔기 때문입니다.
    // println!("{}", account.balance());
}
