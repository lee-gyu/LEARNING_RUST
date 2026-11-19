//! 가계부 항목과 카테고리

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Food,
    Transport,
    Housing,
    Etc,
}

/// 보고서는 언제나 이 순서로 카테고리를 보여 줍니다.
pub const CATEGORIES: [Category; 4] = [
    Category::Food,
    Category::Transport,
    Category::Housing,
    Category::Etc,
];

impl Category {
    pub fn name(self) -> &'static str {
        match self {
            Category::Food => "식비",
            Category::Transport => "교통",
            Category::Housing => "주거",
            Category::Etc => "기타",
        }
    }
}

impl TryFrom<&str> for Category {
    type Error = String;

    fn try_from(name: &str) -> Result<Category, String> {
        CATEGORIES
            .into_iter()
            .find(|c| c.name() == name)
            .ok_or_else(|| format!("모르는 카테고리: {}", name))
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub category: Category,
    /// 원 단위. 지출은 양수, 환불은 음수입니다.
    pub amount: i64,
    pub memo: String,
}

impl Entry {
    /// `"식비 12000 점심"` 꼴의 줄을 읽습니다.
    pub fn parse(line: &str) -> Result<Entry, String> {
        let mut parts = line.split_whitespace();
        let category = Category::try_from(parts.next().unwrap_or(""))?;
        let amount = parts
            .next()
            .ok_or("금액이 없습니다")?
            .parse::<i64>()
            .map_err(|e| e.to_string())?;
        let memo = parts.collect::<Vec<_>>().join(" ");
        Ok(Entry {
            category,
            amount,
            memo,
        })
    }
}

/// 0~100 사이의 정수만 백분율로 인정합니다.
///
/// 표준 라이브러리의 `TryInto`가 없던 시절에 만든 변환 트레이트입니다.
pub trait Percent {
    fn try_into(&self) -> Result<u8, String>;
}

impl Percent for i64 {
    fn try_into(&self) -> Result<u8, String> {
        if (0..=100).contains(self) {
            Ok(*self as u8)
        } else {
            Err(format!("{}%는 백분율이 아닙니다", self))
        }
    }
}
