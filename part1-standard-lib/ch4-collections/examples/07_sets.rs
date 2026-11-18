// 이 예제가 보여 주는 것: HashSet/BTreeSet — 값이 "있는지 없는지"만 기억하는 집합.
//                        insert의 bool 반환값으로 처음 본 값 가려내기, 순서를 지키며 중복 없애기,
//                        합집합·교집합·차집합·대칭 차집합, 부분집합 판정,
//                        derive(PartialEq, Eq, Hash)로 직접 만든 타입을 원소로 쓰기
// 실행: cargo run -p p1-ch4-collections --example 07_sets
//
// "이것이 아니면 곤란한 상황": "이 값을 본 적이 있나?"를 많이 묻거나, 두 모음의 공통·차이를 구할 때입니다.
// Vec의 contains는 O(n)이라 n번 물으면 O(n²)이 되지만, HashSet의 contains는 평균 O(1)입니다.
// HashSet의 순회 순서는 실행마다 달라지므로, 이 예제는 HashSet의 결과를 정렬하거나
// BTreeSet(항상 정렬된 순서)에 모아 출력합니다.

use std::collections::{BTreeSet, HashSet};

// 격자 위의 칸. HashSet의 원소가 되려면 Eq와 Hash가 필요합니다.
// derive(Hash)는 모든 필드를 해시하므로, 같은(==) 두 값은 해시도 같다는 규칙이 저절로 지켜집니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cell {
    x: i32,
    y: i32,
}

fn main() {
    // 1. insert는 "새로 넣었는지"를 bool로 돌려줍니다. 이미 있으면 false이고 집합은 그대로입니다.
    println!("== 1. insert의 반환값으로 중복 찾기 ==");
    let logins = ["민수", "지영", "민수", "하늘", "지영", "민수"];
    let mut seen = HashSet::new();
    for name in logins {
        if !seen.insert(name) {
            println!("{name}: 이미 로그인한 사용자");
        }
    }
    println!("서로 다른 사용자 수: {}", seen.len());
    // 출력:
    // 민수: 이미 로그인한 사용자
    // 지영: 이미 로그인한 사용자
    // 민수: 이미 로그인한 사용자
    // 서로 다른 사용자 수: 3

    // 2. 처음 나온 순서를 지키며 중복 없애기
    //    sort + dedup은 모든 중복을 지우지만 원래 순서를 잃습니다.
    //    retain의 클로저에서 seen.insert를 부르면 "처음 보는 값만 남긴다"가 됩니다.
    println!("\n== 2. 순서를 지키며 중복 없애기 ==");
    let mut tags = vec!["rust", "web", "rust", "cli", "web", "async"];
    let mut sorted = tags.clone();
    sorted.sort();
    sorted.dedup();
    let mut seen = HashSet::new();
    tags.retain(|t| seen.insert(*t)); // 처음이면 insert가 true → 남김, 두 번째부터는 false → 지움
    println!("sort + dedup: {sorted:?}"); // 출력: sort + dedup: ["async", "cli", "rust", "web"]
    println!("retain + set: {tags:?}"); // 출력: retain + set: ["rust", "web", "cli", "async"]

    // 3. 집합 연산 (BTreeSet: 결과가 정렬된 순서로 나옵니다)
    println!("\n== 3. 집합 연산 (BTreeSet) ==");
    let band = BTreeSet::from(["가온", "나래", "다솜", "라온"]);
    let choir = BTreeSet::from(["다솜", "라온", "마루", "바다"]);
    // 메서드는 결과를 하나씩 내놓는 반복자를 돌려줍니다. 새 집합이 필요하면 collect로 모읍니다.
    let both: BTreeSet<&str> = band.intersection(&choir).copied().collect();
    println!("교집합 (둘 다): {both:?}"); // 출력: 교집합 (둘 다): {"다솜", "라온"}
    println!("합집합 (어느 쪽이든): {:?}", &band | &choir); // 출력: 합집합 (어느 쪽이든): {"가온", "나래", "다솜", "라온", "마루", "바다"}
    println!("차집합 (밴드만): {:?}", &band - &choir); // 출력: 차집합 (밴드만): {"가온", "나래"}
    println!("대칭 차집합 (한쪽만): {:?}", &band ^ &choir); // 출력: 대칭 차집합 (한쪽만): {"가온", "나래", "마루", "바다"}
    // 연산자 &, |, -, ^는 두 집합의 참조에 쓰며, 새 집합을 만들어 돌려줍니다. 원래 집합은 그대로입니다.

    // 4. HashSet도 같은 연산을 제공합니다. 결과의 순서는 정해져 있지 않으므로 정렬해서 출력합니다.
    println!("\n== 4. 집합 연산 (HashSet) ==");
    let mine: HashSet<u32> = [3, 7, 12, 25, 31, 40].into_iter().collect();
    let drawn: HashSet<u32> = [7, 9, 25, 33, 40, 41].into_iter().collect();
    let mut hits: Vec<u32> = mine.intersection(&drawn).copied().collect();
    hits.sort();
    println!("맞힌 번호: {hits:?}"); // 출력: 맞힌 번호: [7, 25, 40]
    println!("12가 들어 있나? {}", mine.contains(&12)); // 출력: 12가 들어 있나? true

    // 5. 부분집합·상위집합·서로소
    println!("\n== 5. is_subset / is_superset / is_disjoint ==");
    let required = BTreeSet::from(["자료구조", "운영체제"]);
    let taken = BTreeSet::from(["자료구조", "운영체제", "컴파일러"]);
    let other = BTreeSet::from(["미술사", "음악 이론"]);
    println!("필수 과목을 모두 들었나? {}", required.is_subset(&taken)); // 출력: 필수 과목을 모두 들었나? true
    let covers = taken.is_superset(&required);
    println!("taken이 required를 포함하나? {covers}"); // 출력: taken이 required를 포함하나? true
    println!("겹치는 과목이 없나? {}", taken.is_disjoint(&other)); // 출력: 겹치는 과목이 없나? true

    // 6. 직접 만든 타입을 원소로: 로봇이 지나간 서로 다른 칸의 수
    println!("\n== 6. derive(Hash, Eq)한 타입을 원소로 ==");
    let moves = "RRUULLDDRRR"; // R: 오른쪽, L: 왼쪽, U: 위, D: 아래
    let mut pos = Cell { x: 0, y: 0 };
    let mut visited = HashSet::from([pos]);
    for m in moves.chars() {
        match m {
            'R' => pos.x += 1,
            'L' => pos.x -= 1,
            'U' => pos.y += 1,
            'D' => pos.y -= 1,
            _ => {}
        }
        visited.insert(pos); // Cell은 Copy라서 복사본이 집합에 들어갑니다.
    }
    println!("이동 {}번, 서로 다른 칸 {}개", moves.len(), visited.len()); // 출력: 이동 11번, 서로 다른 칸 9개
    let target = Cell { x: 1, y: 2 };
    println!("{target:?}에 갔었나? {}", visited.contains(&target)); // 출력: Cell { x: 1, y: 2 }에 갔었나? true
}
