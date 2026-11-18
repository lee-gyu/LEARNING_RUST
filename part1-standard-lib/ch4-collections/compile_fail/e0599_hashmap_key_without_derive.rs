// 기대하는 에러: error[E0599]: the method `insert` exists for struct `HashMap<Cell, u32>`,
//               but its trait bounds were not satisfied
// 핵심 메시지: doesn't satisfy `Cell: Eq` or `Cell: Hash`
//             help: consider annotating `Cell` with `#[derive(Eq, Hash, PartialEq)]`
//
// HashMap의 insert, get, entry 등은 `impl<K, V, S, A> HashMap<K, V, S, A> where K: Eq + Hash, ...`
// 블록 안에 있습니다. 키 타입이 Eq와 Hash를 구현하지 않으면 이 메서드들을 부를 수 없습니다.
// (HashMap::new는 바운드가 없는 블록에 있어서 만들기까지는 됩니다.)
// 해결: struct 위에 #[derive(PartialEq, Eq, Hash)]를 붙입니다.  (examples/07_sets.rs의 Cell)
//
// 확인: rustc --edition 2024 compile_fail/e0599_hashmap_key_without_derive.rs --out-dir <임시 디렉터리>

use std::collections::HashMap;

#[derive(Debug)]
struct Cell {
    x: i32,
    y: i32,
}

fn main() {
    let mut visits: HashMap<Cell, u32> = HashMap::new();
    visits.insert(Cell { x: 1, y: 2 }, 1);
    println!("{visits:?}");
}
