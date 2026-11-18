// 기대하는 에러: E0308 — mismatched types: expected `u32`, found floating-point number
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0308_duration_times_float.rs --out-dir /tmp/rust-p1-ch10
//
// 재시도할 때마다 기다리는 시간을 1.5배씩 늘리려고 Duration에 실수를 곱했습니다.
// Duration은 * 연산자로 u32만 곱할 수 있습니다(impl Mul<u32> for Duration이 유일한 구현).
// 구현이 하나뿐이라 컴파일러는 오른쪽이 u32여야 한다고 보고, 실수가 오자 "타입이 다르다"(E0308)고 알립니다.
// 실수 곱셈은 반올림 방식과 음수·NaN 처리를 정해야 하므로 연산자 대신 이름 있는 메서드로 제공됩니다.
// 해결: delay.mul_f64(1.5) (음수나 NaN이면 패닉), 정수 배율이면 delay * 2
//       (examples/05_duration.rs의 3번 구역)

use std::time::Duration;

fn main() {
    let mut delay = Duration::from_millis(100);
    for attempt in 1..=3 {
        println!("{attempt}번째 시도 전 {delay:?} 기다림");
        delay = delay * 1.5;
    }
}
