// 기대하는 에러: E0308 (두 개) — mismatched types
//   1) expected `u32`, found `u16`  (넓히는 변환: 값이 반드시 들어가지만 그래도 자동으로 바꾸지 않음)
//   2) expected `u16`, found `i64`  (좁히는 변환: 값이 들어가지 않을 수 있음)
//
// 확인: rustc --edition 2024 compile_fail/e0308_no_implicit_int_conversion.rs
//
// Rust는 정수 타입 사이에서 어떤 변환도 몰래 하지 않습니다. 넓히는 쪽이든 좁히는 쪽이든
// 변환하는 지점을 코드에 적어야 합니다. help:가 두 경우에 서로 다른 해결책을 제안하는 것도 눈여겨보세요.

fn address(port: u16) -> String {
    format!("127.0.0.1:{port}")
}

// 0~65535 범위의 포트는 65536개라서 u16에 담을 수 없습니다. 그래서 반환 타입을 u32로 정했습니다.
fn ports_in_range(first: u16, last: u16) -> u32 {
    last - first + 1 // E0308: u16 값을 u32 자리에 돌려줌
}

fn main() {
    let configured: i64 = 8080; // 설정에서 "넉넉하게" i64로 읽은 값
    println!("{}", address(configured)); // E0308: i64 값을 u16 매개변수에 넘김
    println!("{}", ports_in_range(1024, 2047));
}
