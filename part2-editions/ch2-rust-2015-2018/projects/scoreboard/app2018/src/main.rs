// 점수표 프로그램 (2018 에디션판)
//
// "이름 점수" 줄들을 읽어 두 가지 스타일로 출력합니다. app2015와 출력이 같습니다.
// 실행: cargo run -p app2018

// 2018: Cargo.toml의 [dependencies]에 적은 크레이트는 `extern crate` 없이 어디서나 이름으로 씁니다.
// 매크로도 다른 항목처럼 `use`로 가져옵니다(report.rs의 `use scorekit::bar;`).

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
