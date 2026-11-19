// 기대하는 에러: E0277 — the trait bound `u64: From<i32>` is not satisfied
// 핵심 메시지: required for `i32` to implement `Into<u64>`
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_into_bound_int_literal.rs --out-dir /tmp/er-ch5
//
// `impl Into<u64>` 매개변수는 "u64로 바꿀 수 있는 무엇이든"을 받으므로 인자 자리가 리터럴의 타입을
// 정해 주지 못합니다. `u64: From<T>`를 만족하는 정수 타입이 여럿(`u8`, `u16`, `u32`, `u64`)이기
// 때문입니다. 타입을 정할 단서가 없는 정수 리터럴은 `i32`가 되고, `i32`는 음수를 담을 수 있어
// `u64: From<i32>`가 없습니다. 500은 u64에 들어가는 값인데도 컴파일되지 않습니다.
// 해결: 부르는 쪽은 `500_u32`처럼 접미사로 타입을 정합니다.
//       받는 쪽은 받을 타입이 정수 하나뿐이라면 `delay_ms: u64`로 받습니다. 그러면 리터럴이
//       `u64`로 정해지고, `u32` 변수를 가진 부르는 쪽은 `u64::from(x)`로 넘깁니다.

fn schedule_retry(delay_ms: impl Into<u64>) -> u64 {
    let delay_ms = delay_ms.into();
    delay_ms * 2
}

fn main() {
    let short: u16 = 100;
    println!("{}", schedule_retry(short)); // 타입이 정해진 변수는 됩니다.
    println!("{}", schedule_retry(500)); // E0277
}
