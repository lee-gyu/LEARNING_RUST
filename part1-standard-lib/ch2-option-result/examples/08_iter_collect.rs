// 이 예제가 보여 주는 것: 반복자와의 만남 — Option을 반복자로 쓰기, 실패를 건너뛰기(filter_map, flatten),
//                        Result<Vec<T>, E>와 Option<Vec<T>>로 collect하기, Result/Option으로 sum하기
// 실행: cargo run -p p1-ch2-option-result --example 08_iter_collect
//
// Option은 "원소가 0개 또는 1개인 컬렉션"처럼 반복할 수 있고, 거꾸로 반복자의 결과를 Option/Result로 모을 수도
// 있습니다. 반복자 어댑터 전반과 collect가 여러 타입을 만들 수 있는 원리(FromIterator 트레이트)는 ch5-iterators에서 다룹니다.

use std::num::ParseIntError;

// 장바구니 상품의 가격. 모르는 상품은 None입니다.
fn price_of(item: &str) -> Option<u32> {
    match item {
        "사과" => Some(1200),
        "우유" => Some(2500),
        "빵" => Some(3000),
        _ => None,
    }
}

// Part 0 ch9 연습 1의 sum_csv를 반복자로 쓴 것입니다. 조각마다 parse한 Result들을 곧바로 sum합니다.
// 반환 타입이 Result<i32, _>이므로 sum은 "전부 Ok면 Ok(합계), 하나라도 Err면 첫 번째 Err"를 만듭니다.
fn sum_csv(line: &str) -> Result<i32, ParseIntError> {
    line.split(',').map(|part| part.trim().parse::<i32>()).sum()
}

// 함수 안에서 반복자의 원소마다 ?를 쓰고 싶을 때의 모양입니다.
// map의 클로저 안에서 ?를 쓰면 클로저에서 빠져나갈 뿐 이 함수에서 빠져나가지 않습니다(클로저가 i32를 돌려주면 E0277).
// 그래서 먼저 Result<Vec<_>, _>로 collect한 뒤, 그 결과에 ?를 한 번 붙입니다.
fn parse_all(line: &str) -> Result<Vec<i32>, ParseIntError> {
    let numbers = line
        .split(',')
        .map(|part| part.trim().parse::<i32>())
        .collect::<Result<Vec<i32>, _>>()?;
    println!("  (parse_all: 모두 읽었습니다)");
    Ok(numbers)
}

