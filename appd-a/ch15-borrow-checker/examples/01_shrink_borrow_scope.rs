//! 지침 1: 대여는 쓰는 곳까지만 살게 한다.
//!
//! 여행 짐 목록에서 가장 무거운 짐을 찾아 둔 채로 짐을 하나 더 넣으려 하면 E0502입니다.
//! 이 에러를 피하려고 목록을 통째로 복제하는 판(비권장)과, 필요한 값만 꺼내 두거나
//! 순서를 바꿔 대여를 일찍 끝내는 판(권장)을 차례로 실행해 복제 횟수를 비교합니다.
//! 마지막 구역은 마지막 사용에서 끝나는 대여(NLL)도 아직 받아 주지 않는 모양,
//! "찾으면 그 참조를, 못 찾으면 새로 넣고 그 참조를" 돌려주는 함수를 인덱스로 풉니다.
//!
//! 실행: cargo run -p er-ch15-borrow-checker --example 01_shrink_borrow_scope

use std::sync::atomic::{AtomicUsize, Ordering};

/// `Gear`가 몇 번 복제됐는지 셉니다. 복제 비용을 눈으로 보려는 장치입니다.
static GEAR_CLONES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct Gear {
    name: String,
    grams: u32,
}

// derive 대신 손으로 구현해 복제될 때마다 수를 셉니다. 하는 일은 derive와 같습니다.
impl Clone for Gear {
    fn clone(&self) -> Self {
        GEAR_CLONES.fetch_add(1, Ordering::Relaxed);
        Gear {
            name: self.name.clone(),
            grams: self.grams,
        }
    }
}

fn gear(name: &str, grams: u32) -> Gear {
    Gear {
        name: name.to_string(),
        grams,
    }
}

fn packed_bag() -> Vec<Gear> {
    vec![
        gear("텐트", 2100),
        gear("침낭", 1300),
        gear("버너", 400),
        gear("물통", 900),
    ]
}

fn report(label: &str, heaviest: &str, grams: u32, bag: &[Gear]) {
    println!("{label}");
    println!(
        "  가장 무거운 짐: {heaviest}({grams}g), 우비를 넣은 뒤 {}개",
        bag.len()
    );
    println!(
        "  복제한 Gear: {}개",
        GEAR_CLONES.swap(0, Ordering::Relaxed)
    );
}

// 출발점(컴파일되지 않음, E0502):
//
//     let heaviest = bag.iter().max_by_key(|g| g.grams).unwrap(); // bag을 빌림
//     bag.push(gear("우비", 300));                                // bag을 고침
//     println!("{}", heaviest.name);                              // 빌린 것을 그 뒤에 씀
//
// `heaviest`는 `bag` 안의 원소를 가리키므로, `push`가 버퍼를 다시 할당하면
// 해제된 메모리를 가리키게 됩니다. 컴파일러가 막는 것은 바로 이 상황입니다.

/// 비권장: 대여를 끊으려고 목록 전체를 복제하고, 사본에서 찾습니다.
fn snapshot_whole_list(bag: &mut Vec<Gear>) {
    let snapshot = bag.clone(); // 짐 4개가 모두 복제됩니다.
    let heaviest = snapshot.iter().max_by_key(|g| g.grams).unwrap();
    bag.push(gear("우비", 300)); // bag은 빌려지지 않았으므로 고칠 수 있음
    report(
        "[비권장] 대여를 끊으려고 목록을 통째로 복제",
        &heaviest.name,
        heaviest.grams,
        bag,
    );
}

/// 권장 1: 나중에 필요한 것은 인덱스(Copy 값) 하나뿐입니다.
/// 인덱스를 꺼내는 순간 대여가 끝나고, 고친 뒤에 인덱스로 다시 빌립니다.
fn keep_only_the_index(bag: &mut Vec<Gear>) {
    let index = bag
        .iter()
        .enumerate()
        .max_by_key(|(_, g)| g.grams)
        .map(|(i, _)| i)
        .unwrap(); // usize. 이 줄에서 bag의 대여가 끝남
    bag.push(gear("우비", 300)); // push는 앞쪽 원소의 인덱스를 바꾸지 않음
    let heaviest = &bag[index]; // 고친 뒤에 다시 빌림
    report(
        "[권장 1] 인덱스만 꺼내 두었다가 고친 뒤 다시 빌리기",
        &heaviest.name,
        heaviest.grams,
        bag,
    );
}

