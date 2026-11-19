//! report와 invoice를 함께 쓰는 애플리케이션. 더하기만 하는 기능 덕분에 빌드되고 동작합니다.
//!
//!   cargo run -p app

fn main() {
    let today = report::Date {
        year: 2026,
        month: 11,
        day: 19,
    };
    println!("{}", report::title(&today)); // 출력: [주간 보고] 2026-11-19
    println!("{}", invoice::issued(&today)); // 출력: 발행일: 2026년 11월 19일 (목)
}
