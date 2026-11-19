// 02_box_slice_into_iter: Box<[T]>에 .into_iter()를 부르면 원소를 값으로 내놓습니다
//
// Box<[T]>는 길이가 정해진 힙 버퍼입니다(Part 1 ch7 2.7절). 1.80.0부터 Box<[T]>는 모든 에디션에서
// 값으로 IntoIterator를 구현하지만, 2021 이전 에디션의 코드가 **메서드 호출 문법으로**
// boxed.into_iter()를 부를 때만 컴파일러가 이 구현을 못 본 척하고 예전처럼 &[T]의 반복자를
// 고릅니다. 이 파일은 2024 에디션이므로 1번 구역이 값(u32)을 내놓습니다. 2021 에디션에서 같은 줄은
// Option<&u32>이고 boxed_slice_into_iter 경고가 납니다(projects/ecosystem_compare의 2번 구역).
//
// 실행: cargo run -p p2-ch5-rust-2024-ecosystem --example 02_box_slice_into_iter

use std::any::type_name_of_val;

fn scores() -> Box<[u32]> {
    vec![70, 85, 90].into_boxed_slice()
}

fn main() {
    println!("1. boxed.into_iter()는 값을 내놓습니다 (2024 에디션)");
    let boxed = scores();
    let mut iter = boxed.into_iter();
    // 반복자 타입은 Vec의 것과 같습니다. Box<[T]>를 Vec<T>로 바꾸는 비용이 없기 때문입니다.
    println!("    반복자: {}", type_name_of_val(&iter)); // 출력: 반복자: alloc::vec::into_iter::IntoIter<u32>
    let first = iter.next();
    println!("    첫 원소: {first:?}, {}", type_name_of_val(&first)); // 출력: 첫 원소: Some(70), core::option::Option<u32>
    // boxed는 into_iter(self)로 옮겨졌으므로 여기서부터 쓸 수 없습니다.

    println!("2. 값으로 꺼내야만 되는 일: String을 복제 없이 옮기기");
    let names: Box<[String]> = vec!["가람".to_string(), "나래".to_string()].into_boxed_slice();
    // 2021 에디션에서 이 줄은 &String으로 Vec<String>을 만들려다 E0277입니다
    // (compile_fail/e0277_box_slice_by_value_2021.rs).
    let mut members: Vec<String> = names.into_iter().collect();
    members.push("다온".to_string()); // Vec이 되었으니 다시 늘릴 수 있습니다
    println!("    {members:?}"); // 출력: ["가람", "나래", "다온"]
    // 원소마다 바꾸는 것도 값으로 받으니 새 String을 만들 필요가 없습니다.
    let greeted: Vec<String> = members.into_iter().map(|n| n + "님").collect();
    println!("    {greeted:?}"); // 출력: ["가람님", "나래님", "다온님"]

    println!("3. 참조가 필요하면 .iter() — 모든 에디션에서 &T");
    let boxed = scores();
    let total: u32 = boxed.iter().sum();
    let first_ref = boxed.iter().next();
    println!("    합계 {total}, {}", type_name_of_val(&first_ref)); // 출력: 합계 245, core::option::Option<&u32>
    println!("    boxed는 그대로: {boxed:?}"); // 출력: boxed는 그대로: [70, 85, 90]

    println!("4. 메서드 문법이 아닌 모양은 1.80부터 모든 에디션에서 값");
    // for는 IntoIterator::into_iter(scores())로 펼쳐지므로 에디션의 예외를 타지 않습니다.
    let mut kinds = Vec::new();
    for x in scores() {
        kinds.push(type_name_of_val(&x));
    }
    println!("    for x in boxed: {}", kinds[0]); // 출력: for x in boxed: u32
    let explicit = IntoIterator::into_iter(scores()).next();
    println!(
        "    IntoIterator::into_iter(boxed): {}",
        type_name_of_val(&explicit)
    ); // 출력: IntoIterator::into_iter(boxed): core::option::Option<u32>
}
