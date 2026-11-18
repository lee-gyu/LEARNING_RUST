// 기대하는 에러: error[E0271]: type mismatch resolving `<Vec<{integer}> as IntoIterator>::Item == &{integer}`
// 핵심 메시지: expected `&{integer}`, found integer
//             note: required by a bound in `std::iter::Iterator::chain`
//             (이어서 E0599: the method `copied` exists for struct `std::iter::Chain<...>`, but its trait bounds were not satisfied)
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0271_chain_item_mismatch.rs --out-dir /tmp/rust-p1-ch5
//
// chain의 시그니처는 fn chain<U>(self, other: U) -> Chain<Self, U::IntoIter> where U: IntoIterator<Item = Self::Item> 입니다.
// 이어 붙일 쪽의 원소 타입이 앞쪽과 "정확히" 같아야 합니다. morning.iter()의 원소는 &i32인데,
// evening(Vec<i32>)을 값으로 넘기면 원소가 i32라서 맞지 않습니다. 참조와 값은 다른 타입입니다.
// 고치는 방법: 원소 타입을 한쪽으로 맞춥니다.
//   morning.iter().chain(&evening).copied().collect()     // 둘 다 &i32로 맞춘 뒤 복사
//   morning.iter().copied().chain(evening).collect()      // 둘 다 i32로 맞춤 (evening은 이동함)

fn main() {
    let morning = vec![3, 5];
    let evening = vec![7, 9];
    let all: Vec<i32> = morning.iter().chain(evening).copied().collect();
    println!("{all:?}");
}
