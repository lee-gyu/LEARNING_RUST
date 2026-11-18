// 기대하는 에러: error[E0599]: no method named `sort` found for struct `VecDeque<T, A>` in the current scope
// 핵심 메시지: method not found in `VecDeque<i32>`
//
// sort는 슬라이스([T])의 메서드입니다. Vec은 슬라이스로 역참조되어 sort를 쓸 수 있지만,
// VecDeque는 링 버퍼라서 요소가 메모리에서 두 조각으로 나뉠 수 있으므로 슬라이스로 역참조되지 않습니다.
// 해결: line.make_contiguous().sort();  (examples/05_vecdeque.rs의 4번 구역)
//
// 확인: rustc --edition 2024 compile_fail/e0599_vecdeque_sort.rs --out-dir <임시 디렉터리>

use std::collections::VecDeque;

fn main() {
    let mut line: VecDeque<i32> = VecDeque::from([30, 10, 20]);
    line.push_front(40);
    line.sort();
    println!("{line:?}");
}
