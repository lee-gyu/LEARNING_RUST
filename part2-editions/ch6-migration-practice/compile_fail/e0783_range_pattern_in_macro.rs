// 기대하는 에러: E0783
// 핵심 메시지: `...` range patterns are deprecated
// 에디션: 2021, 2024에서 실패 / 2018에서는 경고(ellipsis_inclusive_range_patterns)와 함께 통과
//
// 매크로 본문에 남은 옛 문법은 매크로가 "펼쳐질 때" 에러가 됩니다. 그래서 에러는 매크로를 부른
// 줄에서 보고되고, 메시지가 매크로 본문의 위치를 함께 알려 줍니다. 이 파일처럼 매크로를 같은
// 크레이트에서 부르면 `cargo fix --edition`이 본문을 고쳐 주지만, `#[macro_export]`로 내보내기만
// 하고 정의한 크레이트에서는 부르지 않는 매크로는 `cargo fix`가 볼 기회가 없습니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0783_range_pattern_in_macro.rs --out-dir /tmp/rust-p2-ch6   → 에러
//   rustc --edition 2021 compile_fail/e0783_range_pattern_in_macro.rs --out-dir /tmp/rust-p2-ch6   → 에러
//   rustc --edition 2018 compile_fail/e0783_range_pattern_in_macro.rs --out-dir /tmp/rust-p2-ch6   → 경고와 함께 통과

/// 점수를 등급으로 바꿉니다.
macro_rules! grade {
    ($score:expr) => {
        match $score {
            90...100 => 'A', // → 2021 에디션부터 E0783
            80...89 => 'B',
            _ => 'C',
        }
    };
}

fn main() {
    let scores = [95, 83, 40];
    for score in scores.iter() {
        println!("{} → {}", score, grade!(*score));
    }
}
