// 이 예제가 보여 주는 것: 슬라이스 정렬 메서드(sort, sort_unstable, sort_by, sort_by_key,
//                        sort_by_cached_key)의 차이, 안정 정렬의 의미, 실수(f64) 정렬과 total_cmp
// 실행: cargo run -p p1-ch4-collections --example 02_sort
//
// 정렬 메서드는 Vec이 아니라 슬라이스 [T]에 정의되어 있습니다. Vec은 슬라이스로 역참조되므로
// v.sort()처럼 바로 부를 수 있고, 배열이나 &mut [T]에도 똑같이 쓸 수 있습니다.

use std::cmp::Reverse;

struct Student {
    name: &'static str,
    class: u8,
    score: u32,
}

// 예제 데이터를 짧게 적으려고 만든 함수입니다.
fn student(name: &'static str, class: u8, score: u32) -> Student {
    Student { name, class, score }
}

fn main() {
    // 1. sort와 sort_unstable: 정수처럼 "같으면 구별할 수 없는" 값은 결과가 똑같습니다.
    println!("== 1. sort / sort_unstable ==");
    let mut a = [42, 7, 19, 7, 3];
    let mut b = a;
    a.sort(); // 안정 정렬
    b.sort_unstable(); // 불안정 정렬: 보통 더 빠르고, 추가 메모리를 쓰지 않습니다.
    println!("sort: {a:?}, sort_unstable: {b:?}"); // 출력: sort: [3, 7, 7, 19, 42], sort_unstable: [3, 7, 7, 19, 42]

    // 2. 안정 정렬(stable sort)은 비교 결과가 같은 요소의 원래 순서를 지킵니다.
    // 이 성질 덕분에 "보조 기준으로 먼저 정렬하고, 주 기준으로 다시 정렬"하면 두 기준이 함께 적용됩니다.
    println!("\n== 2. 안정 정렬로 두 기준 적용하기 ==");
    let mut students = vec![
        student("하윤", 2, 88),
        student("민준", 1, 92),
        student("서연", 2, 75),
        student("도윤", 1, 92),
        student("지우", 1, 64),
    ];
    students.sort_by_key(|s| s.name); // 보조 기준: 이름 가나다순
    students.sort_by_key(|s| s.class); // 주 기준: 반. 같은 반 안에서는 이름순이 그대로 남습니다.
    for s in &students {
        println!("{}반 {} {}점", s.class, s.name, s.score);
    }
    // 출력:
    // 1반 도윤 92점
    // 1반 민준 92점
    // 1반 지우 64점
    // 2반 서연 75점
    // 2반 하윤 88점

    // 3. 불안정 정렬은 같은 요소의 순서를 바꿀 수 있습니다.
    // 주문 50건을 지역별로 정렬한 뒤, 같은 지역 안에서 주문 번호가 오름차순으로 남았는지 확인합니다.
    // 불안정 정렬의 결과는 이 툴체인에서의 결과입니다. 요소가 적으면 순서가 우연히 유지되기도 하고,
    // 구현이 바뀌면 달라질 수 있습니다. "보장하지 않는다"는 것이 핵심입니다.
    println!("\n== 3. 불안정 정렬은 같은 요소의 순서를 보장하지 않습니다 ==");
    let regions = ["서울", "부산", "대구"];
    let orders: Vec<(u32, &str)> = (1..=50).map(|id| (id, regions[id as usize % 3])).collect();
    let mut stable = orders.clone();
    stable.sort_by_key(|o| o.1);
    let mut unstable = orders.clone();
    unstable.sort_unstable_by_key(|o| o.1);
    for region in regions {
        let ids_s: Vec<u32> = stable
            .iter()
            .filter(|o| o.1 == region)
            .map(|o| o.0)
            .collect();
        let ids_u: Vec<u32> = unstable
            .iter()
            .filter(|o| o.1 == region)
            .map(|o| o.0)
            .collect();
        // is_sorted()는 슬라이스가 오름차순인지 확인합니다.
        println!(
            "{region}: 안정 {:?}.. 유지={} / 불안정 {:?}.. 유지={}",
            &ids_s[..4],
            ids_s.is_sorted(),
            &ids_u[..4],
            ids_u.is_sorted()
        );
    }
    // 출력:
    // 서울: 안정 [3, 6, 9, 12].. 유지=true / 불안정 [21, 9, 24, 27].. 유지=false
    // 부산: 안정 [1, 4, 7, 10].. 유지=true / 불안정 [25, 19, 4, 22].. 유지=false
    // 대구: 안정 [2, 5, 8, 11].. 유지=true / 불안정 [2, 5, 8, 11].. 유지=true

    // 4. 기준을 직접 정하기: sort_by는 두 요소를 비교하는 클로저를, sort_by_key는 키를 뽑는 클로저를 받습니다.
    println!("\n== 4. sort_by / sort_by_key와 Reverse ==");
    // 점수 내림차순, 점수가 같으면 이름 오름차순
    students.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(b.name)));
    let by_cmp: Vec<&str> = students.iter().map(|s| s.name).collect();
    // 같은 기준을 키로 표현할 수도 있습니다. 튜플은 앞 요소부터 비교하고(사전식),
    // Reverse(x)는 x의 비교 결과를 뒤집은 키를 만듭니다. 그래서 (Reverse(점수), 이름)은 "점수 내림차순, 이름 오름차순"입니다.
    students.sort_by_key(|s| (Reverse(s.score), s.name));
    let by_key: Vec<&str> = students.iter().map(|s| s.name).collect();
    println!("sort_by:     {by_cmp:?}"); // 출력: sort_by:     ["도윤", "민준", "하윤", "서연", "지우"]
    println!("sort_by_key: {by_key:?}"); // 출력: sort_by_key: ["도윤", "민준", "하윤", "서연", "지우"]

    // 5. sort_by_cached_key: 키를 만드는 비용이 클 때 요소마다 한 번만 계산합니다.
    // sort_by_key는 비교할 때마다 키 함수를 다시 부릅니다. 키가 to_lowercase()처럼 새 String을 만든다면
    // 그만큼 할당이 반복됩니다. 호출 횟수를 세어 비교합니다(sort_by_key의 횟수는 이 툴체인에서의 결과).
    println!("\n== 5. sort_by_key와 sort_by_cached_key의 키 함수 호출 횟수 ==");
    let words = [
        "fig", "Banana", "kiwi", "apple", "Grape", "date", "Cherry", "Lemon",
    ];
    let mut plain = words;
    plain.sort(); // 문자열의 기본 순서는 바이트(코드 포인트) 순이라 대문자가 소문자보다 앞에 옵니다.
    println!("sort:               {plain:?}");
    // 출력: sort:               ["Banana", "Cherry", "Grape", "Lemon", "apple", "date", "fig", "kiwi"]
    let mut calls = 0;
    let mut by_key = words;
    by_key.sort_by_key(|w| {
        calls += 1;
        w.to_lowercase()
    });
    let mut cached_calls = 0;
    let mut cached = words;
    cached.sort_by_cached_key(|w| {
        cached_calls += 1;
        w.to_lowercase()
    });
    println!("sort_by_cached_key: {cached:?}");
    // 출력: sort_by_cached_key: ["apple", "Banana", "Cherry", "date", "fig", "Grape", "kiwi", "Lemon"]
    println!("결과가 같은가? {}", by_key == cached); // 출력: 결과가 같은가? true
    println!("키 함수 호출: sort_by_key {calls}번, sort_by_cached_key {cached_calls}번"); // 출력: 키 함수 호출: sort_by_key 34번, sort_by_cached_key 8번

    // 6. 실수 정렬: f64는 Ord가 아니라서 sort()를 쓸 수 없습니다(compile_fail/e0277_sort_f64.rs).
    // NaN은 어떤 값과도(자기 자신과도) 같지 않아 "전순서"가 성립하지 않기 때문입니다.
    // total_cmp는 IEEE 754의 totalOrder 규칙으로 NaN과 -0.0까지 순서를 정해 주는 비교 메서드입니다.
    println!("\n== 6. f64 정렬과 total_cmp ==");
    let mut temps = [36.5, -1.0, f64::NAN, 0.0, -0.0, 37.25, f64::INFINITY];
    temps.sort_by(|a, b| a.total_cmp(b));
    println!("오름차순: {temps:?}"); // 출력: 오름차순: [-1.0, -0.0, 0.0, 36.5, 37.25, inf, NaN]
    temps.sort_by(|a, b| b.total_cmp(a));
    println!("내림차순: {temps:?}"); // 출력: 내림차순: [NaN, inf, 37.25, 36.5, 0.0, -0.0, -1.0]
    // NaN이 섞일 일이 없다고 확신한다면 partial_cmp(b).unwrap()도 쓸 수 있지만, NaN을 만나면 패닉합니다.
    let mut prices = [12.5, 3.0, 7.75];
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("partial_cmp: {prices:?}"); // 출력: partial_cmp: [3.0, 7.75, 12.5]
}
