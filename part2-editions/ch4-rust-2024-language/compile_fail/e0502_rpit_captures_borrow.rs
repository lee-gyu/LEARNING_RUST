// 에디션별 결과: 2024에서 실패, 2021에서 통과
// 기대하는 에러(2024): E0502 — cannot borrow `scores` as mutable because it is also borrowed as immutable
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0502_rpit_captures_borrow.rs --out-dir /tmp/rust-p2-ch4   # 실패
//   rustc --edition 2021 compile_fail/e0502_rpit_captures_borrow.rs --out-dir /tmp/rust-p2-ch4   # 통과
//
// `positions`의 반환 타입 `impl Iterator<Item = usize>`(RPIT)는 2024 에디션에서 인자
// `numbers`의 라이프타임을 자동으로 캡처합니다. 실제 숨은 타입(`Range<usize>`)은 아무것도
// 빌리지 않지만, 부르는 쪽은 시그니처만 보므로 `it`이 살아 있는 동안 `scores`를 빌린 것으로
// 취급합니다. 그래서 그 사이의 `scores.push(90)`이 거부됩니다.
// 2021 에디션에서는 이런 함수가 라이프타임을 캡처하지 않았으므로 통과합니다.
// 해결: 캡처하지 않겠다고 `impl Iterator<Item = usize> + use<>`로 적습니다.
// 또는 `it`을 다 쓴 뒤에 `push`합니다.

fn positions(numbers: &[i32]) -> impl Iterator<Item = usize> {
    0..numbers.len()
}

fn main() {
    let mut scores = vec![70, 85];
    let it = positions(&scores);
    scores.push(90); // → 2024에서 E0502
    println!("{:?}", it.collect::<Vec<_>>());
}
