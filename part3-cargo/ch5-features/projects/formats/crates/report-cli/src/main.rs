//! 표를 CSV로 출력하는 프로그램입니다. textfmt의 `csv` 기능만 요청합니다.
//!
//! 실행: cargo run -q -p report-cli   (formats 디렉터리에서)

fn main() {
    let table = textfmt::Table::new(&["도시", "기온"], &[&["서울", "3"], &["부산", "9"]]);
    // 이 프로그램은 csv만 요청했지만, 실제로 켜진 기능은 함께 빌드한 패키지에 따라 달라집니다.
    println!("textfmt에 켜진 기능: {:?}", textfmt::enabled_formats());
    println!("{}", textfmt::to_csv(&table));
}
