//! `HashMap`은 std에만 있습니다. `no_std` 라이브러리에서는 alloc의 `BTreeMap`으로 바꿉니다.
//! 함수 본문은 거의 그대로이고, 키에 요구하는 트레이트(`Hash + Eq` → `Ord`)와 순회 순서가 달라집니다.
//!
//! 실행: cargo run -p er-ch33-no-std --example 03_hashmap_to_btreemap

extern crate alloc;

/// 경보를 보낸 장치. `HashMap`의 키가 되려면 `Hash + Eq`, `BTreeMap`의 키가 되려면 `Ord`가 필요해서
/// 둘 다 derive했습니다. derive한 `Ord`는 필드를 선언 순서(rack, slot)로 비교합니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Device {
    rack: u8,
    slot: u8,
}

const fn dev(rack: u8, slot: u8) -> Device {
    Device { rack, slot }
}

/// 하룻밤 동안 들어온 경보(보낸 장치만 남김)
const ALARMS: [Device; 6] = [
    dev(1, 2),
    dev(2, 1),
    dev(1, 2),
    dev(3, 4),
    dev(2, 1),
    dev(1, 2),
];

/// std판: 해시 맵으로 셉니다.
mod with_hashmap {
    use super::Device;
    use std::collections::HashMap;

    pub fn tally(alarms: &[Device]) -> HashMap<Device, u32> {
        let mut counts = HashMap::new();
        for d in alarms {
            *counts.entry(*d).or_insert(0) += 1;
        }
        counts
    }
}

/// `no_std`판: 경로와 타입 이름만 바꿨습니다. `entry`·`or_insert`는 그대로 있습니다.
mod with_btreemap {
    use super::Device;
    use alloc::collections::BTreeMap;

    pub fn tally(alarms: &[Device]) -> BTreeMap<Device, u32> {
        let mut counts = BTreeMap::new();
        for d in alarms {
            *counts.entry(*d).or_insert(0) += 1;
        }
        counts
    }
}

fn main() {
    let hashed = with_hashmap::tally(&ALARMS);
    let ordered = with_btreemap::tally(&ALARMS);

    // ── 1. 셈의 결과는 같습니다 ───────────────────────────────────────────
    // HashMap의 순서는 정해져 있지 않으므로 정렬한 뒤에 비교합니다.
    let mut hashed_sorted: Vec<(Device, u32)> = hashed.into_iter().collect();
    hashed_sorted.sort();
    let ordered_vec: Vec<(Device, u32)> = ordered.iter().map(|(d, n)| (*d, *n)).collect();
    println!(
        "1. 정렬한 HashMap판 == BTreeMap판: {}",
        hashed_sorted == ordered_vec
    ); // 출력: 1. 정렬한 HashMap판 == BTreeMap판: true

    // ── 2. BTreeMap은 키 순서로 돕니다 ────────────────────────────────────
    // 테스트의 기대 출력이나 보고서처럼 순서가 필요한 곳에서는 오히려 편합니다.
    println!("2. BTreeMap의 순회 순서(키 순서)");
    for (d, n) in &ordered {
        println!("  랙 {} 슬롯 {}: {n}번", d.rack, d.slot);
    }
    // 출력:   랙 1 슬롯 2: 3번
    // 출력:   랙 2 슬롯 1: 2번
    // 출력:   랙 3 슬롯 4: 1번

    // 순서가 있으니 범위로 찾을 수도 있습니다. 랙 2에 있는 장치만:
    let rack2: Vec<u32> = ordered
        .range(dev(2, 0)..dev(3, 0))
        .map(|(_, n)| *n)
        .collect();
    println!("  랙 2의 경보 수: {rack2:?}"); // 출력:   랙 2의 경보 수: [2]

    // ── 3. HashMap의 순서는 해셔의 시드에 달려 있습니다 ───────────────────────
    // `HashMap::new()`가 쓰는 `RandomState::new()`는 운영체제에서 받은 난수로 시드를 정합니다.
    // 시드가 다르면 같은 키도 다른 해시 값이 되어, 순회 순서가 실행마다 달라질 수 있습니다.
    // core와 alloc은 운영체제를 가정하지 않으므로 이 난수를 얻을 수 없고, 그래서 HashMap은 std에만 있습니다.
    use std::hash::{BuildHasher, RandomState};
    let (a, b) = (RandomState::new(), RandomState::new());
    // 해시 값 자체는 실행마다 다르므로 출력하지 않고 비교 결과만 출력합니다(64비트 해시가 우연히 같을
    // 확률은 무시할 만큼 작습니다).
    println!(
        "3. 시드가 다른 두 해셔가 같은 키에 같은 해시를 내는가: {}",
        a.hash_one(dev(1, 2)) == b.hash_one(dev(1, 2))
    ); // 출력: 3. 시드가 다른 두 해셔가 같은 키에 같은 해시를 내는가: false
}
