// 기대하는 에러: error[E0277]: a value of type `Vec<i32>` cannot be built from an iterator over elements of type `Result<i32, ParseIntError>`
// 핵심 메시지: help: the trait `FromIterator<Result<i32, ParseIntError>>` is not implemented for `Vec<i32>`
//                   but trait `FromIterator<i32>` is implemented for it
//             note: `Iterator::Item` changed to `Result<i32, ParseIntError>` here
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_collect_results_into_vec.rs --out-dir /tmp/rust-p1-ch2
//
// map(|s| s.parse::<i32>())이 내놓는 원소는 i32가 아니라 Result<i32, ParseIntError>입니다.
// Vec<i32>는 i32 원소로만 만들 수 있으므로, 실패할 수도 있는 원소들을 어떻게 할지 정하지 않은 채로는 모을 수 없습니다.
// 고치는 방법: 실패를 어떻게 다룰지 결과 타입으로 정합니다.
//   - 하나라도 실패하면 전체 실패:  let numbers: Result<Vec<i32>, _> = ...collect();   (뒤에 ?를 붙여 전파할 수도 있음)
//   - 실패도 그대로 보관:          let numbers: Vec<Result<i32, _>> = ...collect();
//   - 실패는 건너뜀:               inputs.iter().filter_map(|s| s.parse().ok()).collect::<Vec<i32>>()
// 세 가지 모두 examples/08_iter_collect.rs에 있습니다.

fn main() {
    let inputs = ["1", "2", "셋"];
    let numbers: Vec<i32> = inputs.iter().map(|s| s.parse::<i32>()).collect();
    println!("{numbers:?}");
}
