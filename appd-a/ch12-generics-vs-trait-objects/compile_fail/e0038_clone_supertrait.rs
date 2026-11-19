// 기대하는 에러: E0038 — the trait `SnapshotItem` is not dyn compatible
// 핵심 메시지: ...because it requires `Self: Sized`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0038_clone_supertrait.rs --out-dir /tmp/er-ch12
//
// 조합 트레이트의 슈퍼트레이트에 `Clone`을 넣었습니다. `Clone`은 `Sized`를 슈퍼트레이트로 가지므로
// (`trait Clone: Sized`), `SnapshotItem`을 구현하는 타입은 모두 크기를 알아야 합니다.
// 크기를 모르는 `dyn SnapshotItem`은 그 조건을 만족할 수 없으니 트레이트 객체가 될 수 없습니다.
// 제네릭이라면 `T: Loanable + Clone`으로 문제없이 받을 수 있는 조합입니다
// (examples/03_combined_bounds.rs의 `snapshot`).
// 해결: 사본이 필요하면 제네릭으로 받습니다. 트레이트 객체로 사본을 만들어야 한다면
//       `Self` 대신 `Box<dyn Loanable>`을 돌려주는 메서드(`fn boxed_copy(&self) -> Box<dyn Loanable>`)를
//       트레이트에 두고 타입마다 구현합니다.

trait Loanable {
    fn loan_days(&self) -> u32;
}

trait SnapshotItem: Loanable + Clone {}

impl<T: Loanable + Clone> SnapshotItem for T {}

#[derive(Clone)]
struct Book;

impl Loanable for Book {
    fn loan_days(&self) -> u32 {
        14
    }
}

// E0038: `SnapshotItem`은 `Clone` 때문에 `Self: Sized`를 요구함
fn keep(item: &dyn SnapshotItem, history: &mut Vec<u32>) {
    history.push(item.loan_days());
}

fn main() {
    let mut history = Vec::new();
    keep(&Book, &mut history);
    println!("{history:?}");
}
