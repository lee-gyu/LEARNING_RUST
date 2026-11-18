// 기대하는 에러: error[E0369]: cannot add `&str` to `&str`
// 핵심 메시지:   string concatenation requires an owned `String` on the left
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0369_add_two_str.rs --out-dir /tmp/rust-p1-ch3
//
// + 연산자는 String에만 정의되어 있습니다(왼쪽 String의 버퍼 뒤에 오른쪽 &str을 덧붙임, Part 0 ch8).
// &str 두 개는 둘 다 빌린 문자열이라 결과를 담을 버퍼가 없습니다. 새 버퍼를 몰래 할당하는 대신
// Rust는 이 코드를 거부하고, 할당이 일어나는 곳을 코드에 드러내게 합니다.
// 고치는 방법: 왼쪽을 소유한 String으로 만들거나(greeting.to_owned() + name), format!을 씁니다.

fn main() {
    let greeting = "안녕하세요, ";
    let name = "페리스";
    let message = greeting + name;
    println!("{message}");
}
