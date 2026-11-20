//! `examples/`의 파일이라 표를 적지 않아도 예제 `quick`이 됩니다.
//!
//! 실행: cargo run --example quick

fn main() {
    println!("10 km = {:.2} mi", units::km_to_mi(10.0));
}
