// 기대하는 에러: error[E0499]: cannot borrow `tokens` as mutable more than once at a time
// 핵심 메시지: first mutable borrow occurs here / second mutable borrow occurs here / first borrow later used here
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0499_peek_then_next.rs --out-dir /tmp/rust-p1-ch5
//
// peek의 시그니처는 pub fn peek(&mut self) -> Option<&I::Item> 입니다. 다음 원소를 미리 꺼내 Peekable 안에 넣어 두고,
// 그 원소를 가리키는 참조를 돌려주므로 peekable 전체를 가변으로 빌립니다. 그 참조(next_char)를 쥔 채 next를 부르면,
// next가 바로 그 원소를 Peekable 밖으로 옮겨 버려 참조가 가리킬 곳이 사라집니다. 대여 검사기가 이것을 막습니다.
// 고치는 방법:
//   1) peek의 결과를 복사해 참조를 끊습니다: let next_char = tokens.peek().copied();  (char는 Copy)
//   2) 판단을 먼저 끝내고 꺼냅니다: if tokens.peek() == Some(&'1') { tokens.next(); }
//   3) "조건이 맞으면 꺼내기"가 목적이면 next_if / next_if_eq를 씁니다(examples/05_stateful.rs).

fn main() {
    let mut tokens = "12+3".chars().peekable();
    let next_char = tokens.peek();
    let consumed = tokens.next();
    println!("미리 본 글자: {next_char:?}, 꺼낸 글자: {consumed:?}");
}
