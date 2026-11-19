// 2021 에디션부터 배열의 .into_iter()가 원소의 참조가 아니라 원소 값을 내놓습니다.
// 반복자가 내놓는 항목의 타입을 출력해 세 가지 모양을 비교하고, 값으로 꺼내서 생기는 이점을 봅니다.
//
// 실행: cargo run -p p2-ch3-rust-2021 --example 02_array_into_iter

use std::any::type_name;

/// 반복자가 내놓는 항목(Item)의 타입 이름을 돌려줍니다. 반복자를 소비하지 않습니다.
fn item_type<I: Iterator>(_: &I) -> &'static str {
    type_name::<I::Item>()
}

fn main() {
    println!("1. 반복자가 내놓는 항목의 타입");
    let arr = [10, 20, 30];
    println!(
        "    arr.into_iter()                → {}",
        item_type(&arr.into_iter())
    );
    // 출력:     arr.into_iter()                → i32
    println!(
        "    arr.iter()                     → {}",
        item_type(&arr.iter())
    );
    // 출력:     arr.iter()                     → &i32
    // &[T; N]의 IntoIterator 구현을 고른 것입니다. 2018 에디션의 arr.into_iter()가 이것이었습니다.
    let by_ref = IntoIterator::into_iter(&arr);
    println!(
        "    IntoIterator::into_iter(&arr)  → {}",
        item_type(&by_ref)
    );
    // 출력:     IntoIterator::into_iter(&arr)  → &i32
    // for x in arr는 IntoIterator::into_iter(arr)를 부르므로 어느 에디션에서나 값입니다.
    let by_value = IntoIterator::into_iter(arr);
    println!(
        "    IntoIterator::into_iter(arr)   → {}",
        item_type(&by_value)
    );
    // 출력:     IntoIterator::into_iter(arr)   → i32

    println!("2. Copy 배열은 into_iter() 뒤에도 쓸 수 있습니다");
    // [i32; 3]은 Copy라서 into_iter()가 배열의 복사본을 가져갑니다.
    let doubled: Vec<i32> = arr.into_iter().map(|x| x * 2).collect();
    println!("    doubled = {doubled:?}, arr = {arr:?}");
    // 출력:     doubled = [20, 40, 60], arr = [10, 20, 30]

    println!("3. Copy가 아닌 원소는 복제 없이 옮겨 낼 수 있습니다");
    let names = [
        String::from("민수"),
        String::from("지영"),
        String::from("현우"),
    ];
    // 값으로 꺼내므로 String을 그대로 Vec으로 옮깁니다. 참조를 내놓는 iter()였다면
    // .cloned()로 복제하거나 다른 방법을 찾아야 합니다.
    let mut owned: Vec<String> = names.into_iter().collect();
    owned.push(String::from("서연"));
    println!("    owned = {owned:?}");
    // 출력:     owned = ["민수", "지영", "현우", "서연"]
    // names는 이동되었으므로 여기서 쓰면 E0382입니다.
    // 2018에서는: names.into_iter()가 &String을 내놓으므로 collect가 E0277
    // (a value of type `Vec<String>` cannot be built from an iterator over elements of type `&String`)입니다.

    println!("4. IntoIterator를 받는 곳에는 배열을 그대로 넘깁니다");
    // zip, chain, extend는 IntoIterator를 받으므로 배열을 넘기면 원소 값이 들어옵니다.
    // 이 동작은 배열이 IntoIterator를 구현한 1.53.0부터 모든 에디션에서 같습니다.
    // 에디션에 따라 달라지는 것은 배열에 직접 .into_iter()를 부르는 메서드 호출 문법뿐입니다.
    let pairs: Vec<(char, i32)> = "abc".chars().zip([1, 2, 3]).collect();
    let mut all = vec![0];
    all.extend([1, 2]);
    let chained: Vec<i32> = all.into_iter().chain([3, 4]).collect();
    println!("    zip = {pairs:?}");
    // 출력:     zip = [('a', 1), ('b', 2), ('c', 3)]
    println!("    extend + chain = {chained:?}");
    // 출력:     extend + chain = [0, 1, 2, 3, 4]
}
