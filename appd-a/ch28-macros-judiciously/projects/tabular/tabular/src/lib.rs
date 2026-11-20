//! 구조체를 ` | `로 나눈 표로 출력하는 작은 라이브러리입니다.
//!
//! - [`Columns`] 트레이트: 열 이름과 한 행의 칸
//! - `#[derive(Columns)]`: `tabular_derive`가 만들고 이 크레이트가 재내보냅니다.
//! - [`row!`]: 식 몇 개를 "식의 글자 | 값" 표로 만드는 선언적 매크로
//!
//! 비교를 위해 `$crate` 없이 쓴 비권장 매크로 세 개(`row_bare!`, `row_crate!`, `row_private!`)도 함께 내보냅니다.
//! 실제 라이브러리라면 내보내지 않을 매크로입니다. 이 크레이트의 테스트에서는 넷 다 동작하지만,
//! 다른 크레이트(`user_fails`)에서 부르면 `row!` 말고는 컴파일되지 않습니다.

use std::fmt::Display;

// 트레이트와 derive 매크로가 같은 이름 `Columns`를 씁니다. 이름 공간이 달라 함께 둘 수 있고,
// 쓰는 쪽은 `use tabular::Columns;` 한 줄로 둘을 모두 가져옵니다.
pub use tabular_derive::Columns;

/// 표의 한 행이 될 수 있는 타입입니다.
pub trait Columns {
    /// 열 이름입니다. [`Columns::cells`]가 돌려주는 칸과 개수·순서가 같아야 합니다.
    const NAMES: &'static [&'static str];

    /// 이 행의 칸입니다.
    fn cells(&self) -> Vec<String>;
}

/// 머리글과 행을 ` | `로 나눈 표로 만듭니다.
///
/// 칸 개수가 머리글과 다른 행에는 끝에 표시를 붙입니다. 손으로 쓴 구현이 어긋났을 때 눈에 띄게 하려는 것입니다.
pub fn render<T: Columns>(rows: &[T]) -> String {
    let mut out = T::NAMES.join(" | ");
    for row in rows {
        let cells = row.cells();
        out.push('\n');
        out.push_str(&cells.join(" | "));
        if cells.len() != T::NAMES.len() {
            out.push_str(&format!(
                "   ← 칸 {}개, 머리글 {}개",
                cells.len(),
                T::NAMES.len()
            ));
        }
    }
    out
}

// row!가 하는 일의 실제 구현입니다. 비공개 함수입니다.
fn format_row(pairs: &[(&str, &dyn Display)]) -> String {
    let header: Vec<&str> = pairs.iter().map(|(text, _)| *text).collect();
    let values: Vec<String> = pairs.iter().map(|(_, value)| value.to_string()).collect();
    format!("{}\n{}", header.join(" | "), values.join(" | "))
}

/// `row!`가 펼친 코드에서 부르는 함수입니다.
///
/// 다른 크레이트에서 펼쳐진 코드가 부를 수 있도록 `pub`이어야 하지만, 직접 쓰라고 내놓은 API는 아니므로
/// 문서에서 숨기고 이름 앞에 `__`를 붙였습니다.
#[doc(hidden)]
pub fn __format_row(pairs: &[(&str, &dyn Display)]) -> String {
    format_row(pairs)
}

/// 식 몇 개를 두 줄짜리 표로 만듭니다. 첫 줄은 식의 글자, 둘째 줄은 값입니다.
///
/// `println!`처럼 인자를 빌려 읽기만 하고 옮기지 않습니다. 각 식은 한 번만 평가됩니다.
///
/// 권장 판: 자기 크레이트의 함수를 `$crate::`로 가리킵니다.
#[macro_export]
macro_rules! row {
    ($($value:expr),+ $(,)?) => {
        $crate::__format_row(&[$(
            (::core::stringify!($value), &$value as &dyn ::core::fmt::Display)
        ),+])
    };
}

/// 비권장 판 ①: 경로 없이 `__format_row`라고 씁니다. 쓰는 쪽에서 그 이름을 찾지 못합니다(E0425).
#[macro_export]
macro_rules! row_bare {
    ($($value:expr),+ $(,)?) => {
        __format_row(&[$(
            (::core::stringify!($value), &$value as &dyn ::core::fmt::Display)
        ),+])
    };
}

/// 비권장 판 ②: `crate::`로 씁니다. 쓰는 쪽에서는 쓰는 쪽 크레이트의 루트를 가리킵니다(E0425).
#[macro_export]
#[expect(
    clippy::crate_in_macro_def,
    reason = "비교용 비권장 판: crate::는 부른 크레이트를 가리킨다"
)]
macro_rules! row_crate {
    ($($value:expr),+ $(,)?) => {
        crate::__format_row(&[$(
            (::core::stringify!($value), &$value as &dyn ::core::fmt::Display)
        ),+])
    };
}

/// 비권장 판 ③: `$crate::`로 찾아가지만 비공개 함수를 부릅니다. `$crate`는 가시성을 바꾸지 않습니다(E0603).
#[macro_export]
macro_rules! row_private {
    ($($value:expr),+ $(,)?) => {
        $crate::format_row(&[$(
            (::core::stringify!($value), &$value as &dyn ::core::fmt::Display)
        ),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Point {
        x: i32,
        y: i32,
    }

    impl Columns for Point {
        const NAMES: &'static [&'static str] = &["x", "y"];

        fn cells(&self) -> Vec<String> {
            vec![self.x.to_string(), self.y.to_string()]
        }
    }

    // 열 이름에 하나를 빠뜨린 구현입니다.
    struct Mismatched;

    impl Columns for Mismatched {
        const NAMES: &'static [&'static str] = &["a"];

        fn cells(&self) -> Vec<String> {
            vec!["1".to_string(), "2".to_string()]
        }
    }

    #[test]
    fn render_writes_header_then_rows() {
        let rows = [Point { x: 1, y: 2 }, Point { x: -3, y: 4 }];
        assert_eq!(render(&rows), "x | y\n1 | 2\n-3 | 4");
    }

    #[test]
    fn render_marks_rows_with_wrong_cell_count() {
        assert_eq!(render(&[Mismatched]), "a\n1 | 2   ← 칸 2개, 머리글 1개");
    }

    // 정의한 크레이트 안에서는 비권장 판 셋도 모두 컴파일되고 통과합니다.
    // 이 테스트만으로는 다른 크레이트에서 깨진다는 것을 알 수 없습니다(user_fails에서 확인).
    #[test]
    fn every_row_macro_works_inside_the_defining_crate() {
        let total = 3;
        let average = 1.5;
        let expected = "total | average\n3 | 1.5";
        assert_eq!(row!(total, average), expected);
        assert_eq!(row_bare!(total, average), expected);
        assert_eq!(row_crate!(total, average), expected);
        assert_eq!(row_private!(total, average), expected);
    }
}
