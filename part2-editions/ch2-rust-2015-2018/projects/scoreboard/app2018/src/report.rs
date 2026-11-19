// 점수표를 그리는 모듈입니다.
//
// 2018 관례: 하위 모듈(table)이 있어도 본문은 report.rs에 두고, 하위 모듈은 report/table.rs에 둡니다.

mod table;

// 2018: `use` 경로도 다른 경로처럼 현재 모듈에서 출발하므로, 자식 모듈을 이름으로 바로 씁니다.
pub use table::render;

// 2018: 매크로도 경로로 가져옵니다. 2015판의 `#[macro_use] extern crate`를 대신합니다.
use scorekit::bar;

/// 한 줄을 어떻게 그릴지 정하는 트레이트입니다.
// 2018: 트레이트 메서드 선언에도 매개변수 이름을 적습니다(쓰지 않을 이름이면 _).
pub trait Style {
    fn line(&self, name: &str, score: u32) -> String;
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
        // bar!는 위의 `use scorekit::bar;`로 이 모듈에 가져왔습니다.
        format!("{} {}", name, bar!(score))
    }
}

/// 이름으로 스타일을 고릅니다.
// 2018: 트레이트 객체 타입에는 dyn을 붙입니다(Box<dyn Style>).
pub fn style_for(name: &str) -> Box<dyn Style> {
    if name == "graph" {
        Box::new(Graph)
    } else {
        Box::new(Plain)
    }
}

/// 점수를 등급으로 바꿉니다.
// 2018: 끝을 포함하는 범위 패턴은 ..=로 적습니다.
pub fn grade(score: u32) -> char {
    match score {
        90..=100 => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        _ => 'F',
    }
}
