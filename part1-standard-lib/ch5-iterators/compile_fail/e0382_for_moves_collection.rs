// 기대하는 에러: error[E0382]: borrow of moved value: `names`
// 핵심 메시지: `names` moved due to this implicit call to `.into_iter()`
//             note: the `for` loop is desugared into a call to `std::iter::IntoIterator::into_iter`,
//                   which takes ownership of the receiver `self`, which moves `names`
//             help: consider iterating over a slice of the `Vec<String>`'s content to avoid moving into the `for` loop
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0382_for_moves_collection.rs --out-dir /tmp/rust-p1-ch5
//
// for name in names 는 IntoIterator::into_iter(names)를 부릅니다. Vec<String>의 into_iter는 self를 값으로 받으므로
// names의 소유권이 반복자로 옮겨 가고, 반복이 끝나면 반복자와 함께 버려집니다. 그 뒤에 names를 쓸 수 없습니다.
// 고치는 방법: 원소를 읽기만 한다면 &names를 넘깁니다. 그러면 &Vec<String>의 IntoIterator가 골라져 name은 &String이 됩니다.
//   for name in &names { ... }   // names.iter()와 같음
// 원소의 소유권이 정말 필요하다면(다른 컬렉션으로 옮기는 등) 반복 뒤에 names를 쓰지 않도록 코드를 바꿉니다.

fn main() {
    let names = vec![String::from("가람"), String::from("나래")];
    for name in names {
        println!("안녕, {name}");
    }
    println!("모두 {}명", names.len());
}
