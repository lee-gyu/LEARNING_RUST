//! 항목 개수를 세고 많은 순으로 보여 주는 작은 라이브러리입니다.
//!
//! ch32의 CI 실습용으로 만든 크레이트입니다. 기능 두 개, MSRV, 문서 테스트,
//! 통합 테스트, 예제 프로그램을 모두 갖춰 `ci.sh`의 단계마다 검사할 거리가 있습니다.
//!
//! ```
//! use pollcount::Votes;
//!
//! let t: Votes = ["사과", "배", "사과"].into_iter().collect();
//! assert_eq!(t.count("사과"), 2);
//! assert_eq!(t.top(1), vec![("사과", 2)]);
//! ```
//!
//! # 기능
//!
//! - `csv` (기본): [`Votes::to_csv`]로 CSV 문자열을 만듭니다.
//! - `json`: [`Votes::to_json`]으로 JSON 문자열을 만듭니다.

use std::collections::BTreeMap;
use std::convert::Infallible;
use std::str::FromStr;

// 내보내기 함수만 `write!`를 씁니다. 기능이 하나도 없는 빌드에서
// 쓰지 않는 가져오기 경고가 나지 않도록, 쓰는 기능들로 가립니다.
#[cfg(any(feature = "csv", feature = "json"))]
use std::fmt::Write as _;

/// 항목별 개수를 모읍니다.
///
/// 항목은 앞뒤 공백을 지운 문자열로 비교합니다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Votes {
    counts: BTreeMap<String, u64>,
}

impl Votes {
    /// 빈 집계를 만듭니다.
    pub fn new() -> Self {
        Self::default()
    }

    /// 항목 하나를 셉니다. 앞뒤 공백을 지우고, 비어 있으면 세지 않습니다.
    ///
    /// ```
    /// let mut t = pollcount::Votes::new();
    /// t.add(" 사과 ");
    /// t.add("   ");
    /// assert_eq!(t.count("사과"), 1);
    /// assert_eq!(t.total(), 1);
    /// ```
    pub fn add(&mut self, item: &str) {
        let item = item.trim();
        if item.is_empty() {
            return;
        }
        *self.counts.entry(item.to_owned()).or_insert(0) += 1;
    }

    /// 항목의 개수를 돌려줍니다. 센 적이 없으면 0입니다.
    pub fn count(&self, item: &str) -> u64 {
        self.counts.get(item.trim()).copied().unwrap_or(0)
    }

    /// 센 항목의 총 개수입니다.
    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }

    /// 서로 다른 항목의 수입니다.
    pub fn len(&self) -> usize {
        self.counts.len()
    }

    /// 센 항목이 하나도 없으면 `true`입니다.
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// 많은 순으로 최대 `n`개를 돌려줍니다. 개수가 같으면 이름순입니다.
    ///
    /// ```
    /// let t: pollcount::Votes = "배, 사과, 배, 귤".parse().unwrap();
    /// assert_eq!(t.top(2), vec![("배", 2), ("귤", 1)]);
    /// ```
    pub fn top(&self, n: usize) -> Vec<(&str, u64)> {
        let mut rows: Vec<(&str, u64)> = self
            .counts
            .iter()
            .map(|(item, &count)| (item.as_str(), count))
            .collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        rows.truncate(n);
        rows
    }

    /// 많은 순으로 `item,count` 머리줄이 붙은 CSV를 만듭니다.
    ///
    /// 쉼표나 큰따옴표가 든 항목은 큰따옴표로 감쌉니다.
    ///
    /// ```
    /// let t: pollcount::Votes = "배, 사과, 배".parse().unwrap();
    /// assert_eq!(t.to_csv(), "item,count\n배,2\n사과,1\n");
    /// ```
    #[cfg(feature = "csv")]
    pub fn to_csv(&self) -> String {
        let mut out = String::from("item,count\n");
        for (item, count) in self.top(usize::MAX) {
            if item.contains([',', '"']) {
                let quoted = item.replace('"', "\"\"");
                let _ = writeln!(out, "\"{quoted}\",{count}");
            } else {
                let _ = writeln!(out, "{item},{count}");
            }
        }
        out
    }

    /// 많은 순으로 `{"item":…,"count":…}` 객체의 배열을 JSON으로 만듭니다.
    ///
    /// ```
    /// let t: pollcount::Votes = "배, 사과, 배".parse().unwrap();
    /// assert_eq!(
    ///     t.to_json(),
    ///     r#"[{"item":"배","count":2},{"item":"사과","count":1}]"#
    /// );
    /// ```
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        let mut out = String::from("[");
        for (i, (item, count)) in self.top(usize::MAX).into_iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"item\":\"");
            for c in item.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    c if c.is_control() => {
                        let _ = write!(out, "\\u{:04x}", c as u32);
                    }
                    c => out.push(c),
                }
            }
            let _ = write!(out, "\",\"count\":{count}}}");
        }
        out.push(']');
        out
    }
}

impl<'a> Extend<&'a str> for Votes {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, items: I) {
        for item in items {
            self.add(item);
        }
    }
}

impl<'a> FromIterator<&'a str> for Votes {
    fn from_iter<I: IntoIterator<Item = &'a str>>(items: I) -> Self {
        let mut t = Votes::new();
        t.extend(items);
        t
    }
}

/// 쉼표로 나눈 목록을 셉니다. 빈 칸은 건너뛰므로 실패하지 않습니다.
impl FromStr for Votes {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.split(',').collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ties_are_sorted_by_name() {
        let t: Votes = "다, 가, 나".parse().unwrap();
        assert_eq!(t.top(3), vec![("가", 1), ("나", 1), ("다", 1)]);
    }

    #[cfg(feature = "csv")]
    #[test]
    fn csv_quotes_special_items() {
        let t: Votes = ["a,b", "say \"hi\""].into_iter().collect();
        assert_eq!(t.to_csv(), "item,count\n\"a,b\",1\n\"say \"\"hi\"\"\",1\n");
    }

    #[cfg(feature = "json")]
    #[test]
    fn json_escapes_quotes_and_controls() {
        let t: Votes = ["a\"b", "tab\there"].into_iter().collect();
        assert_eq!(
            t.to_json(),
            r#"[{"item":"a\"b","count":1},{"item":"tab\u0009here","count":1}]"#
        );
    }
}
