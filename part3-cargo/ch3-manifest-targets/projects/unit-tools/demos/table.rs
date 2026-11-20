//! `Cargo.toml`의 `[[example]]` 표가 선언한 예제입니다(`examples/` 밖의 경로).
//!
//! 실행: cargo run --example table

fn main() {
    println!("섭씨   화씨");
    for celsius in [-40.0, 0.0, 36.5, 100.0] {
        println!("{celsius:>5} {:>6.1}", units::c_to_f(celsius));
    }
}
