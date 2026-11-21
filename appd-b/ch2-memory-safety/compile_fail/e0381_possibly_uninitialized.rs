// 기대하는 에러: E0381
// 핵심 메시지: used binding `limit` is possibly-uninitialized
//
// 초기화 안전성: 값을 넣지 않은 변수를 읽는 길이 하나라도 있으면 컴파일되지 않습니다.
// `verbose`가 거짓인 실행에서는 `limit`에 아무것도 들어가지 않습니다. C라면 값이 정해지지 않은
// 변수를 읽게 되어, 그 자리의 메모리에 남아 있던 이전 데이터가 나오거나 최적화에 따라 더 이상한
// 결과가 나올 수 있습니다(ch10 `ch10-undefined-behavior`). 컴파일러는 조건이 실제로 거짓이 될 수
// 있는지 따지지 않고, 그런 길이 코드에 있는지만 봅니다.
//
// 고치는 법: 모든 길에서 값을 넣습니다.
//   let limit = if verbose { 100 } else { 20 };
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch2-memory-safety compile_fail/e0381_possibly_uninitialized.rs

fn main() {
    let verbose = std::env::args().count() > 1;
    let limit: usize;
    if verbose {
        limit = 100;
    }
    println!("최대 {limit}줄까지 출력합니다"); // E0381
}
