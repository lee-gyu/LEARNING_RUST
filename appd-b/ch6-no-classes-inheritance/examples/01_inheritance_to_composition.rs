// Q6.2 · Q6.4 · Q6.10 — 상속 계층을 조합과 트레이트로 옮기기
//
// Java라면 이렇게 쓸 급여 계산을 Rust로 옮깁니다.
//
//   class Employee {
//       protected String name; protected int annual;
//       int monthlyPay() { return annual / 12; }
//       String paySlip() { return name + ": 월 " + monthlyPay() + "만 원"; }
//   }
//   class Manager extends Employee {
//       int bonus;
//       @Override int monthlyPay() { return super.monthlyPay() + bonus; }
//   }
//
// - 물려받던 메서드(`paySlip`) → 트레이트의 기본 메서드. 트레이트는 필드를 가질 수 없으므로 기본 메서드는
//   필요한 값을 `name()`, `monthly_pay()` 같은 메서드로 얻습니다(기본 메서드 자체는 Part 0 ch10 예제 03).
// - 물려받던 필드 → `Manager`가 `Employee`를 필드로 품는 조합. 바깥 값의 메서드가 저절로 올라오지 않으므로
//   (Go의 임베딩과 다른 점) 필요한 메서드를 직접 넘겨줍니다(위임).
// - `super.monthlyPay()` → `self.base.monthly_pay()`
// - `protected` → 없음. 공개 범위는 타입이 아니라 모듈 단위라서, 같은 모듈의 `Manager` 코드는 `Employee`의
//   비공개 필드를 읽을 수 있습니다(Part 0 ch7 2.5절).
//
// 실행: cargo run -p faq-ch6-no-classes-inheritance --example 01_inheritance_to_composition

mod payroll {
    pub trait Payable {
        fn name(&self) -> &str;
        fn monthly_pay(&self) -> u32;

        /// 기본 메서드: 구현하는 타입이 따로 쓰지 않으면 이 본문을 씁니다.
        fn pay_slip(&self) -> String {
            format!("{}: 월 {}만 원", self.name(), self.monthly_pay())
        }
    }

    pub struct Employee {
        name: String, // 비공개 필드: 이 모듈 밖에서는 읽을 수 없습니다.
        annual: u32,
    }

    impl Employee {
        pub fn new(name: &str, annual: u32) -> Employee {
            Employee {
                name: name.to_string(),
                annual,
            }
        }
    }

    impl Payable for Employee {
        fn name(&self) -> &str {
            &self.name
        }

        fn monthly_pay(&self) -> u32 {
            self.annual / 12
        }
    }

    /// `extends Employee` 대신 `Employee`를 필드로 품습니다.
    pub struct Manager {
        base: Employee,
        bonus: u32,
    }

    impl Manager {
        pub fn new(name: &str, annual: u32, bonus: u32) -> Manager {
            Manager {
                base: Employee::new(name, annual),
                bonus,
            }
        }

        /// 연간 총액. 같은 모듈이라 `Employee`의 비공개 필드 `annual`을 바로 읽습니다.
        pub fn annual_total(&self) -> u32 {
            self.base.annual + self.bonus * 12
        }
    }

    impl Payable for Manager {
        fn name(&self) -> &str {
            self.base.name() // 위임: 품은 값의 메서드를 직접 부릅니다.
        }

        fn monthly_pay(&self) -> u32 {
            self.base.monthly_pay() + self.bonus // super.monthlyPay() 자리
        }
        // pay_slip은 쓰지 않았으므로 기본 메서드를 그대로 씁니다.
    }
}

use payroll::{Employee, Manager, Payable};

fn main() {
    let staff = Employee::new("김하나", 3600);
    let lead = Manager::new("이두리", 4200, 100);

    println!("1. 기본 메서드와 위임");
    // 부모 타입의 목록 대신, 트레이트를 구현한 값들의 목록(트레이트 객체, Part 0 ch17)
    let people: [&dyn Payable; 2] = [&staff, &lead];
    for p in people {
        println!("  {}", p.pay_slip());
    }
    // 출력:   김하나: 월 300만 원
    // 출력:   이두리: 월 450만 원

    println!("2. 같은 모듈이면 다른 타입의 비공개 필드도 읽음");
    println!(
        "  {}의 연간 총액: {}만 원",
        lead.name(),
        lead.annual_total()
    );
    // 출력:   이두리의 연간 총액: 5400만 원
    // 여기는 payroll 모듈 밖이므로 lead.base나 staff.annual은 쓸 수 없습니다(E0616, Part 0 ch7 4.2절).
}
