// 이 예제가 보여 주는 것: BTreeMap/BTreeSet — 키가 항상 정렬되어 있는 맵과 집합.
//                        정렬된 순회, 범위 조회(range), 최소·최대 키, "x 이하에서 가장 큰 키" 찾기,
//                        derive(Ord)로 직접 만든 타입을 키로 쓰기
// 실행: cargo run -p p1-ch4-collections --example 06_btreemap
//
// "이것이 아니면 곤란한 상황": 키의 순서가 의미 있을 때입니다. HashMap은 순서가 없어서
// "11월 10일부터 20일까지", "가장 이른 날짜", "80점 이하 중 가장 큰 기준점" 같은 질문에
// 답하려면 모든 항목을 꺼내 정렬하거나 다 훑어야 합니다. BTreeMap은 O(log n)에 찾아갑니다.

use std::collections::{BTreeMap, BTreeSet};

// 직접 만든 타입을 BTreeMap의 키로 쓰려면 Ord가 필요합니다.
// derive(PartialOrd, Ord)는 필드를 "선언한 순서대로" 비교합니다. major가 같을 때만 minor를,
// minor도 같을 때만 patch를 비교하므로 버전 번호의 순서와 정확히 맞습니다.
// Ord는 Eq를, Eq는 PartialEq를, PartialOrd는 PartialEq를 전제로 하므로 넷을 함께 derive합니다.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

fn v(major: u32, minor: u32, patch: u32) -> Version {
    Version {
        major,
        minor,
        patch,
    }
}

fn main() {
    // 1. 넣은 순서와 상관없이 키 순서로 순회합니다.
    println!("== 1. 정렬된 순회 ==");
    let mut ledger: BTreeMap<&str, u32> = BTreeMap::new();
    ledger.insert("2026-11-15", 12000);
    ledger.insert("2026-11-03", 4500);
    ledger.insert("2026-11-28", 30000);
    ledger.insert("2026-11-10", 8000);
    ledger.insert("2026-11-21", 5500);
    // 날짜를 "연-월-일" 문자열로 적으면 문자열 순서와 날짜 순서가 같아집니다.
    for (date, amount) in &ledger {
        println!("{date}: {amount}원");
    }
    // 출력:
    // 2026-11-03: 4500원
    // 2026-11-10: 8000원
    // 2026-11-15: 12000원
    // 2026-11-21: 5500원
    // 2026-11-28: 30000원

    // 2. range: 키의 범위로 잘라 봅니다. 범위 문법(a..b, a..=b, ..b, a..)을 그대로 씁니다.
    println!("\n== 2. range로 범위 조회 ==");
    let mut total = 0;
    for (date, amount) in ledger.range("2026-11-10"..="2026-11-21") {
        println!("{date}: {amount}원");
        total += amount;
    }
    println!("11/10 ~ 11/21 합계: {total}원");
    // 출력:
    // 2026-11-10: 8000원
    // 2026-11-15: 12000원
    // 2026-11-21: 5500원
    // 11/10 ~ 11/21 합계: 25500원

    // 3. 가장 작은 키와 가장 큰 키
    println!("\n== 3. first_key_value / last_key_value / pop_first ==");
    println!("가장 이른 기록: {:?}", ledger.first_key_value()); // 출력: 가장 이른 기록: Some(("2026-11-03", 4500))
    println!("가장 늦은 기록: {:?}", ledger.last_key_value()); // 출력: 가장 늦은 기록: Some(("2026-11-28", 30000))
    // pop_first는 가장 작은 키의 항목을 꺼냅니다. 마감이 가장 이른 일부터 처리하는 식으로 쓸 수 있습니다.
    let mut deadlines = BTreeMap::from([(20, "보고서"), (5, "세금 신고"), (12, "발표 준비")]);
    while let Some((day, task)) = deadlines.pop_first() {
        println!("{day}일 마감: {task}");
    }
    // 출력:
    // 5일 마감: 세금 신고
    // 12일 마감: 발표 준비
    // 20일 마감: 보고서

    // 4. "x 이하에서 가장 큰 키"와 "x 이상에서 가장 작은 키"
    // range(..=x)는 x 이하의 항목을 오름차순으로 내놓으므로, 그 마지막 것이 답입니다.
    // next_back()은 반복자를 "뒤에서부터" 하나 꺼내는 메서드입니다(반복자는 ch5-iterators에서 자세히 다룹니다).
    println!("\n== 4. 기준점 찾기 ==");
    let cutoffs = BTreeMap::from([(0, 'F'), (60, 'D'), (70, 'C'), (80, 'B'), (90, 'A')]);
    for score in [95, 80, 79, 42] {
        let grade = cutoffs.range(..=score).next_back().map(|(_, &g)| g);
        println!("{score}점 → {grade:?}");
    }
    // 출력:
    // 95점 → Some('A')
    // 80점 → Some('B')
    // 79점 → Some('C')
    // 42점 → Some('F')

    // 집합도 같습니다. 버스 출발 시각(분 단위)에서 "지금 이후 첫 버스"를 찾습니다.
    let departures = BTreeSet::from([6 * 60 + 30, 7 * 60, 7 * 60 + 20, 8 * 60 + 5]);
    let now = 7 * 60 + 1; // 07:01
    match departures.range(now..).next() {
        Some(&t) => println!("다음 버스: {:02}:{:02}", t / 60, t % 60), // 출력: 다음 버스: 07:20
        None => println!("오늘 버스가 끝났습니다"),
    }

    // 5. 직접 만든 타입을 키로 쓰기: derive(Ord)는 필드 순서대로 비교합니다.
    println!("\n== 5. derive(Ord)한 타입을 키로 ==");
    let mut releases = BTreeMap::new();
    releases.insert(v(1, 10, 0), "검색 기능");
    releases.insert(v(2, 0, 0), "새 디자인");
    releases.insert(v(1, 2, 3), "버그 수정");
    releases.insert(v(1, 9, 1), "성능 개선");
    // 문자열 "1.10.0"과 "1.9.1"을 비교하면 "1.10.0"이 앞에 오지만, 숫자 필드로 비교하면 1.9.1이 앞입니다.
    for (ver, note) in releases.range(v(1, 5, 0)..v(2, 0, 0)) {
        println!("{}.{}.{}: {note}", ver.major, ver.minor, ver.patch);
    }
    // 출력:
    // 1.9.1: 성능 개선
    // 1.10.0: 검색 기능
    if let Some((latest, note)) = releases.last_key_value() {
        println!("최신: {latest:?} ({note})"); // 출력: 최신: Version { major: 2, minor: 0, patch: 0 } (새 디자인)
    }
}
