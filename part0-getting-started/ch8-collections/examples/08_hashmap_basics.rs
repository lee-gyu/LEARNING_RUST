// HashMap<K, V>로 키와 값을 짝지어 저장하고, 찾고, 순회하고, 갱신하는 기본 방법을 보여 줍니다.
// 값을 넣을 때 소유권이 어떻게 움직이는지, 같은 키에 다시 넣으면 어떻게 되는지도 확인합니다.
//
// 실행: cargo run -p p0-ch8-collections --example 08_hashmap_basics

// HashMap은 프렐류드(모든 파일에 자동으로 가져와지는 항목)에 없으므로 use로 직접 가져옵니다(ch7).
use std::collections::HashMap;

fn main() {
    println!("1. new로 만들고 insert로 (키, 값)을 넣습니다");
    // 청백전 점수판입니다. 키는 팀 이름(String), 값은 점수(i32)입니다.
    // 타입을 적지 않아도 아래 insert를 보고 HashMap<String, i32>로 추론합니다.
    let mut scores = HashMap::new();
    scores.insert(String::from("청팀"), 10);
    scores.insert(String::from("백팀"), 50);
    println!("    팀 수: {}", scores.len()); // 출력: 팀 수: 2

    println!("2. get은 키로 값을 찾아 Option<&V>로 돌려줍니다");
    // get에는 키의 참조를 넘깁니다. 키 타입이 String이어도 &str로 찾을 수 있습니다.
    for team in ["청팀", "홍팀"] {
        match scores.get(team) {
            Some(score) => println!("    {team}: {score}점"),
            None => println!("    {team}: 없는 팀입니다"),
        }
    }
    // 출력:
    //     청팀: 10점
    //     홍팀: 없는 팀입니다

    println!("3. for (키, 값) in &맵 형태로 순회합니다 (순서는 정해져 있지 않습니다)");
    // HashMap은 키를 해시값에 따라 배치하므로, 순회 순서가 넣은 순서와 다르고 실행할 때마다 달라질 수 있습니다.
    // 출력을 일정하게 보여 주기 위해 벡터에 모은 뒤 정렬해서 출력합니다.
    let mut lines = Vec::new();
    for (team, score) in &scores {
        // team은 &String, score는 &i32입니다. 맵을 빌려 순회하므로 맵은 그대로 남습니다.
        lines.push(format!("{team}={score}"));
    }
    lines.sort(); // sort()는 벡터의 요소를 오름차순으로 그 자리에서 정렬합니다.
    println!("    {lines:?}"); // 출력: ["백팀=50", "청팀=10"]

    println!("4. 같은 키로 다시 insert하면 값이 덮어써집니다");
    // insert는 그 키에 원래 있던 값을 Option으로 돌려줍니다. 처음 넣는 키였다면 None입니다.
    let old = scores.insert(String::from("청팀"), 25);
    println!("    원래 값: {old:?}, 새 값: {:?}", scores.get("청팀")); // 출력: 원래 값: Some(10), 새 값: Some(25)
    // 키 하나에는 값이 하나만 있으므로, 팀 수는 그대로 2입니다.
    println!("    팀 수: {}", scores.len()); // 출력: 팀 수: 2

    println!("5. entry().or_insert()는 키가 없을 때만 넣습니다");
    // entry(키)는 "그 키의 자리"를 나타내는 값을 돌려주고,
    // or_insert(기본값)은 자리가 비어 있을 때만 기본값을 넣습니다. 이미 있으면 기존 값을 그대로 둡니다.
    scores.entry(String::from("홍팀")).or_insert(30); // 홍팀은 없으므로 30이 들어갑니다.
    scores.entry(String::from("백팀")).or_insert(0); // 백팀은 이미 있으므로 50이 유지됩니다.
    let mut lines = Vec::new();
    for (team, score) in &scores {
        lines.push(format!("{team}={score}"));
    }
    lines.sort();
    println!("    {lines:?}"); // 출력: ["백팀=50", "청팀=25", "홍팀=30"]

    println!("6. insert하면 String 키와 값의 소유권이 맵으로 이동합니다");
    let field_name = String::from("좋아하는 색");
    let field_value = String::from("파랑");
    let mut profile = HashMap::new();
    profile.insert(field_name, field_value);
    // 이제 두 String의 소유자는 profile입니다. field_name, field_value를 쓰면 E0382입니다
    // (compile_fail/e0382_hashmap_insert_moves.rs). i32처럼 Copy인 값은 복사되어 들어갑니다.
    match profile.get("좋아하는 색") {
        Some(color) => println!("    좋아하는 색: {color}"), // 출력: 좋아하는 색: 파랑
        None => println!("    좋아하는 색: 모름"),
    }

    println!("7. 원래 String을 계속 써야 하면 clone한 사본이나 참조를 넣습니다");
    let city = String::from("부산");
    let mut copied = HashMap::new();
    copied.insert("민수", city.clone()); // 사본을 넣습니다. 맵은 사본의 소유자가 됩니다.
    let mut borrowed = HashMap::new();
    // 참조를 넣으면 맵은 city를 빌리기만 합니다. 대신 맵이 쓰이는 동안 city가 살아 있어야 합니다
    // (이 조건을 컴파일러가 확인하는 방법은 ch10의 라이프타임에서 배웁니다).
    borrowed.insert("지영", &city);
    println!("    city = {city}"); // 출력: city = 부산
    println!(
        "    민수: {:?}, 지영: {:?}",
        copied.get("민수"),
        borrowed.get("지영")
    );
    // 출력: 민수: Some("부산"), 지영: Some("부산")
}