fn main() {
    println!("== 1. Option을 반복자로 쓰기 ==");
    // Some(v)는 v 하나를, None은 아무것도 내놓지 않는 반복자처럼 동작합니다.
    // chain이나 extend처럼 반복할 수 있는 것(IntoIterator)을 받는 자리에 Option을 그대로 넘길 수 있습니다.
    for bonus in ["4", "없음"] {
        let extra: Option<i32> = bonus.parse().ok();
        let numbers: Vec<i32> = (1..=3).chain(extra).collect();
        println!("보너스 {bonus:?} → {numbers:?}");
    }
    // 출력: 보너스 "4" → [1, 2, 3, 4]
    // 출력: 보너스 "없음" → [1, 2, 3]

    let mut tags = vec!["rust"];
    tags.extend(price_of("사과").map(|_| "과일")); // Some이면 하나를 추가
    tags.extend(price_of("수박").map(|_| "과일")); // None이면 아무것도 추가하지 않음
    println!("tags = {tags:?}");
    // 출력: tags = ["rust", "과일"]

    println!("\n== 2. 실패를 건너뛰기: filter_map, flatten ==");
    let inputs = ["10", "스물", "30", ""];
    // filter_map: 클로저가 Some(v)를 돌려준 원소만 v로 남깁니다. ok()로 실패를 None으로 바꿔 버리는 셈입니다.
    let parsed: Vec<i32> = inputs.iter().filter_map(|s| s.parse().ok()).collect();
    println!("filter_map: {parsed:?}");
    // 출력: filter_map: [10, 30]

    // 반복자의 flatten: 원소가 Option이면 Some 안의 값만 꺼내고 None은 건너뜁니다.
    let readings = [Some(3), None, Some(5), None];
    let present: Vec<i32> = readings.iter().flatten().copied().collect();
    println!("flatten: {present:?}");
    // 출력: flatten: [3, 5]
    // 이 방법들은 실패한 원소를 조용히 버립니다. 실패를 알아야 한다면 3번 구역처럼 모읍니다.

    println!("\n== 3. Result<Vec<T>, E>로 collect: 하나라도 실패하면 Err ==");
    // 같은 반복자라도 무엇으로 모을지(타입 표기)에 따라 결과가 달라집니다.
    let all: Vec<Result<i32, ParseIntError>> = inputs.iter().map(|s| s.parse::<i32>()).collect();
    println!("Vec<Result<..>>로: {all:?}");
    // 출력: Vec<Result<..>>로: [Ok(10), Err(ParseIntError { kind: InvalidDigit }), Ok(30), Err(ParseIntError { kind: Empty })]

    // Result<Vec<_>, _>로 모으면 첫 Err에서 멈춥니다. 클로저에 출력문을 넣어 어디까지 실행되는지 봅니다.
    let checked: Result<Vec<i32>, ParseIntError> = inputs
        .iter()
        .map(|s| {
            println!("  읽는 중: {s:?}");
            s.parse::<i32>()
        })
        .collect();
    println!("Result<Vec<..>>로: {checked:?}");
    // 출력:   읽는 중: "10"
    // 출력:   읽는 중: "스물"
    // 출력: Result<Vec<..>>로: Err(ParseIntError { kind: InvalidDigit })
    // 두 번째 원소에서 Err가 나오자 collect가 더 꺼내지 않았으므로 "30"과 ""은 읽지도 않았습니다(단락 평가).

    let good = ["1", "2", "3"];
    let checked: Result<Vec<i32>, ParseIntError> = good.iter().map(|s| s.parse::<i32>()).collect();
    println!("모두 성공하면: {checked:?}");
    // 출력: 모두 성공하면: Ok([1, 2, 3])

    println!("\n== 4. Option<Vec<T>>로 collect: 하나라도 None이면 None ==");
    for cart in [["사과", "우유"], ["사과", "수박"]] {
        let prices: Option<Vec<u32>> = cart.iter().map(|item| price_of(item)).collect();
        println!("{cart:?} → {prices:?}");
    }
    // 출력: ["사과", "우유"] → Some([1200, 2500])
    // 출력: ["사과", "수박"] → None

    println!("\n== 5. Option, Result로 sum ==");
    for cart in [["빵", "우유"], ["빵", "케이크"]] {
        let total: Option<u32> = cart.iter().map(|item| price_of(item)).sum();
        println!("{cart:?} 합계 → {total:?}");
    }
    // 출력: ["빵", "우유"] 합계 → Some(5500)
    // 출력: ["빵", "케이크"] 합계 → None

    for line in ["10, 20, 30", "1, 두, 3"] {
        println!("sum_csv({line:?}) → {:?}", sum_csv(line));
    }
    // 출력: sum_csv("10, 20, 30") → Ok(60)
    // 출력: sum_csv("1, 두, 3") → Err(ParseIntError { kind: InvalidDigit })

    println!("\n== 6. collect한 뒤 ?로 전파하기 ==");
    for line in ["4, 5, 6", "4, 오, 6"] {
        println!("parse_all({line:?})");
        println!("  → {:?}", parse_all(line));
    }
    // 출력: parse_all("4, 5, 6")
    // 출력:   (parse_all: 모두 읽었습니다)
    // 출력:   → Ok([4, 5, 6])
    // 출력: parse_all("4, 오, 6")
    // 출력:   → Err(ParseIntError { kind: InvalidDigit })
    // 두 번째 줄은 collect 결과가 Err라서 ?가 함수를 바로 끝냈으므로 "모두 읽었습니다"가 출력되지 않았습니다.
}
