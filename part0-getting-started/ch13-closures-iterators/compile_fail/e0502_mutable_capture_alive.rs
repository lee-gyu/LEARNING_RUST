// 기대하는 에러: error[E0502]: cannot borrow `list` as immutable because it is also borrowed as mutable
// 핵심 메시지: mutable borrow occurs here / first borrow occurs due to use of `list` in closure
//             immutable borrow occurs here / mutable borrow later used here
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0502_mutable_capture_alive.rs --out-dir /tmp/rust-ch13
//
// borrows_mutably는 list.push(7)을 하므로 list의 가변 참조를 캡처합니다. 클로저를 정의한 순간부터
// 마지막으로 호출하는 줄까지 그 가변 대여가 살아 있습니다. 그 사이에 println!으로 list를 읽으려면
// 불변 대여가 필요한데, 가변 대여가 있는 동안에는 다른 대여가 있을 수 없습니다(ch4의 대여 규칙).
// 고치는 방법: 클로저의 마지막 호출 뒤로 println!을 옮기거나, 클로저를 쓰기 전에 출력합니다.

fn main() {
    let mut list = vec![1, 2, 3];

    let mut borrows_mutably = || list.push(7);

    println!("호출 전: {list:?}");
    borrows_mutably();
    println!("호출 후: {list:?}");
}
