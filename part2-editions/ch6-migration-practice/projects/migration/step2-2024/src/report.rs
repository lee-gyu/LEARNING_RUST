//! 보고서 만들기

use crate::book::Ledger;
use crate::entry::{CATEGORIES, Percent};

/// 카테고리 한 줄을 어떻게 그릴지 정하는 스타일
pub trait Style {
    fn line(&self, name: &str, amount: i64, share: Option<u8>) -> String;
}

/// `식비   30000원  75%`
pub struct Plain;

/// `식비 #######`
pub struct Bar;

impl Style for Plain {
    fn line(&self, name: &str, amount: i64, share: Option<u8>) -> String {
        match share {
            Some(p) => format!("{} {:>7}원 {:>3}%", name, amount, p),
            None => format!("{} {:>7}원    -", name, amount),
        }
    }
}

impl Style for Bar {
    fn line(&self, name: &str, _amount: i64, share: Option<u8>) -> String {
        let width = share.map_or(0, |p| usize::from(p / 10));
        format!("{} {}", name, "#".repeat(width))
    }
}

/// 이름으로 스타일을 고릅니다. 모르는 이름이면 `Plain`입니다.
pub fn style_for(name: &str) -> Box<dyn Style> {
    match name {
        "bar" => Box::new(Bar),
        _ => Box::new(Plain),
    }
}

/// 한 달 지출 총액의 등급
pub fn grade(total: i64) -> &'static str {
    match total {
        0..=99_999 => "알뜰",
        100_000..=499_999 => "보통",
        _ if total < 0 => "환불 초과",
        _ => "과소비",
    }
}

/// 카테고리별 합계와 비율, 총액을 보고서로 만듭니다.
pub fn render(book: &Ledger, style: &dyn Style) -> String {
    let total = book.balance();
    let mut lines = Vec::new();
    for &c in CATEGORIES.iter() {
        let sum = book.total_of(c);
        if sum == 0 {
            continue;
        }
        // 환불이 있으면 비율이 100을 넘을 수 있습니다. Percent::try_into는 그런 값을 거절합니다.
        let share = if total == 0 {
            None
        } else {
            Percent::try_into(&(sum * 100 / total)).ok()
        };
        lines.push(style.line(c.name(), sum, share));
    }
    lines.push(format!("합계 {}원 ({})", total, grade(total)));
    lines.join("\n")
}
