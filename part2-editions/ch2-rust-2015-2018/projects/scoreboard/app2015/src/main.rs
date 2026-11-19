// 점수표 프로그램 (2015 에디션판)
//
// "이름 점수" 줄들을 읽어 두 가지 스타일로 출력합니다. app2018과 출력이 같습니다.
// 실행: cargo run -p app2015

// 2015: 외부 크레이트는 `extern crate`로 선언해야 `use` 경로에서 쓸 수 있습니다.
// `#[macro_use]`를 붙이면 그 크레이트가 내보낸 매크로(bar!)를 이 크레이트 어디서나 쓸 수 있습니다.
#[macro_use]
extern crate scorekit;

mod record;
mod report;

const INPUT: &str = "민수 87\n지영 93\n철호 8O\n수진 100\n현우 72";

fn main() {
    let mut records = Vec::new();
    for line in INPUT.lines() {
        match record::parse_line(line) {
            Ok(r) => records.push(r),
            Err(e) => println!("건너뜀: {}", e),
        }
    }

    for name in &["plain", "graph"] {
        println!("[{}]", name);
        let style = report::style_for(name);
        report::render(&records, style.as_ref());
    }
}
