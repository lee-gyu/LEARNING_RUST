// 이 예제가 보여 주는 것: FromIterator와 Extend — collect와 extend가 실제로 부르는 트레이트.
//                        같은 반복자를 여러 타입으로 모으기, String으로 모으기, Result/Option 안에 모으기,
//                        튜플로 모으기와 unzip, extend, partition(결과 타입이 Default + Extend면 무엇이든)
// 실행: cargo run -p p1-ch5-iterators --example 02_from_iterator
//
// collect의 시그니처는 fn collect<B>(self) -> B where B: FromIterator<Self::Item> 입니다.
// collect는 아무것도 결정하지 않고, "결과 타입 B가 FromIterator<원소 타입>을 어떻게 구현했는가"가 동작을 정합니다.
// 그래서 결과 타입을 바꾸면 같은 반복자에서 전혀 다른 결과가 나옵니다.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::ParseIntError;

fn main() {
    println!("== 1. 같은 반복자, 다른 결과 타입 ==");
    let fruits = ["사과", "배", "사과", "감"];
    let as_vec: Vec<&str> = fruits.into_iter().collect();
    let as_deque: VecDeque<&str> = fruits.into_iter().collect();
    let as_set: BTreeSet<&str> = fruits.into_iter().collect(); // 집합이라 중복이 빠지고 정렬됩니다.
    let as_string: String = fruits.into_iter().collect(); // String: FromIterator<&str>는 이어 붙입니다.
    let lengths: BTreeMap<&str, usize> = fruits
        .into_iter()
        .map(|f| (f, f.chars().count())) // (키, 값) 튜플을 내놓으면 맵으로 모을 수 있습니다.
        .collect();
    println!("Vec:      {as_vec:?}");
    println!("VecDeque: {as_deque:?}");
    println!("BTreeSet: {as_set:?}");
    println!("String:   {as_string:?}");
    println!("BTreeMap: {lengths:?}");
    // 출력: Vec:      ["사과", "배", "사과", "감"]
    // 출력: VecDeque: ["사과", "배", "사과", "감"]
    // 출력: BTreeSet: {"감", "배", "사과"}
    // 출력: String:   "사과배사과감"
    // 출력: BTreeMap: {"감": 1, "배": 1, "사과": 2}

    // collect는 FromIterator::from_iter를 부를 뿐입니다. 아래 두 줄은 같은 일을 합니다.
    let a: Vec<i32> = (1..=3).collect();
    let b = Vec::from_iter(1..=3);
    println!("collect와 from_iter: {a:?} == {b:?} → {}", a == b);
    // 출력: collect와 from_iter: [1, 2, 3] == [1, 2, 3] → true

    println!("\n== 2. String으로 모으기 ==");
    // String은 char, &str, String, &char 등 여러 원소 타입에 대해 FromIterator를 구현합니다.
    let reversed: String = "반복자".chars().rev().collect(); // 원소: char
    let shouted: String = ["가", "나", "다"]
        .into_iter()
        .map(|s| s.to_string() + "!")
        .collect(); // 원소: String
    let initials: String = ["Rust", "Iterator"]
        .iter()
        .filter_map(|w| w.chars().next())
        .collect(); // 원소: char
    println!("char 모으기:   {reversed}");
    println!("String 모으기: {shouted}");
    println!("첫 글자 모으기: {initials}");
    // 출력: char 모으기:   자복반
    // 출력: String 모으기: 가!나!다!
    // 출력: 첫 글자 모으기: RI
    // ["가", "나"].iter()의 원소는 &&str입니다. String은 FromIterator<&&str>를 구현하지 않으므로
    // .iter().collect::<String>()은 E0277입니다. into_iter()나 .copied()로 원소를 &str로 만들어 모읍니다.

    println!("\n== 3. Result와 Option 안에 모으기 ==");
    // impl<T, E, V> FromIterator<Result<T, E>> for Result<V, E> where V: FromIterator<T>
    // "원소가 Result이고, 안쪽 값 T로 V를 만들 수 있으면 Result<V, E>를 만들 수 있다"는 뜻입니다.
    // V 자리에 Vec뿐 아니라 FromIterator를 구현한 무엇이든 올 수 있습니다.
    let inputs = ["30", "10", "20", "10"];
    let as_vec: Result<Vec<i32>, ParseIntError> = inputs.iter().map(|s| s.parse()).collect();
    let as_set: Result<BTreeSet<i32>, ParseIntError> = inputs.iter().map(|s| s.parse()).collect();
    let bad: Result<BTreeSet<i32>, ParseIntError> = ["1", "둘"].iter().map(|s| s.parse()).collect();
    println!("Result<Vec<_>, _>:      {as_vec:?}");
    println!("Result<BTreeSet<_>, _>: {as_set:?}");
    println!("하나라도 실패하면:      {bad:?}");
    // 출력: Result<Vec<_>, _>:      Ok([30, 10, 20, 10])
    // 출력: Result<BTreeSet<_>, _>: Ok({10, 20, 30})
    // 출력: 하나라도 실패하면:      Err(ParseIntError { kind: InvalidDigit })

    // Option도 같은 모양입니다. char::from_digit은 0~9가 아니면 None입니다.
    let digits: Option<String> = [3, 1, 4]
        .into_iter()
        .map(|d| char::from_digit(d, 10))
        .collect();
    let broken: Option<String> = [3, 12, 4]
        .into_iter()
        .map(|d| char::from_digit(d, 10))
        .collect();
    println!("Option<String>: {digits:?}, {broken:?}");
    // 출력: Option<String>: Some("314"), None

    println!("\n== 4. 튜플로 모으기: collect와 unzip ==");
    let scores = [("가람", 88), ("나래", 72), ("다온", 95)];
    // 원소가 (A, B)이면 (A를 모을 컬렉션, B를 모을 컬렉션)으로 나눠 모을 수 있습니다.
    let (names, points): (Vec<&str>, Vec<u32>) = scores.into_iter().unzip();
    println!("unzip:   {names:?} {points:?}");
    // collect도 튜플을 결과 타입으로 받습니다(1.79부터). 각 자리의 타입은 Default + Extend면 됩니다.
    let (names, joined): (BTreeSet<&str>, String) = scores
        .into_iter()
        .map(|(n, p)| (n, format!("{p} ")))
        .collect();
    println!("collect: {names:?} {joined:?}");
    // 출력: unzip:   ["가람", "나래", "다온"] [88, 72, 95]
    // 출력: collect: {"가람", "나래", "다온"} "88 72 95 "

    println!("\n== 5. Extend: 이미 있는 컬렉션에 더하기 ==");
    // collect가 새 컬렉션을 만든다면, extend는 있는 컬렉션 뒤에 반복자의 원소를 붙입니다(Extend 트레이트).
    let mut log = vec![1, 2];
    log.extend([3, 4]); // 배열(IntoIterator)
    log.extend(&[5, 6]); // Vec<T>는 T: Copy일 때 Extend<&T>도 구현하므로 참조 원소도 받습니다.
    log.extend((7..=9).filter(|n| n % 2 == 1)); // 어댑터를 붙인 반복자
    println!("Vec::extend:    {log:?}");
    let mut title = String::from("ch5");
    title.extend([':', ' ']); // String: Extend<char>
    title.extend(["반복자", "와 어댑터"]); // String: Extend<&str>
    println!("String::extend: {title}");
    let mut stock: BTreeMap<&str, u32> = BTreeMap::from([("사과", 3)]);
    stock.extend([("배", 5), ("사과", 9)]); // 같은 키가 있으면 값이 덮어써집니다.
    println!("BTreeMap::extend: {stock:?}");
    // 출력: Vec::extend:    [1, 2, 3, 4, 5, 6, 7, 9]
    // 출력: String::extend: ch5: 반복자와 어댑터
    // 출력: BTreeMap::extend: {"배": 5, "사과": 9}

    println!("\n== 6. partition: 결과 타입이 Default + Extend면 무엇이든 ==");
    // fn partition<B, F>(self, f: F) -> (B, B) where B: Default + Extend<Self::Item>, F: FnMut(&Self::Item) -> bool
    // 빈 컬렉션 두 개를 Default로 만들고, 조건이 true인 원소는 왼쪽에, false인 원소는 오른쪽에 extend합니다.
    let (even, odd): (Vec<i32>, Vec<i32>) = (1..=9).partition(|n| n % 2 == 0);
    println!("Vec 두 개:    {even:?} / {odd:?}");
    let (hangul, other): (String, String) = "Rust 반복자 101"
        .chars()
        .partition(|c| ('가'..='힣').contains(c));
    println!("String 두 개: {hangul:?} / {other:?}");
    // 출력: Vec 두 개:    [2, 4, 6, 8] / [1, 3, 5, 7, 9]
    // 출력: String 두 개: "반복자" / "Rust  101"
}
