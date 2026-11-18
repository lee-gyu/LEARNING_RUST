// 이 예제가 보여 주는 것: 소비 메서드 — 접기(fold, reduce, sum, product, count, last),
//                        찾기와 묻기(find, find_map, position, any, all)와 단락 평가,
//                        최대·최소 고르기(max_by_key, min_by_key, max_by), 실패할 수 있는 접기(try_fold, try_for_each),
//                        비교(eq, cmp, lt, is_sorted)
// 실행: cargo run -p p1-ch5-iterators --example 06_consumers
//
// 소비 메서드는 next를 불러 반복자를 끝까지(또는 답이 나올 때까지) 꺼내고 결과값 하나를 돌려줍니다.
// 시그니처의 self 모양을 보면 다 쓴 뒤의 반복자를 다시 쓸 수 있는지 알 수 있습니다.
// fold, sum, count 등은 self(소유권을 가져감), find, any, all, position, try_fold 등은 &mut self(빌려서 일부만 꺼낼 수 있음)입니다.

use std::cmp::Ordering;

fn main() {
    println!("== 1. 접기: fold, reduce, sum, product ==");
    let prices = [1200, 3000, 800, 4500];
    // fold(초기값, |누적값, 원소| 새 누적값): 원소를 하나씩 누적값에 접어 넣습니다.
    // 누적값이 튜플이면 개수와 합계를 한 번의 순회로 함께 구할 수 있습니다.
    let (count, total) = prices.iter().fold((0, 0), |(n, sum), p| (n + 1, sum + p));
    println!(
        "fold로 개수·합계: {count}개, {total}원, 평균 {}원",
        total / count
    );
    // 출력: fold로 개수·합계: 4개, 9500원, 평균 2375원

    let words = ["iter", "into", "from", "extend"];
    let kebab = words.iter().fold(String::new(), |mut acc, w| {
        if !acc.is_empty() {
            acc.push('-');
        }
        acc.push_str(w);
        acc
    });
    println!("fold로 문자열: {kebab}");
    // 출력: fold로 문자열: iter-into-from-extend

    // reduce: 첫 원소를 초기값으로 씁니다. 그래서 빈 반복자면 None입니다.
    let longest = words
        .iter()
        .copied()
        .reduce(|a, b| if b.len() > a.len() { b } else { a });
    let none: Option<&str> = [].iter().copied().reduce(|a: &str, _| a);
    println!("reduce: {longest:?}, 빈 반복자: {none:?}");
    // 출력: reduce: Some("extend"), 빈 반복자: None

    // sum과 product는 결과 타입이 필요합니다(Sum, Product 트레이트). 빈 반복자의 sum은 0, product는 1입니다.
    let factorial: u64 = (1..=10).product();
    let empty_sum: i32 = [].iter().sum();
    let empty_product: i32 = [].iter().product();
    println!("10! = {factorial}, 빈 sum = {empty_sum}, 빈 product = {empty_product}");
    // 출력: 10! = 3628800, 빈 sum = 0, 빈 product = 1
    println!(
        "count = {}, last = {:?}, max = {:?}",
        prices.iter().filter(|p| **p >= 1000).count(),
        prices.iter().last(),
        prices.iter().max()
    );
    // 출력: count = 3, last = Some(4500), max = Some(4500)

    println!("\n== 2. 찾기와 묻기: 답이 나오면 멈춥니다(단락 평가) ==");
    let scores = [72, 95, 58, 88, 64];
    let mut checked = 0;
    let first_high = scores.iter().inspect(|_| checked += 1).find(|s| **s >= 90);
    println!("find(>= 90) = {first_high:?}, 검사한 원소 {checked}개");
    // 출력: find(>= 90) = Some(95), 검사한 원소 2개

    let mut checked = 0;
    let any_fail = scores.iter().inspect(|_| checked += 1).any(|s| *s < 60);
    println!("any(< 60) = {any_fail}, 검사한 원소 {checked}개");
    // 출력: any(< 60) = true, 검사한 원소 3개

    let mut checked = 0;
    let all_pass = scores.iter().inspect(|_| checked += 1).all(|s| *s >= 60);
    println!("all(>= 60) = {all_pass}, 검사한 원소 {checked}개");
    // 출력: all(>= 60) = false, 검사한 원소 3개
    // any는 true를, all은 false를 처음 만나는 순간 답이 정해지므로 나머지를 보지 않습니다.

    let empty: [i32; 0] = [];
    println!(
        "빈 반복자: any = {}, all = {}",
        empty.iter().any(|_| true),
        empty.iter().all(|_| false)
    );
    // 출력: 빈 반복자: any = false, all = true
    // "조건을 어기는 원소가 하나도 없다"는 뜻이라 빈 반복자의 all은 true입니다.

    let lines = ["# 설정", "", "port = 8080", "host = local"];
    let first_value = lines
        .iter()
        .find_map(|l| l.split_once(" = ").map(|(_, v)| v));
    let blank_at = lines.iter().position(|l| l.is_empty());
    println!("find_map: {first_value:?}, position(빈 줄): {blank_at:?}");
    // 출력: find_map: Some("8080"), position(빈 줄): Some(1)
    // find_map은 filter_map(..).next()와 같고, position은 조건을 처음 만족하는 원소의 순번을 돌려줍니다.

    println!("\n== 3. 최대·최소 고르기 ==");
    let players = [("가람", 88), ("나래", 95), ("다온", 95), ("라온", 72)];
    // max_by_key / min_by_key: 키를 뽑는 클로저를 받습니다. 키가 같은 원소가 여럿이면
    // max_by_key는 마지막 것을, min_by_key는 첫 번째 것을 돌려준다고 문서에 정해져 있습니다.
    println!("max_by_key: {:?}", players.iter().max_by_key(|p| p.1));
    println!("min_by_key: {:?}", players.iter().min_by_key(|p| p.1));
    // 출력: max_by_key: Some(("다온", 95))
    // 출력: min_by_key: Some(("라온", 72))

    // f64는 Ord가 아니라서 max()와 max_by_key를 쓸 수 없습니다. max_by에 total_cmp를 넘깁니다(ch4의 정렬과 같은 이유).
    let temps: [f64; 3] = [36.5, 38.2, 37.1]; // 타입을 적지 않으면 {float}의 메서드를 찾지 못합니다(ch1의 E0689와 같은 이유).
    let highest = temps.iter().copied().max_by(|a, b| a.total_cmp(b));
    println!("max_by(total_cmp): {highest:?}");
    // 출력: max_by(total_cmp): Some(38.2)

    println!("\n== 4. 실패할 수 있는 접기: try_fold, try_for_each ==");
    // try_fold의 클로저는 Option이나 Result를 돌려줍니다. None/Err가 나오면 즉시 멈추고 그것을 돌려줍니다.
    let fits: Option<u8> = [100u8, 100, 50]
        .iter()
        .try_fold(0u8, |acc, &x| acc.checked_add(x));
    let overflow: Option<u8> = [100u8, 100, 60]
        .iter()
        .try_fold(0u8, |acc, &x| acc.checked_add(x));
    println!("u8 합계: {fits:?}, 넘치면: {overflow:?}");
    // 출력: u8 합계: Some(250), 넘치면: None

    // try_for_each는 원소마다 일을 하다가 처음 실패하면 멈춥니다. &mut self라서 멈춘 뒤 남은 원소를 이어서 꺼낼 수 있습니다.
    let mut signups = ["가람", "나래", "", "다온"].iter();
    let result: Result<(), String> = signups.try_for_each(|name| {
        if name.is_empty() {
            return Err(String::from("빈 이름은 등록할 수 없습니다"));
        }
        println!("  등록: {name}");
        Ok(())
    });
    let remaining: Vec<&&str> = signups.collect();
    println!("결과: {result:?}");
    println!("처리하지 않은 원소: {remaining:?}");
    // 출력:   등록: 가람
    // 출력:   등록: 나래
    // 출력: 결과: Err("빈 이름은 등록할 수 없습니다")
    // 출력: 처리하지 않은 원소: ["다온"]

    println!("\n== 5. 비교: eq, cmp, lt, is_sorted ==");
    // 반복자끼리는 원소를 앞에서부터 하나씩 비교합니다(사전식). 서로 다른 종류의 반복자도 원소 타입만 맞으면 비교됩니다.
    println!("(1..=3).eq([1, 2, 3]) = {}", (1..=3).eq([1, 2, 3]));
    // 출력: (1..=3).eq([1, 2, 3]) = true

    // 새 문자열을 만들지 않고 대소문자를 무시해 비교하기
    let same = "Rust"
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .eq("rUST".chars().map(|c| c.to_ascii_lowercase()));
    println!("대소문자 무시 비교: {same}");
    // 출력: 대소문자 무시 비교: true

    // 버전 문자열 비교: 문자열로 비교하면 "1.10.0" < "1.9.1"이지만, 숫자 반복자로 비교하면 올바릅니다.
    let parts = |v: &'static str| v.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    let by_text = "1.10.0".cmp("1.9.1");
    let by_numbers: Ordering = parts("1.10.0").cmp(parts("1.9.1"));
    println!("문자열 비교: {by_text:?}, 숫자 반복자 비교: {by_numbers:?}");
    println!("lt: {}", parts("1.2").lt(parts("1.2.1"))); // 앞부분이 같으면 짧은 쪽이 작습니다.
    // 출력: 문자열 비교: Less, 숫자 반복자 비교: Greater
    // 출력: lt: true

    println!(
        "is_sorted: {} {}",
        prices.iter().is_sorted(),
        (1..=5).is_sorted()
    );
    // 출력: is_sorted: false true
}
