//! `Cargo.toml`의 요구 사항 `parcel = "..."`에 따라 Cargo가 고른 parcel 버전을 출력합니다.
//!
//! 실행(이 디렉터리에서): cargo run
//!
//! 이 코드는 parcel 0.x와 1.x의 `fee(u32)`를 씁니다. 2.0.0에서는 `fee`가 `Grams`를 받으므로,
//! 요구 사항이 2.0.0을 허용하면(예: "*") 이 파일이 E0308로 컴파일되지 않습니다.

fn main() {
    println!("Cargo가 고른 parcel: {}", parcel::VERSION);
    println!("1200g 배송비: {}원", parcel::fee(1200));
}
