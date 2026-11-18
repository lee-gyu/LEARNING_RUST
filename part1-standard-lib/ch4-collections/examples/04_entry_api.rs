// 이 예제가 보여 주는 것: 맵의 entry API — "조회 후 삽입"을 한 번의 탐색으로 쓰는 방법.
//                        or_insert / or_insert_with / or_default / and_modify, Entry 열거형을 match하기,
//                        HashMap과 BTreeMap에 같은 API가 있다는 것
// 실행: cargo run -p p1-ch4-collections --example 04_entry_api
//
// HashMap의 순회 순서는 실행마다 달라지므로, 맵 전체를 출력할 때는 BTreeMap에 모아(collect)
// 키 순서로 출력합니다. BTreeMap은 키가 정렬된 순서로 순회됩니다(06 예제).

use std::collections::BTreeMap;
use std::collections::hash_map::{self, HashMap};

// 키가 있는지 먼저 물어보고, 없으면 0을 넣은 뒤, 다시 찾아서 1을 더하는 방식입니다.
// 동작은 올바르지만 같은 키를 여러 번 찾습니다(있으면 2번, 없으면 3번).
fn count_two_lookups(votes: &[&'static str]) -> HashMap<&'static str, u32> {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for &menu in votes {
        if !counts.contains_key(menu) {
            counts.insert(menu, 0);
        }
        *counts.get_mut(menu).unwrap() += 1; // 바로 위에서 넣었거나 이미 있으므로 unwrap은 실패하지 않습니다.
    }
    counts
}

// entry는 키를 한 번 찾고, 찾은 "자리"를 Entry 값으로 돌려줍니다. 그 자리에서 읽고, 넣고, 고칩니다.
fn count_with_entry(votes: &[&'static str]) -> HashMap<&'static str, u32> {
    let mut counts = HashMap::new();
    for &menu in votes {
        *counts.entry(menu).or_insert(0) += 1;
    }
    counts
}

// 기본값을 만드는 데 비용이 드는 상황을 흉내 냅니다. 불릴 때마다 출력합니다.
fn load_profile(name: &str) -> String {
    println!("  (load_profile(\"{name}\") 호출됨)");
    format!("{name}의 프로필")
}

// Entry를 직접 match하면 "있을 때"와 "없을 때"를 완전히 다르게 처리할 수 있습니다.
fn take_out(stock: &mut HashMap<&'static str, u32>, item: &'static str, qty: u32) {
    match stock.entry(item) {
        hash_map::Entry::Occupied(mut entry) => {
            let have = *entry.get();
            if have > qty {
                *entry.get_mut() -= qty;
                println!("{item} {qty}개 출고, 남은 수량 {}", entry.get());
            } else if have == qty {
                entry.remove(); // 수량이 0이 되면 항목 자체를 지웁니다. 키를 다시 찾지 않습니다.
                println!("{item} {qty}개 출고, 품절되어 목록에서 삭제");
            } else {
                println!("{item} 재고 부족: {have}개뿐입니다");
            }
        }
        hash_map::Entry::Vacant(entry) => {
            // VacantEntry에서는 넣는 것만 할 수 있습니다. 여기서는 넣지 않고 키만 확인합니다.
            println!("{}: 등록되지 않은 상품입니다", entry.key());
        }
    }
}

fn main() {
    let votes = ["김밥", "라면", "김밥", "떡볶이", "김밥", "라면", "순대"];

    // 1. 두 번 찾기와 entry는 결과가 같습니다. 차이는 탐색 횟수와 코드의 길이입니다.
    println!("== 1. contains_key + insert 대 entry ==");
    let a = count_two_lookups(&votes);
    let b = count_with_entry(&votes);
    let sorted: BTreeMap<_, _> = b.iter().collect(); // 출력 순서를 고정하려고 BTreeMap에 모읍니다.
    println!("결과가 같은가? {}", a == b); // 출력: 결과가 같은가? true
    println!("{sorted:?}"); // 출력: {"김밥": 3, "떡볶이": 1, "라면": 2, "순대": 1}

    // 2. or_insert(값)은 값을 먼저 만들어 넘기고, or_insert_with(클로저)는 비어 있을 때만 클로저를 부릅니다.
    println!("\n== 2. or_insert 대 or_insert_with ==");
    let mut profiles: HashMap<&str, String> = HashMap::new();
    profiles.insert("민수", String::from("민수의 프로필"));
    println!("or_insert, 이미 있는 키:");
    // 인자는 메서드를 부르기 전에 계산되므로, 키가 있어서 쓰이지 않을 값도 매번 만들어집니다.
    profiles.entry("민수").or_insert(load_profile("민수"));
    // 출력:
    //   (load_profile("민수") 호출됨)
    println!("or_insert_with, 이미 있는 키:");
    profiles
        .entry("민수")
        .or_insert_with(|| load_profile("민수"));
    // 출력: 없음 (키가 있으므로 클로저를 부르지 않습니다)
    println!("or_insert_with, 없는 키:");
    let p = profiles
        .entry("지영")
        .or_insert_with(|| load_profile("지영"));
    println!("  돌려받은 값: {p}");
    // 출력:
    //   (load_profile("지영") 호출됨)
    //   돌려받은 값: 지영의 프로필

    // 3. or_default: 값 타입의 기본값(Default)을 넣습니다. 0, 빈 String, 빈 Vec 등.
    //    BTreeMap에도 같은 entry API가 있습니다.
    println!("\n== 3. or_default로 묶기 (BTreeMap) ==");
    let members = [
        ("개발", "민수"),
        ("영업", "지영"),
        ("개발", "아라"),
        ("디자인", "하늘"),
        ("영업", "철수"),
    ];
    let mut teams: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (team, name) in members {
        // 처음 보는 팀이면 빈 Vec을 넣고, 어느 경우든 그 Vec의 &mut를 돌려받아 바로 push합니다.
        teams.entry(team).or_default().push(name);
    }
    for (team, names) in &teams {
        println!("{team}: {names:?}");
    }
    // 출력:
    // 개발: ["민수", "아라"]
    // 디자인: ["하늘"]
    // 영업: ["지영", "철수"]

    // 4. and_modify: 키가 있을 때만 값을 고치고, 없으면 뒤의 or_insert가 넣습니다.
    println!("\n== 4. and_modify + or_insert로 최고 기록 갱신 ==");
    let attempts = [("민수", 72), ("지영", 85), ("민수", 90), ("지영", 80)];
    let mut best: BTreeMap<&str, u32> = BTreeMap::new();
    for (name, score) in attempts {
        best.entry(name)
            .and_modify(|b| *b = (*b).max(score)) // 있으면: 더 큰 값으로
            .or_insert(score); // 없으면: 첫 기록
    }
    println!("{best:?}"); // 출력: {"민수": 90, "지영": 85}

    // 5. Entry를 match하기: Occupied(있음)와 Vacant(없음)에서 할 수 있는 일이 다릅니다.
    println!("\n== 5. Entry 열거형을 match하기 ==");
    let mut stock: HashMap<&str, u32> = HashMap::from([("사과", 5), ("배", 2)]);
    take_out(&mut stock, "사과", 3);
    take_out(&mut stock, "배", 2);
    take_out(&mut stock, "사과", 4);
    take_out(&mut stock, "감", 1);
    // 출력:
    // 사과 3개 출고, 남은 수량 2
    // 배 2개 출고, 품절되어 목록에서 삭제
    // 사과 재고 부족: 2개뿐입니다
    // 감: 등록되지 않은 상품입니다
    let sorted: BTreeMap<_, _> = stock.iter().collect();
    println!("남은 재고: {sorted:?}"); // 출력: 남은 재고: {"사과": 2}
}
