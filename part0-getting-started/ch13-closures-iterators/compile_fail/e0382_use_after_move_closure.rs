// 기대하는 에러: error[E0382]: borrow of moved value: `list`
// 핵심 메시지: value moved into closure here / variable moved due to use in closure
//             value borrowed here after move
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0382_use_after_move_closure.rs --out-dir /tmp/rust-ch13
//
// 본문은 list를 읽기만 하지만 move를 붙였으므로, 클로저를 정의하는 순간 list의 소유권이 클로저 안으로
// 옮겨 갑니다. Vec은 Copy가 아니므로 원래 변수 list는 더 이상 쓸 수 없습니다(ch4의 이동과 같은 규칙).
// 고치는 방법: 클로저가 소유할 필요가 없다면 move를 빼서 빌리게 합니다. 클로저가 꼭 소유해야 한다면
// (함수 밖으로 돌려주는 경우 등) 컴파일러의 help:처럼 list.clone()한 복제본을 옮겨 넣습니다.

fn main() {
    let list = vec![1, 2, 3];

    let print_list = move || println!("클로저 안: {list:?}");
    print_list();

    println!("클로저 밖: {list:?}");
}
