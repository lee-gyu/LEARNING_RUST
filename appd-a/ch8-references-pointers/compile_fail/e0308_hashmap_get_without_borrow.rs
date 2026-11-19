// 기대 에러: E0308 mismatched types
//   expected `&StrictCode`, found `&str`
//
// `HashMap::get`은 `K: Borrow<Q>`인 `&Q`를 받습니다. `StrictCode`는 `Borrow<str>`를 구현하지
// 않았으므로 쓸 수 있는 `Q`는 반사 구현에 의한 `StrictCode` 자신뿐이고, 컴파일러는 인자 타입을
// `&StrictCode`로 정한 뒤 `&str`과 맞지 않는다고 알립니다. (E0277이 아니라 E0308입니다.)
// `Borrow`의 약속을 지킬 수 없는 키 타입이라면 이것이 원하는 결과입니다. 찾을 때 키 타입을 만듭니다.
//
// 확인: rustc --edition 2024 compile_fail/e0308_hashmap_get_without_borrow.rs --out-dir /tmp/er-ch8

use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash)]
struct StrictCode(String);

impl StrictCode {
    fn new(raw: &str) -> Self {
        StrictCode(raw.trim().to_ascii_uppercase())
    }
}

fn main() {
    let mut airports: HashMap<StrictCode, &str> = HashMap::new();
    airports.insert(StrictCode::new("icn"), "인천");
    println!("{:?}", airports.get(&StrictCode::new("icn"))); // 키 타입으로 찾으면 됨
    println!("{:?}", airports.get("ICN")); // E0308
}
