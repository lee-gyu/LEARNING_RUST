// 이 예제가 보여 주는 것: 어댑터는 구조체를 돌려준다는 것(type_name, 크기), 원소가 체인을 하나씩 흐르는 순서,
//                        그리고 "바꾸기·거르기" 어댑터 — map, filter, filter_map, flat_map, flatten, inspect, copied/cloned
// 실행: cargo run -p p1-ch5-iterators --example 03_transform_filter
//
// map을 부르면 클로저가 실행되는 것이 아니라, "안쪽 반복자 + 클로저"를 필드로 가진 Map 구조체가 만들어질 뿐입니다.
// 이 구조체의 next가 불릴 때 비로소 안쪽 반복자의 next를 부르고 클로저를 적용합니다. 그래서 반복자는 게으릅니다.
// type_name 출력은 툴체인에 따라 바뀔 수 있고, size_of_val 값은 64비트 환경(이 문서는 macOS aarch64)에서의 결과입니다.

use std::any::type_name;
use std::mem::size_of_val;

fn type_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

fn main() {
    println!("== 1. 어댑터가 돌려주는 것은 구조체 ==");
    let scores = [72, 95, 58, 88];
    let chain = scores.iter().filter(|s| **s >= 70).map(|s| s / 10);
    println!("타입: {}", type_of(&chain));
    // 출력: 타입: core::iter::adapters::map::Map<core::iter::adapters::filter::Filter<core::slice::iter::Iter<'_, i32>, 03_transform_filter::main::{{closure}}>, 03_transform_filter::main::{{closure}}>
    // 바깥에서 안쪽으로 읽으면 Map( Filter( slice::Iter, 클로저 ), 클로저 )입니다. 어댑터를 붙일 때마다 한 겹씩 감쌉니다.

    // size_of_val은 값이 메모리에서 차지하는 바이트 수를 돌려줍니다(std::mem은 ch7-smart-pointers-memory에서 다룹니다).
    let base = 0..10;
    let no_capture = (0..10).map(|n| n * 2);
    let limit = 5;
    let by_copy = (0..10).map(move |n| n * limit); // limit(i32, 4바이트)을 복사해 들고 있음
    let by_ref = (0..10).map(|n| n * limit); // &limit(참조, 8바이트)을 들고 있음
    println!(
        "크기: Range {}바이트, map(캡처 없음) {}, map(move) {}, map(참조 캡처) {}",
        size_of_val(&base),
        size_of_val(&no_capture),
        size_of_val(&by_copy),
        size_of_val(&by_ref)
    );
    println!("filter + map 체인: {}바이트", size_of_val(&chain));
    // 출력: 크기: Range 8바이트, map(캡처 없음) 8, map(move) 12, map(참조 캡처) 16
    // 출력: filter + map 체인: 16바이트
    // 어댑터 구조체는 "안쪽 반복자 + 클로저가 캡처한 값"만큼의 크기입니다. 힙 할당도, 중간 결과를 담는 벡터도 없습니다.
    let collected: Vec<i32> = chain.collect();
    println!("결과: {collected:?}");
    // 출력: 결과: [7, 9, 8]

    println!("\n== 2. 원소는 체인을 하나씩 끝까지 흐릅니다 ==");
    // inspect는 원소를 바꾸지 않고 들여다보기만 하는 어댑터입니다. 체인 중간에 끼워 흐름을 관찰합니다.
    let first_two: Vec<i32> = scores
        .iter()
        .inspect(|s| println!("  꺼냄: {s}"))
        .filter(|s| **s >= 70)
        .inspect(|s| println!("    통과: {s}"))
        .map(|s| s / 10)
        .take(2)
        .collect();
    println!("앞의 두 개: {first_two:?}");
    // 출력:   꺼냄: 72
    // 출력:     통과: 72
    // 출력:   꺼냄: 95
    // 출력:     통과: 95
    // 출력: 앞의 두 개: [7, 9]
    // take(2)가 두 개를 받자 더 요청하지 않았으므로 58과 88은 꺼내지도 않았습니다.

    println!("\n== 3. map / filter / filter_map ==");
    let inputs = ["3", "열", "7", "", "12"];
    // map: 원소마다 바꿉니다. 원소 수는 그대로입니다.
    let lens: Vec<usize> = inputs.iter().map(|s| s.len()).collect();
    // filter: 조건이 true인 원소만 남깁니다. 원소는 바꾸지 않습니다. 클로저는 &Item(여기서는 &&str)을 받습니다.
    let non_empty: Vec<&str> = inputs.iter().copied().filter(|s| !s.is_empty()).collect();
    // filter_map: 바꾸면서 거릅니다. 클로저가 Some(v)를 돌려준 원소만 v로 남습니다.
    let numbers: Vec<i32> = inputs.iter().filter_map(|s| s.parse().ok()).collect();
    println!("map:        {lens:?}");
    println!("filter:     {non_empty:?}");
    println!("filter_map: {numbers:?}");
    // 출력: map:        [1, 3, 1, 0, 2]
    // 출력: filter:     ["3", "열", "7", "12"]
    // 출력: filter_map: [3, 7, 12]

    println!("\n== 4. flat_map / flatten ==");
    let lines = ["fn main", "let x = 1", ""];
    // flat_map: 원소 하나를 "여러 개를 내놓는 것(IntoIterator)"으로 바꾼 뒤 이어 붙입니다.
    let words: Vec<&str> = lines.iter().flat_map(|l| l.split_whitespace()).collect();
    println!("flat_map(split_whitespace): {words:?}");
    // 출력: flat_map(split_whitespace): ["fn", "main", "let", "x", "=", "1"]

    // flatten: 원소 자체가 IntoIterator일 때 한 겹 벗깁니다. map(f).flatten()은 flat_map(f)와 같습니다
    // (clippy는 map(..).flatten()을 보면 flat_map으로 바꾸라고 제안합니다: map_flatten).
    let nested = vec![vec![1, 2], vec![], vec![3]];
    let flat: Vec<i32> = nested.into_iter().flatten().collect();
    // Option과 Result도 IntoIterator(Some/Ok면 값 하나, None/Err면 0개)이므로 flatten이 None과 Err를 건너뜁니다.
    let maybe = [Some('가'), None, Some('나')];
    let present: String = maybe.into_iter().flatten().collect();
    let results: Vec<Result<u8, std::num::ParseIntError>> =
        ["1", "x", "3"].iter().map(|s| s.parse()).collect();
    let parsed: Vec<u8> = results.into_iter().flatten().collect();
    println!("Vec<Vec> flatten:    {flat:?}");
    println!("Option flatten:      {present:?}");
    println!("Result flatten:      {parsed:?}");
    // 출력: Vec<Vec> flatten:    [1, 2, 3]
    // 출력: Option flatten:      "가나"
    // 출력: Result flatten:      [1, 3]
    // Result를 flatten하면 Err를 조용히 버립니다. 실패를 알아야 하면 Result<Vec<_>, _>로 collect합니다(예제 02).

    println!("\n== 5. copied / cloned ==");
    // iter()는 &T를 내놓습니다. copied()는 *x를 복사해(T: Copy), cloned()는 x.clone()으로(T: Clone) T를 내놓습니다.
    let prices = [1200, 3000, 800];
    let max_ref: Option<&i32> = prices.iter().max(); // 원소의 참조
    let max_val: Option<i32> = prices.iter().copied().max(); // 복사한 값
    println!("max: {max_ref:?}, copied().max(): {max_val:?}");
    // 출력: max: Some(3000), copied().max(): Some(3000)
    // 디버그 형식은 참조를 따라가 보여 주므로 출력은 같지만, 타입이 Option<&i32>와 Option<i32>로 다릅니다.

    let names = vec![
        String::from("가람"),
        String::from("나래"),
        String::from("다온"),
    ];
    // cloned()를 filter 뒤에 두면 걸러지고 남은 원소만 복제합니다.
    let picked: Vec<String> = names
        .iter()
        .filter(|n| n.ends_with('래'))
        .cloned()
        .collect();
    println!("cloned: {picked:?}, 원본: {names:?}");
    // 출력: cloned: ["나래"], 원본: ["가람", "나래", "다온"]
}
