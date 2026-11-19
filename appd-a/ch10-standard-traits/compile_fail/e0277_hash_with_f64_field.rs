// 기대하는 에러: E0277 — the trait bound `f64: Eq` is not satisfied
//                E0277 — the trait bound `f64: Hash` is not satisfied
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_hash_with_f64_field.rs --out-dir /tmp/er-ch10
//
// derive는 필드마다 같은 트레이트를 요구합니다. f64는 NaN 때문에 Eq(자기 자신과 같다는 약속)를
// 지킬 수 없고, 그래서 Hash도 구현하지 않습니다. f64 필드가 하나라도 있으면 바깥 타입도
// Eq와 Hash를 derive할 수 없습니다.
// 해결: 비교만 필요하면 PartialEq까지만 derive합니다. 집합이나 맵의 키가 필요하면
//       정수 표현으로 바꾼 타입을 따로 둡니다(examples/04_float_fields.rs의 PinKey).

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pin {
    lat: f64,
    lng: f64,
}

fn main() {
    let mut seen = HashSet::new();
    seen.insert(Pin {
        lat: 37.5663,
        lng: 126.9779,
    });
    println!("{}", seen.len());
}
