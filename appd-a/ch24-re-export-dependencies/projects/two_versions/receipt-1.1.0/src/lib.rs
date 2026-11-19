//! receipt 1.1.0: 주문 내역을 모아 영수증 문자열을 만드는 라이브러리입니다.
//!
//! money 0.3의 `Money`(타입)와 `Priced`(트레이트)가 공개 API에 드러나므로 money는 이 크레이트의
//! 공개 의존성입니다. 1.1.0은 1.0.0과 달리 money를 통째로 재내보냅니다.

use std::fmt::{Display, Write};

use money::{Money, Priced};

/// 공개 API에 드러나는 의존성을 통째로 재내보냅니다. 사용자는 `receipt::money::won(5000)`처럼
/// 이 크레이트가 쓰는 바로 그 버전의 money에 닿습니다. 타입뿐 아니라 생성자 함수 `won`도 쓸 수 있습니다.
pub use money;

pub const VERSION: &str = "1.1.0";

#[derive(Debug, Default)]
pub struct Receipt {
    lines: Vec<(String, Money)>,
}

impl Receipt {
    pub fn new() -> Receipt {
        Receipt::default()
    }

    /// 원 단위 정수로 품목을 더합니다. money의 타입이 드러나지 않는 메서드입니다.
    pub fn add_won(&mut self, name: &str, won: i64) {
        self.add(name, money::won(won));
    }

    /// money 0.3의 `Money`를 받습니다.
    pub fn add(&mut self, name: &str, price: Money) {
        self.lines.push((name.to_string(), price));
    }

    /// money 0.3의 `Priced`를 구현한 품목을 받습니다. 품목 이름은 `Display`로 얻습니다.
    pub fn add_item<T: Priced + Display>(&mut self, item: &T) {
        self.add(&item.to_string(), item.price());
    }

    /// 합계를 money 0.3의 `Money`로 돌려줍니다.
    pub fn total(&self) -> Money {
        self.lines.iter().map(|(_, price)| *price).sum()
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for (name, price) in &self.lines {
            writeln!(out, "  {name}: {price}").unwrap();
        }
        writeln!(out, "  합계: {}", self.total()).unwrap();
        out
    }
}
