// 이 예제가 보여 주는 것: IntoIterator — for 문이 실제로 부르는 트레이트.
//                        for를 손으로 풀어 쓰기, Vec/&Vec/&mut Vec의 세 가지 구현과 그 Item 타입,
//                        여러 타입의 IntoIterator 구현, 반복자 자신도 IntoIterator라는 것, IntoIterator를 받는 함수
// 실행: cargo run -p p1-ch5-iterators --example 01_into_iterator
//
// for x in 값 은 IntoIterator::into_iter(값)으로 반복자를 얻은 뒤 None이 나올 때까지 next를 부르는 문법입니다.
// 그래서 "for에 무엇을 넣을 수 있는가"와 "x가 어떤 타입인가"는 그 값의 타입이 IntoIterator를 어떻게 구현했는지로 정해집니다.
// std::any::type_name::<T>()는 타입 T의 이름을 문자열로 돌려주는 함수입니다(std::any는 ch14-ffi-any-misc에서 다룹니다).
// 이름 문자열은 디버깅용이라 툴체인에 따라 바뀔 수 있으므로, 출력 주석은 이 문서를 쓸 때(rustc 1.100.0)의 결과입니다.

use std::any::type_name;
use std::collections::{BTreeMap, BTreeSet};

// C를 for에 넣으면 into_iter()가 돌려주는 반복자의 타입(C::IntoIter)과
// 반복 변수가 받는 값의 타입(C::Item)을 출력합니다. IntoIter와 Item은 IntoIterator의 연관 타입입니다(Part 0 ch19).
fn describe<C: IntoIterator>(label: &str) {
    println!("{label}");
    println!("    IntoIter = {}", type_name::<C::IntoIter>());
    println!("    Item     = {}", type_name::<C::Item>());
}

// for 문이 하는 일을 손으로 풀어 쓴 것입니다. clippy는 while let 대신 for를 쓰라고 제안하지만(while_let_on_iterator),
// 여기서는 into_iter와 next의 관계를 그대로 보여 주려고 일부러 풀어 씁니다.
#[expect(
    clippy::while_let_on_iterator,
    reason = "for 문의 동작을 into_iter와 next로 풀어 보여 주는 예제"
)]
fn manual_for(names: &Vec<String>) {
    let mut iter = IntoIterator::into_iter(names); // for name in names 의 첫 단계
    while let Some(name) = iter.next() {
        println!("  손으로 쓴 for: {name}");
    }
}

// "u32를 내놓는 것으로 바꿀 수 있는 것이면 무엇이든" 받습니다.
// Vec, 배열, 범위, Option, 집합, 그리고 반복자 자체까지 모두 넘길 수 있습니다.
fn total(prices: impl IntoIterator<Item = u32>) -> u32 {
    let mut sum = 0;
    for p in prices {
        sum += p;
    }
    sum
}

