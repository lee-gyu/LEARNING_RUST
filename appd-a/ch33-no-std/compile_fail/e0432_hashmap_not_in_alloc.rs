// 기대하는 에러: E0432 unresolved import `alloc::collections::HashMap`
//   no `HashMap` in `collections`
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --crate-type lib --out-dir <임시 디렉터리> compile_fail/e0432_hashmap_not_in_alloc.rs
//
// `std::collections`의 컬렉션 가운데 `HashMap`과 `HashSet`만 alloc에 없습니다. 기본 해셔의 시드를
// 운영체제의 난수로 정하기 때문입니다. 컴파일러는 대안을 제안하지 않습니다. `BTreeMap`으로 바꾸면
// 컴파일됩니다(키에 `Hash + Eq` 대신 `Ord`가 필요, examples/03_hashmap_to_btreemap.rs).

#![no_std]

extern crate alloc;

use alloc::collections::HashMap;

/// 장치 번호별 경보 횟수를 셉니다.
pub fn tally(alarms: &[u16]) -> HashMap<u16, u32> {
    let mut counts = HashMap::new();
    for a in alarms {
        *counts.entry(*a).or_insert(0) += 1;
    }
    counts
}
