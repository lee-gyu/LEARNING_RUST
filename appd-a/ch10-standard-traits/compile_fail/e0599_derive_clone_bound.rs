// 기대하는 에러: E0599 — the method `clone` exists for struct `Shared<Connection>`,
//                but its trait bounds were not satisfied
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0599_derive_clone_bound.rs --out-dir /tmp/er-ch10
//
// derive(Clone)는 impl<T: Clone> Clone for Shared<T>를 만듭니다. 복제하는 것은 Rc뿐이라
// T가 Clone일 필요가 없는데도 바운드가 붙어서, Clone이 아닌 Connection을 담으면 clone()이 사라집니다.
// 주의: 컴파일러의 마지막 `help:`는 Connection에 derive(Clone)를 붙이라고 제안합니다.
//       연결은 하나뿐이어야 해서 일부러 Clone을 빼 둔 타입이므로 따르면 안 됩니다.
// 해결: 바로 위의 `help:`(consider manually implementing the trait)대로 Clone을 손으로 씁니다.
//       impl<T> Clone for Shared<T> { fn clone(&self) -> Self { Shared(Rc::clone(&self.0)) } }
//       (examples/01_derive_matches_meaning.rs의 권장 판)

use std::rc::Rc;

#[derive(Debug)]
struct Connection {
    id: u32,
}

#[derive(Debug, Clone)]
struct Shared<T>(Rc<T>);

fn main() {
    let conn = Shared(Rc::new(Connection { id: 7 }));
    let other = conn.clone(); // E0599
    println!("{} {}", conn.0.id, Rc::strong_count(&other.0));
}
