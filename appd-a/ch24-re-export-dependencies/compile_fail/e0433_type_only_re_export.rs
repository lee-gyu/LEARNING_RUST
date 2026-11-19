// 기대하는 에러: E0433 cannot find `money` in `receipt` (rustc 1.100.0의 문구)
//
// 컴파일러의 첫 help는 `receipt::Money::won(5000)`으로 바꾸라고 하지만, `Money`에는 연관 함수
// `won`이 없어서 그대로 따르면 E0599로 바뀔 뿐입니다(README 4.3절).
//
// receipt가 의존성 money의 타입과 트레이트만 재내보내고 크레이트 전체는 재내보내지 않았습니다.
// 사용자는 `receipt::Money`라는 이름은 쓸 수 있지만, money의 생성자 함수 `won`에 닿는 경로가 없어서
// receipt가 기대하는 버전의 `Money` 값을 만들 수 없습니다.
//
// 확인: rustc --edition 2024 compile_fail/e0433_type_only_re_export.rs --out-dir <임시 디렉터리>
//
// 모듈 `money`는 receipt가 의존하는 money 0.3이라고 칩니다. 사용자는 자기 `money`로 다른 버전을
// 쓴다고 보므로 `crate::money`를 직접 쓰지 않고 receipt를 거칩니다.

pub mod money {
    pub struct Money {
        won: i64,
    }

    pub fn won(amount: i64) -> Money {
        Money { won: amount }
    }

    impl Money {
        pub fn as_won(&self) -> i64 {
            self.won
        }
    }
}

pub mod receipt {
    // 타입만 재내보냄
    pub use crate::money::Money;

    pub fn line(name: &str, price: Money) -> String {
        format!("{name}: {}원", price.as_won())
    }
}

fn main() {
    // money 문서의 사용법(`money::won(5000)`)을 receipt 경로로 옮겨 적었지만,
    // receipt 안에는 `money`라는 이름이 없습니다.
    let latte = receipt::money::won(5000); // E0433
    println!("{}", receipt::line("카페라테", latte));
}
