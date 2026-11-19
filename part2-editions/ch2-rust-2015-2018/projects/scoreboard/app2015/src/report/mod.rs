// 점수표를 그리는 모듈입니다.
//
// 2015 관례: 하위 모듈(table)이 있는 모듈은 디렉터리를 만들고 본문을 report/mod.rs에 둡니다.

mod table;

// 2015: `use` 경로는 크레이트 루트에서 출발하므로, 자식 모듈은 self::로 가리킵니다.
pub use self::table::render;

/// 한 줄을 어떻게 그릴지 정하는 트레이트입니다.
// 2015 문법: 트레이트 메서드 선언에 매개변수 이름 없이 타입만 적었습니다(2018부터 에러).
#[allow(anonymous_parameters)]
pub trait Style {
    fn line(&self, &str, u32) -> String;
}

pub struct Plain;
pub struct Graph;

impl Style for Plain {
    fn line(&self, name: &str, score: u32) -> String {
        format!("{} {:>3}점 {}", name, score, grade(score))
    }
}

impl Style for Graph {
    fn line(&self, name: &str, score: u32) -> String {
        // bar!는 main.rs의 `#[macro_use] extern crate scorekit;` 덕분에 이 모듈에서도 보입니다.
        format!("{} {}", name, bar!(score))
    }
}

/// 이름으로 스타일을 고릅니다.
// 2015 문법: 트레이트 객체 타입을 트레이트 이름만으로 적었습니다(Box<Style>).
#[allow(bare_trait_objects)]
pub fn style_for(name: &str) -> Box<Style> {
    if name == "graph" {
        Box::new(Graph)
    } else {
        Box::new(Plain)
    }
}

/// 점수를 등급으로 바꿉니다.
// 2015 문법: 끝을 포함하는 범위 패턴을 ...로 적었습니다.
#[allow(ellipsis_inclusive_range_patterns)]
pub fn grade(score: u32) -> char {
    match score {
        90...100 => 'A',
        80...89 => 'B',
        70...79 => 'C',
        _ => 'F',
    }
}
