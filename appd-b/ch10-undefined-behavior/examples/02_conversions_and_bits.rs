// Q10.4 · Q10.8 — 정수 승격, 부호가 다른 비교, 비트로 다른 타입 읽기
//
// C에서는 연산 전에 작은 정수가 int로 승격되고, 부호 있는 수와 없는 수를 비교하면 부호 있는 쪽이 부호 없는
// 수로 바뀝니다. 결과가 정해져 있어도 예상과 다르기 쉽습니다(compare/04_sign_compare.c: -1 < 1u는 거짓,
// unsigned char 200 + 100은 300). 유니언으로 float의 비트를 int로 읽는 일(type punning)도 C에서 흔합니다.
// Rust는 변환을 자동으로 하지 않고, 어떤 변환인지 코드에 적게 합니다.
//
//   1. 승격 대신: u8끼리 더하면 u8. 넓혀서 더하려면 u16::from으로 먼저 바꿈
//   2. 부호가 다른 비교: 둘 다 담을 수 있는 i64로 넓혀 비교하거나, as로 C처럼 바꾸거나, try_from으로 실패를 받음
//   3. 비트로 다른 타입 읽기: f32::to_bits / from_bits(안전한 함수)
//
// try_from의 NegOverflow, as가 값을 조용히 바꾸는 것, from_bits는 ch2 예제 04에서 봤고, 넓히는 변환과 좁히는
// 변환을 고르는 지침은 부록 A ch5에 있습니다. 여기서 더하는 것은 C의 정수 승격·부호 비교 규칙과 나란히 놓는
// 것입니다.
//
// 실행: cargo run -p faq-ch10-undefined-behavior --example 02_conversions_and_bits

fn main() {
    println!("1. 정수 승격 대신 명시적 확장");
    let (a, b): (u8, u8) = (200, 100);
    println!("  200u8.checked_add(100) = {:?}", a.checked_add(b));
    println!(
        "  u16::from(a) + u16::from(b) = {}",
        u16::from(a) + u16::from(b)
    );
    // 출력:   200u8.checked_add(100) = None
    // 출력:   u16::from(a) + u16::from(b) = 300

    println!("2. 부호가 다른 수의 비교");
    let (balance, limit): (i32, u32) = (-1, 1);
    println!(
        "  i64로 넓혀 비교: {}",
        i64::from(balance) < i64::from(limit)
    );
    println!(
        "  as u32로 C처럼: {} (balance as u32 = {})",
        (balance as u32) < limit,
        balance as u32
    );
    println!("  u32::try_from(balance) = {:?}", u32::try_from(balance));
    // 출력:   i64로 넓혀 비교: true
    // 출력:   as u32로 C처럼: false (balance as u32 = 4294967295)
    // 출력:   u32::try_from(balance) = Err(TryFromIntError(NegOverflow))

    println!("3. 비트로 다른 타입 읽기");
    let one = 1.0f32;
    println!("  1.0f32.to_bits() = {:#010x}", one.to_bits());
    println!(
        "  f32::from_bits(0x4049_0fdb) = {}",
        f32::from_bits(0x4049_0fdb)
    );
    // 출력:   1.0f32.to_bits() = 0x3f800000
    // 출력:   f32::from_bits(0x4049_0fdb) = 3.1415927
}
