// 기대하는 에러: E0382 — borrow of moved value: `scores`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0382_use_guard_after_drop.rs --out-dir /tmp/er-ch11
//
// drop(scores)는 잠금 가드를 drop 함수로 옮겨 그 자리에서 잠금을 풉니다. 그 뒤에 가드로 순위표를
// 고치려 하면, 잠그지 않은 채 공유 값을 고치는 셈입니다. 수동으로 unlock하는 언어에서는
// 컴파일되는 실수이지만, Rust에서는 가드가 이미 옮겨졌으므로 E0382로 막힙니다.
// 해결: 잠금 안에서 할 일을 drop(scores) 앞으로 옮깁니다. 정말 다시 고쳐야 한다면 lock()으로
//       새 가드를 얻습니다(examples/02_narrow_guard_scope.rs의 3번 구역).

use std::sync::Mutex;

fn main() {
    let board = Mutex::new(vec![("민지", 900), ("서준", 750)]);
    let mut scores = board.lock().unwrap();
    scores.push(("도윤", 800));
    drop(scores); // 잠금 해제
    let announcement = format!("참가자 {}명", board.lock().unwrap().len());
    scores.push(("하린", 600)); // E0382
    println!("{announcement}");
}
