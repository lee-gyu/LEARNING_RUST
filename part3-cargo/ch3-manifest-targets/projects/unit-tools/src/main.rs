//! `src/main.rs`는 패키지 이름과 같은 바이너리 `unit-tools`가 됩니다.
//! `Cargo.toml`의 `default-run`이 이 바이너리를 가리키므로 `cargo run`만으로 실행됩니다.
//!
//! 실행: cargo run

fn main() {
    println!("unit-tools {}", env!("CARGO_PKG_VERSION"));
    println!("  c2f [섭씨]    섭씨를 화씨로   (cargo run --bin c2f -- 36.5)");
    println!("  km2mi [km]    킬로미터를 마일로 (cargo run --bin km2mi -- 10)");
    println!("물이 끓는 온도: {}°F", units::c_to_f(100.0));
}
