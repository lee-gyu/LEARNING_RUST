// 기대하는 에러: error[E0277]: the trait bound `f64: Ord` is not satisfied
// 핵심 메시지: the trait `Ord` is not implemented for `f64`
//             note: required by a bound in `slice::<impl [T]>::sort`
//
// sort는 요소 타입에 Ord(전순서)를 요구합니다. f64는 NaN 때문에 PartialOrd만 구현하므로 sort를 쓸 수 없습니다.
// 해결: temps.sort_by(|a, b| a.total_cmp(b));  (examples/02_sort.rs의 6번 구역)
//
// 확인: rustc --edition 2024 compile_fail/e0277_sort_f64.rs --out-dir <임시 디렉터리>

fn main() {
    let mut temps: Vec<f64> = vec![36.5, -1.0, 37.25, 0.0];
    temps.sort();
    println!("{temps:?}");
}