fn main() {
    println!("== 1. for 문을 풀어 쓰면 ==");
    let names = vec![String::from("가람"), String::from("나래")];
    for name in &names {
        println!("  for 문: {name}");
    }
    manual_for(&names);
    // 출력:   for 문: 가람
    // 출력:   for 문: 나래
    // 출력:   손으로 쓴 for: 가람
    // 출력:   손으로 쓴 for: 나래

    println!("\n== 2. Vec의 세 가지 IntoIterator 구현 ==");
    // 같은 Vec이라도 값으로 넘기는지, &로 넘기는지, &mut로 넘기는지에 따라 서로 다른 impl이 골라집니다.
    describe::<Vec<String>>("for x in v");
    describe::<&Vec<String>>("for x in &v");
    describe::<&mut Vec<String>>("for x in &mut v");
    // 출력: for x in v
    // 출력:     IntoIter = alloc::vec::into_iter::IntoIter<alloc::string::String>
    // 출력:     Item     = alloc::string::String
    // 출력: for x in &v
    // 출력:     IntoIter = core::slice::iter::Iter<'_, alloc::string::String>
    // 출력:     Item     = &alloc::string::String
    // 출력: for x in &mut v
    // 출력:     IntoIter = core::slice::iter::IterMut<'_, alloc::string::String>
    // 출력:     Item     = &mut alloc::string::String
    // &v의 IntoIter가 v.iter()의 반환 타입(slice::Iter)과 같습니다. 그래서 for x in &v 와 for x in v.iter()가 같습니다.

    let mut tags = vec![String::from("rust"), String::from("iter")];
    for tag in &mut tags {
        tag.push('!'); // Item이 &mut String이므로 원소를 그 자리에서 바꿀 수 있습니다.
    }
    println!("&mut로 고친 뒤: {tags:?}");
    // 출력: &mut로 고친 뒤: ["rust!", "iter!"]

    println!("\n== 3. 다른 타입들의 IntoIterator ==");
    describe::<[i32; 3]>("배열 [i32; 3]");
    describe::<Option<i32>>("Option<i32>");
    describe::<BTreeMap<&str, u32>>("BTreeMap<&str, u32>");
    describe::<&BTreeMap<&str, u32>>("&BTreeMap<&str, u32>");
    describe::<std::ops::Range<i32>>("범위 0..n");
    // 출력: 배열 [i32; 3]
    // 출력:     IntoIter = core::array::iter::IntoIter<i32, 3>
    // 출력:     Item     = i32
    // 출력: Option<i32>
    // 출력:     IntoIter = core::option::IntoIter<i32>
    // 출력:     Item     = i32
    // 출력: BTreeMap<&str, u32>
    // 출력:     IntoIter = alloc::collections::btree::map::IntoIter<&str, u32>
    // 출력:     Item     = (&str, u32)
    // 출력: &BTreeMap<&str, u32>
    // 출력:     IntoIter = alloc::collections::btree::map::Iter<'_, &str, u32>
    // 출력:     Item     = (&&str, &u32)
    // 출력: 범위 0..n
    // 출력:     IntoIter = core::ops::range::Range<i32>
    // 출력:     Item     = i32
    // 맵은 (키, 값) 튜플을 내놓고, &맵은 (&키, &값)을 내놓습니다. 범위는 그 자체가 반복자라서 IntoIter가 자기 자신입니다.

    println!("\n== 4. 반복자도 IntoIterator입니다 ==");
    // 표준 라이브러리에 impl<I: Iterator> IntoIterator for I 가 있어서, 모든 반복자는 자기 자신을 돌려주는 into_iter를 가집니다.
    describe::<std::iter::Rev<std::slice::Iter<'_, i32>>>("v.iter().rev()");
    // 출력: v.iter().rev()
    // 출력:     IntoIter = core::iter::adapters::rev::Rev<core::slice::iter::Iter<'_, i32>>
    // 출력:     Item     = &i32
    // 그래서 for x in v.iter().rev() 처럼 어댑터를 붙인 반복자도 for에 넣을 수 있습니다.

    println!("\n== 5. IntoIterator를 받는 함수 ==");
    let cart = vec![1200, 3000];
    let set: BTreeSet<u32> = BTreeSet::from([500, 700]);
    println!("Vec:        {}", total(cart.clone()));
    println!("배열:       {}", total([1000, 2000, 3000]));
    println!("범위:       {}", total(1..=4));
    println!("Some/None:  {} / {}", total(Some(800)), total(None));
    println!("BTreeSet:   {}", total(set));
    println!("반복자:     {}", total(cart.iter().map(|p| p / 100)));
    // 출력: Vec:        4200
    // 출력: 배열:       6000
    // 출력: 범위:       10
    // 출력: Some/None:  800 / 0
    // 출력: BTreeSet:   1200
    // 출력: 반복자:     42
    // total(&cart)는 Item이 &u32라서 Item = u32 바운드에 맞지 않습니다. 그럴 때는 cart.iter().copied()를 넘깁니다.
    println!("copied:     {}", total(cart.iter().copied()));
    println!("cart는 그대로: {cart:?}");
    // 출력: copied:     4200
    // 출력: cart는 그대로: [1200, 3000]
}
