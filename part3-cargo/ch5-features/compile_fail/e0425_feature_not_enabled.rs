// 기대하는 에러: E0425
//   error[E0425]: cannot find function `average` in this scope
//   note: found an item that was configured out
//   … the item is gated behind the `stats` feature
//
// 기능으로 가린 함수를, 그 기능을 켜지 않고 부른 실수입니다.
// `#[cfg(feature = "stats")]`가 붙은 항목은 기능이 꺼져 있으면 컴파일 결과에서 사라지므로,
// 부르는 쪽에서는 "그런 함수가 없다"는 에러가 납니다. 컴파일러는 지워진 항목을 기억해 두었다가
// 어느 기능 뒤에 가려져 있는지 note로 알려 줍니다.
//
// Cargo는 켜진 기능마다 `rustc`에 `--cfg 'feature="이름"'`을 넘깁니다. `rustc`로 이 파일만
// 컴파일하면 그 옵션이 하나도 없으므로 "기능을 켜지 않은" 상황이 됩니다.
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0425_feature_not_enabled.rs
// 비교: rustc --edition 2024 --cfg 'feature="stats"' --out-dir <임시 디렉터리> compile_fail/e0425_feature_not_enabled.rs
//       (기능을 켠 것과 같으므로 컴파일됩니다)

#[cfg(feature = "stats")]
fn average(scores: &[u32]) -> f64 {
    scores.iter().sum::<u32>() as f64 / scores.len() as f64
}

fn main() {
    let scores = [72, 95, 88];
    println!("평균 {:.1}", average(&scores));
}
