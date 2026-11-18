// 이 예제가 보여 주는 것: 원소를 바꾸지 않고 "어디까지, 어떤 순서로, 무엇과 함께" 내놓을지 정하는 어댑터.
//                        자르기·건너뛰기(take, skip, step_by, take_while, skip_while, map_while)와
//                        합치기·순서 바꾸기(chain, zip, enumerate, rev, cycle)
// 실행: cargo run -p p1-ch5-iterators --example 04_take_combine
//
// filter는 끝까지 모든 원소를 검사하지만, take_while은 조건이 처음 false가 되는 순간 반복을 끝냅니다.
// "조건에 맞는 것만"과 "조건에 맞는 동안만"은 다른 질문이라서 어댑터도 다릅니다.

fn main() {
    println!("== 1. take / skip / step_by ==");
    let days = ["월", "화", "수", "목", "금", "토", "일"];
    let first3: Vec<&str> = days.iter().copied().take(3).collect();
    let weekend: Vec<&str> = days.iter().copied().skip(5).collect();
    let every_other: Vec<&str> = days.iter().copied().step_by(2).collect(); // 첫 원소부터 2칸씩
    println!("take(3):    {first3:?}");
    println!("skip(5):    {weekend:?}");
    println!("step_by(2): {every_other:?}");
    // 출력: take(3):    ["월", "화", "수"]
    // 출력: skip(5):    ["토", "일"]
    // 출력: step_by(2): ["월", "수", "금", "일"]

    println!("\n== 2. take_while / skip_while와 filter의 차이 ==");
    // 온도 기록. 처음에는 낮다가 오르고, 다시 낮아집니다.
    let temps = [18, 21, 24, 31, 33, 22, 19];
    let filtered: Vec<i32> = temps.iter().copied().filter(|t| *t < 25).collect();
    let until_hot: Vec<i32> = temps.iter().copied().take_while(|t| *t < 25).collect();
    let from_hot: Vec<i32> = temps.iter().copied().skip_while(|t| *t < 25).collect();
    println!("filter(< 25):     {filtered:?}");
    println!("take_while(< 25): {until_hot:?}");
    println!("skip_while(< 25): {from_hot:?}");
    // 출력: filter(< 25):     [18, 21, 24, 22, 19]
    // 출력: take_while(< 25): [18, 21, 24]
    // 출력: skip_while(< 25): [31, 33, 22, 19]
    // take_while은 31에서 멈췄으므로 뒤의 22, 19는 보지 않습니다. skip_while은 한 번 false가 나오면 그 뒤는 검사 없이 모두 내놓습니다.

    println!("\n== 3. map_while: 바꾸다가 실패하면 멈추기 ==");
    // map_while은 클로저가 Some(v)를 돌려주는 동안 v를 내놓고, 처음 None이 나오면 끝납니다.
    // filter_map이 None을 "건너뛰는" 것과 달리, map_while은 None에서 "멈춥니다".
    let header = ["3", "14", "15", "끝", "92"];
    let until_bad: Vec<u32> = header.iter().map_while(|s| s.parse().ok()).collect();
    let skip_bad: Vec<u32> = header.iter().filter_map(|s| s.parse().ok()).collect();
    println!("map_while:  {until_bad:?}");
    println!("filter_map: {skip_bad:?}");
    // 출력: map_while:  [3, 14, 15]
    // 출력: filter_map: [3, 14, 15, 92]

    println!("\n== 4. chain: 이어 붙이기 ==");
    // chain(other)의 other는 IntoIterator면 되고, 원소 타입(Item)만 같으면 됩니다.
    let morning = ["회의", "메일"];
    let afternoon = ["코드 리뷰"];
    let urgent: Option<&str> = Some("장애 대응");
    let today: Vec<&str> = morning
        .iter()
        .copied() // &&str → &str로 맞춰야 아래의 배열, Option과 원소 타입이 같아집니다.
        .chain(afternoon)
        .chain(urgent)
        .collect();
    println!("chain: {today:?}");
    // 출력: chain: ["회의", "메일", "코드 리뷰", "장애 대응"]
    // copied()를 빼면 원소 타입이 &&str과 &str로 달라 E0271입니다(compile_fail/e0271_chain_item_mismatch.rs).

    println!("\n== 5. zip / enumerate ==");
    let names = ["가람", "나래", "다온"];
    let scores = [88, 72, 95, 60]; // 하나 더 많습니다.
    // zip은 두 반복자에서 하나씩 꺼내 (왼쪽, 오른쪽) 튜플을 만들고, 어느 한쪽이 끝나면 끝납니다.
    let pairs: Vec<(&str, i32)> = names.iter().copied().zip(scores).collect();
    println!("zip:       {pairs:?}");
    // enumerate는 (0부터 센 순번, 원소)입니다. 1부터 세고 싶으면 (1..).zip(...)도 씁니다.
    let numbered: Vec<String> = names
        .iter()
        .enumerate()
        .map(|(i, name)| format!("{i}:{name}"))
        .collect();
    let from_one: Vec<String> = (1..)
        .zip(names)
        .map(|(no, name)| format!("{no}번 {name}"))
        .collect();
    println!("enumerate: {numbered:?}");
    println!("(1..).zip: {from_one:?}");
    // 출력: zip:       [("가람", 88), ("나래", 72), ("다온", 95)]
    // 출력: enumerate: ["0:가람", "1:나래", "2:다온"]
    // 출력: (1..).zip: ["1번 가람", "2번 나래", "3번 다온"]
    // (1..)은 끝이 없는 반복자지만 names가 끝나는 순간 zip도 끝납니다.

    // 이웃한 원소끼리 짝짓기: 같은 반복자를 한 칸 밀어서 zip합니다.
    let deltas: Vec<i32> = temps
        .iter()
        .zip(temps.iter().skip(1))
        .map(|(a, b)| b - a)
        .collect();
    println!("이웃한 차이: {deltas:?}");
    // 출력: 이웃한 차이: [3, 3, 7, 2, -11, -3]

    println!("\n== 6. rev / cycle ==");
    let countdown: Vec<i32> = (1..=5).rev().collect();
    println!("rev:   {countdown:?}");
    // cycle은 끝나면 처음부터 다시 내놓는 끝없는 반복자입니다. 반드시 take나 zip으로 길이를 정합니다.
    let tasks = ["설거지", "빨래", "청소", "장보기", "분리수거"];
    let duty: Vec<String> = ["가람", "나래"]
        .iter()
        .cycle()
        .zip(tasks)
        .map(|(who, task)| format!("{task}={who}"))
        .collect();
    println!("cycle: {duty:?}");
    // 출력: rev:   [5, 4, 3, 2, 1]
    // 출력: cycle: ["설거지=가람", "빨래=나래", "청소=가람", "장보기=나래", "분리수거=가람"]
}
