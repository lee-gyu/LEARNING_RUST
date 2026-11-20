//! `src/bin/`의 파일이라 표를 적지 않아도 바이너리 `km2mi`가 됩니다.
//! `[[bin]]` 표를 하나 적었다고 해서 관례로 찾던 타깃이 사라지지는 않습니다.
//!
//! 실행: cargo run --bin km2mi -- 10

fn main() -> Result<(), String> {
    let km = units::number_or(std::env::args().nth(1), 42.195)?;
    println!("{km} km = {:.1} mi", units::km_to_mi(km));
    Ok(())
}
