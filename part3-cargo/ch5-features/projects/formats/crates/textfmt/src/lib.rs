//! 표 하나를 여러 형식의 글로 바꾸는 라이브러리입니다.
//!
//! 형식마다 기능(feature)이 하나씩 있어, 쓰는 쪽이 필요한 형식만 골라 켭니다.
//!
//! | 기능 | 켜면 생기거나 달라지는 것 |
//! | --- | --- |
//! | `markdown` (기본) | `to_markdown` |
//! | `csv` | `to_csv` |
//! | `json` | `to_json` (선택적 의존성 `jsonesc`를 함께 켬) |
//! | `ascii` | `json`이 켜져 있으면 `to_json`이 ASCII가 아닌 글자를 `\uXXXX`로 씀(출력이 바뀜) |
//! | `full` | `markdown`, `csv`, `json` |

/// 머리글 한 줄과 데이터 줄들로 된 표입니다. 이 타입은 기능과 상관없이 항상 있습니다.
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    /// 문자열 조각들로 표를 만듭니다.
    pub fn new(headers: &[&str], rows: &[&[&str]]) -> Table {
        Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: rows
                .iter()
                .map(|row| row.iter().map(|cell| cell.to_string()).collect())
                .collect(),
        }
    }
}

/// 이 빌드에서 켜진 형식 기능의 이름을 돌려줍니다.
///
/// `cfg!(feature = "…")`는 컴파일할 때 정해지는 `true`/`false`입니다.
/// 같은 소스라도 어떤 기능으로 컴파일했느냐에 따라 결과가 달라집니다.
pub fn enabled_formats() -> Vec<&'static str> {
    let mut names = Vec::new();
    if cfg!(feature = "markdown") {
        names.push("markdown");
    }
    if cfg!(feature = "csv") {
        names.push("csv");
    }
    if cfg!(feature = "json") {
        names.push("json");
    }
    if cfg!(feature = "ascii") {
        names.push("ascii");
    }
    names
}

/// 마크다운 표로 바꿉니다.
#[cfg(feature = "markdown")]
pub fn to_markdown(table: &Table) -> String {
    let line = |cells: &[String]| format!("| {} |", cells.join(" | "));
    let mut out = vec![line(&table.headers)];
    out.push(format!("|{}", " --- |".repeat(table.headers.len())));
    out.extend(table.rows.iter().map(|row| line(row)));
    out.join("\n")
}

/// 쉼표로 구분한 값(CSV)으로 바꿉니다. 쉼표나 큰따옴표가 든 칸은 큰따옴표로 감쌉니다.
#[cfg(feature = "csv")]
pub fn to_csv(table: &Table) -> String {
    let cell = |c: &String| {
        if c.contains(',') || c.contains('"') {
            format!("\"{}\"", c.replace('"', "\"\""))
        } else {
            c.clone()
        }
    };
    let line = |cells: &[String]| cells.iter().map(cell).collect::<Vec<_>>().join(",");
    let mut out = vec![line(&table.headers)];
    out.extend(table.rows.iter().map(|row| line(row)));
    out.join("\n")
}

/// 줄마다 객체 하나인 JSON 배열로 바꿉니다. 문자열 처리는 `jsonesc`에 맡깁니다.
#[cfg(feature = "json")]
pub fn to_json(table: &Table) -> String {
    let objects: Vec<String> = table
        .rows
        .iter()
        .map(|row| {
            let fields: Vec<String> = table
                .headers
                .iter()
                .zip(row)
                .map(|(h, v)| format!("{}: {}", jsonesc::quote(h), jsonesc::quote(v)))
                .collect();
            format!("  {{{}}}", fields.join(", "))
        })
        .collect();
    format!("[\n{}\n]", objects.join(",\n"))
}

#[cfg(all(test, any(feature = "csv", feature = "markdown")))]
mod tests {
    use super::*;

    fn sample() -> Table {
        Table::new(&["이름", "값"], &[&["a,b", "1"]])
    }

    #[test]
    #[cfg(feature = "csv")]
    fn csv_quotes_commas() {
        assert_eq!(to_csv(&sample()), "이름,값\n\"a,b\",1");
    }

    #[test]
    #[cfg(feature = "markdown")]
    fn markdown_has_separator() {
        assert!(to_markdown(&sample()).contains("| --- | --- |"));
    }
}
