//! 표를 JSON과 마크다운으로 출력하는 프로그램입니다.
//!
//! 실행: cargo run -q -p report-web   (formats 디렉터리에서)

fn main() {
    let table = textfmt::Table::new(&["도시", "기온"], &[&["서울", "3"], &["부산", "9"]]);
    println!("textfmt에 켜진 기능: {:?}", textfmt::enabled_formats());
    println!("{}", textfmt::to_json(&table));
    println!("{}", textfmt::to_markdown(&table));
}
