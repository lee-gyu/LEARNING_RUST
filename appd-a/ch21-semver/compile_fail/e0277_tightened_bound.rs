// 기대하는 에러: E0277 — the trait bound `f64: Ord` is not satisfied
// 핵심 메시지: required by this bound in `summarize`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_tightened_bound.rs --out-dir /tmp/er-ch21
//
// 상황: 모듈 parcel은 다른 사람이 관리하는 라이브러리라고 생각합니다.
//   - 1.0.0의 summarize는 `T: Display`만 요구했고, 사용자는 kg 단위 무게(f64)를 넘겼습니다.
//   - 1.1.0에서 라이브러리가 "가벼운 것부터 보여 주자"며 본문에서 정렬하려고 바운드에 Ord를 더했습니다.
//     시그니처의 이름과 인자 수는 그대로인 작은 변경처럼 보입니다.
//   - f64는 NaN 때문에 Ord를 구현하지 않으므로(ch10 2.5절) 사용자의 호출이 깨집니다.
//     바운드를 조이는 것은 "받던 것을 덜 받는" 변경입니다(ch2 2.3절의 Fn* 바운드와 같은 원리).
// 해결: 라이브러리는 기존 함수의 바운드를 그대로 두고, 정렬하는 판을 새 함수
//       (예: summarize_sorted<T: Display + Ord>)로 더합니다. 새 함수 추가는 마이너입니다.
//       바운드를 꼭 조여야 한다면 메이저 버전에서 합니다.

mod parcel {
    use std::fmt::Display;

    // 1.1.0: 바운드에 Ord를 더하고 본문에서 정렬합니다
    pub fn summarize<T: Display + Ord>(weights: &[T]) -> String {
        let mut sorted: Vec<&T> = weights.iter().collect();
        sorted.sort();
        let parts: Vec<String> = sorted.iter().map(|w| format!("{w}kg")).collect();
        parts.join(" + ")
    }
}

fn main() {
    // 1.0.0 시절에 쓴 사용자 코드
    let weights = [1.2_f64, 0.8, 2.5];
    println!("{}", parcel::summarize(&weights)); // E0277
}
