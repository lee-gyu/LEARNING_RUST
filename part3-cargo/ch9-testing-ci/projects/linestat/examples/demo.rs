//! 라이브러리 사용 예. `cargo test`는 예제가 컴파일되는지도 확인합니다.
//!
//! 실행: cargo run -q --example demo

fn main() {
    let s = linestat::count("예제 한 줄\n");
    println!("{s:?}");
}
