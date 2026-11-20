//! `Cargo.toml`의 `[[bin]]` 표가 이 파일을 바이너리 `c2f`로 선언합니다.
//! `src/bin/` 밖에 있으므로 관례만으로는 타깃이 되지 않습니다.
//!
//! 실행: cargo run --bin c2f -- 36.5

fn main() -> Result<(), String> {
    let celsius = units::number_or(std::env::args().nth(1), 100.0)?;
    println!("{celsius}°C = {:.1}°F", units::c_to_f(celsius));
    Ok(())
}