/// 권장 2: 빌린 것을 다 쓰고 나서 고칩니다. 이름이 필요하면 그 하나만 복제합니다.
fn use_then_mutate(bag: &mut Vec<Gear>) {
    let heaviest = bag.iter().max_by_key(|g| g.grams).unwrap();
    let name = heaviest.name.clone(); // String 하나만 복제. Gear 복제는 0
    let grams = heaviest.grams; // heaviest의 마지막 사용 → 대여가 여기서 끝남
    bag.push(gear("우비", 300));
    report(
        "[권장 2] 쓸 값을 먼저 꺼내고 나서 고치기",
        &name,
        grams,
        bag,
    );
}

/// 파우치 하나. 같은 종류의 짐을 모아 둡니다.
struct Pouch {
    label: &'static str,
    items: Vec<&'static str>,
}

// NLL이 아직 받아 주지 않는 모양(1.100.0에서 E0499):
//
//     fn pouch_mut<'a>(pouches: &'a mut Vec<Pouch>, label: &'static str) -> &'a mut Pouch {
//         if let Some(p) = pouches.iter_mut().find(|p| p.label == label) {
//             return p; // 이 갈래에서 돌려준 대여가 'a 동안 이어진다고 보고,
//         }
//         pouches.push(Pouch { label, items: Vec::new() }); // 이 줄까지 살아 있다고 판단함
//         pouches.last_mut().unwrap()
//     }
//
// 사람이 보기에는 `return`하지 않은 갈래에서는 대여가 끝났지만, 지금의 대여 검사기는
// "돌려줄 수도 있는 대여"를 함수 끝까지 살아 있는 것으로 봅니다.

/// 권장: 찾기는 인덱스만 돌려주게 하고, 참조는 마지막에 한 번만 만듭니다.
fn pouch_mut<'a>(pouches: &'a mut Vec<Pouch>, label: &'static str) -> &'a mut Pouch {
    let index = match pouches.iter().position(|p| p.label == label) {
        Some(i) => i,
        None => {
            pouches.push(Pouch {
                label,
                items: Vec::new(),
            });
            pouches.len() - 1
        }
    };
    &mut pouches[index]
}

fn main() {
    let total: u32 = packed_bag().iter().map(|g| g.grams).sum();
    println!("짐 {}개, 총 {total}g", packed_bag().len());
    // 출력: 짐 4개, 총 4700g

    // 세 판의 결과는 같고 복제 횟수만 다릅니다.
    snapshot_whole_list(&mut packed_bag());
    // 출력: [비권장] 대여를 끊으려고 목록을 통째로 복제
    // 출력:   가장 무거운 짐: 텐트(2100g), 우비를 넣은 뒤 5개
    // 출력:   복제한 Gear: 4개
    keep_only_the_index(&mut packed_bag());
    // 출력: [권장 1] 인덱스만 꺼내 두었다가 고친 뒤 다시 빌리기
    // 출력:   가장 무거운 짐: 텐트(2100g), 우비를 넣은 뒤 5개
    // 출력:   복제한 Gear: 0개
    use_then_mutate(&mut packed_bag());
    // 출력: [권장 2] 쓸 값을 먼저 꺼내고 나서 고치기
    // 출력:   가장 무거운 짐: 텐트(2100g), 우비를 넣은 뒤 5개
    // 출력:   복제한 Gear: 0개

    println!("[NLL이 받지 않는 모양] 찾거나 새로 만들어 &mut를 돌려주는 함수");
    let mut pouches = vec![Pouch {
        label: "세면",
        items: vec!["칫솔", "치약"],
    }];
    pouch_mut(&mut pouches, "세면").items.push("수건"); // 있던 파우치
    pouch_mut(&mut pouches, "비상").items.push("밴드"); // 새로 만든 파우치
    for p in &pouches {
        println!("  {} 파우치: {:?}", p.label, p.items);
    }
    // 출력:   세면 파우치: ["칫솔", "치약", "수건"]
    // 출력:   비상 파우치: ["밴드"]
}
