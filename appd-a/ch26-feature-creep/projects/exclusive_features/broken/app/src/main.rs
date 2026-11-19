//! report와 invoice를 함께 쓰는 애플리케이션. 이 워크스페이스에서는 빌드되지 않습니다.
//!
//!   cargo build -p app
//!
//! 이 파일에는 잘못된 곳이 없습니다. 에러는 datefmt 안에서 납니다.

fn main() {
    let today = report::Date {
        year: 2026,
        month: 11,
        day: 19,
    };
    println!("{}", report::title(&today));
    println!("{}", invoice::issued(&today));
}
