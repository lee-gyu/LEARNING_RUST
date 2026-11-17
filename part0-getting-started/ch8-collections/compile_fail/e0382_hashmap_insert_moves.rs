// 기대하는 에러: E0382 — borrow of moved value: `field_name`
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0382_hashmap_insert_moves.rs
//
// insert에 String을 넘기면 소유권이 맵으로 이동합니다. 이후의 소유자는 map이므로
// 원래 변수 field_name으로는 더 이상 그 값을 쓸 수 없습니다.

use std::collections::HashMap;

fn main() {
    let field_name = String::from("좋아하는 색");
    let field_value = String::from("파랑");

    let mut map = HashMap::new();
    map.insert(field_name, field_value);

    println!("{field_name}: {:?}", map.get("좋아하는 색"));
}
