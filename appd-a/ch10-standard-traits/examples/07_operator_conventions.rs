// 지침: 연산자는 읽는 사람이 기호만 보고 결과를 짐작할 수 있을 때만 오버로딩합니다.
//       구현한 연산자끼리도 통념대로 맞물려야 합니다(a += b는 a = a + b와 같은 결과).
// 비권장: 스타일 덮어쓰기를 +로 구현 (a + b와 b + a가 다름)
//         볼륨의 +는 상한을 지키는데 +=는 상한을 잊음
// 권장:   덮어쓰기는 이름 붙은 메서드, +=는 +를 불러서 구현
//
// 실행: cargo run -p er-ch10-standard-traits --example 07_operator_conventions

use std::ops::{Add, AddAssign};

// ---- 기호만으로 뜻을 짐작할 수 없는 연산 -------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Black,
    Red,
}

/// 글자 스타일. None은 "정하지 않음"이라 다른 스타일의 값을 따릅니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Style {
    bold: Option<bool>,
    color: Option<Color>,
}

// 비권장: 오른쪽이 왼쪽을 덮어쓰는 병합을 +로 구현했습니다.
// 덧셈은 순서를 바꿔도 결과가 같다는 통념이 있어서, 읽는 사람은 어느 쪽이 이기는지 알 수 없습니다.
impl Add for Style {
    type Output = Style;
    fn add(self, rhs: Style) -> Style {
        Style {
            bold: rhs.bold.or(self.bold),
            color: rhs.color.or(self.color),
        }
    }
}

// 권장: 방향을 이름에 담습니다. 메서드 이름을 add로 지으면 clippy의 should_implement_trait가
// "Add를 구현하라"고 권하므로, 연산자로 오해받지 않는 이름을 고릅니다.
impl Style {
    fn overridden_by(self, top: Style) -> Style {
        Style {
            bold: top.bold.or(self.bold),
            color: top.color.or(self.color),
        }
    }
}

fn describe(style: Style) -> String {
    let weight = if style.bold == Some(true) {
        "굵게"
    } else {
        "보통"
    };
    let color = match style.color {
        Some(Color::Red) => "빨강",
        Some(Color::Black) | None => "검정",
    };
    format!("{weight}, {color}")
}

// ---- 함께 구현한 연산자끼리 맞물리기 -----------------------------------------

/// 0~100 사이의 볼륨. 올릴 때 100에서 멈춥니다(표준 라이브러리의 Saturating과 같은 뜻).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LooseVolume(u8);

impl Add<u8> for LooseVolume {
    type Output = LooseVolume;
    fn add(self, rhs: u8) -> LooseVolume {
        LooseVolume(self.0.saturating_add(rhs).min(100))
    }
}

// 비권장: +=를 따로 쓰면서 상한을 잊었습니다. +와 +=가 다른 답을 냅니다.
impl AddAssign<u8> for LooseVolume {
    fn add_assign(&mut self, rhs: u8) {
        self.0 = self.0.saturating_add(rhs);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Volume(u8);

impl Add<u8> for Volume {
    type Output = Volume;
    fn add(self, rhs: u8) -> Volume {
        Volume(self.0.saturating_add(rhs).min(100))
    }
}

// 권장: +=는 +를 불러서 씁니다. 규칙이 한 곳에만 있으므로 둘이 어긋날 수 없습니다.
impl AddAssign<u8> for Volume {
    fn add_assign(&mut self, rhs: u8) {
        *self = *self + rhs;
    }
}

fn main() {
    let theme = Style {
        bold: Some(false),
        color: Some(Color::Black),
    };
    let warning = Style {
        bold: Some(true),
        color: Some(Color::Red),
    };

    println!("[비권장] 덮어쓰기를 +로");
    println!("  theme + warning = {}", describe(theme + warning));
    // 출력:   theme + warning = 굵게, 빨강
    // "더하기니까 순서는 상관없다"고 생각한 다른 사람의 코드:
    println!("  warning + theme = {}", describe(warning + theme));
    // 출력:   warning + theme = 보통, 검정
    println!("  같은가? {}", theme + warning == warning + theme);
    // 출력:   같은가? false

    println!("[권장] 방향이 이름에 드러나는 메서드");
    println!(
        "  theme.overridden_by(warning) = {}",
        describe(theme.overridden_by(warning))
    );
    // 출력:   theme.overridden_by(warning) = 굵게, 빨강

    println!("[비권장] +와 +=가 따로 구현됨");
    let start = LooseVolume(90);
    let mut turned_up = start;
    turned_up += 30;
    println!(
        "  {:?} + 30 = {:?}, += 30 → {:?}",
        start,
        start + 30,
        turned_up
    );
    // 출력:   LooseVolume(90) + 30 = LooseVolume(100), += 30 → LooseVolume(120)

    println!("[권장] +=는 +를 불러서");
    let start = Volume(90);
    let mut turned_up = start;
    turned_up += 30;
    println!(
        "  {:?} + 30 = {:?}, += 30 → {:?}",
        start,
        start + 30,
        turned_up
    );
    // 출력:   Volume(90) + 30 = Volume(100), += 30 → Volume(100)
}
