// 기대하는 에러: error[E0277]: the trait bound `{float}: LowerHex` is not satisfied
// 핵심 메시지:   the trait `LowerHex` is not implemented for `{float}`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_float_lower_hex.rs --out-dir /tmp/rust-p1-ch3
//
// {:x}는 "16진수로 보여 줘"가 아니라 "LowerHex 트레이트의 fmt를 불러 줘"라는 뜻입니다.
// 표준 라이브러리는 LowerHex를 정수 타입에만 구현했으므로 실수는 {:x}로 출력할 수 없습니다.
// 고치는 방법: 실수의 비트 패턴을 보고 싶다면 정수로 바꿔서 출력합니다: {:x}에 ratio.to_bits()
//             (to_bits는 f64를 같은 비트의 u64로 바꾸는 메서드입니다).

fn main() {
    let ratio = 0.5;
    println!("{ratio:x}");
}
