// 기대하는 에러: E0005 — refutable pattern in local binding
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0005_refutable_pattern_in_let.rs
//
// 일반 let의 패턴은 어떤 값이 와도 반드시 맞아야 합니다(반박 불가능한 패턴).
// Some(port)는 값이 None이면 맞지 않는 패턴(반박 가능한 패턴)이라서,
// None일 때 port에 무엇을 넣어야 할지 정해지지 않습니다. 그래서 컴파일러가 거부합니다.

fn find_port(config: &str) -> Option<u16> {
    if config.contains("port") {
        Some(8080)
    } else {
        None
    }
}

fn main() {
    let Some(port) = find_port("host=localhost");
    println!("포트: {port}");
}
