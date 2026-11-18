// 오버플로 처리를 고르는 두 방법: 연산마다 메서드로 고르기(Part 1 ch1)와
// 값의 타입으로 고르기(Wrapping<T>, Saturating<T>).
//
// 실행: cargo run -p p1-ch14-ffi-any-misc --example 07_wrapping_saturating

use std::num::{Saturating, Wrapping};

const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

/// FNV-1a 32비트 해시를 메서드로 계산합니다. 곱셈마다 wrapping_mul을 적어야 합니다.
fn fnv1a_methods(bytes: &[u8]) -> u32 {
    let mut hash = FNV_OFFSET;
    for &b in bytes {
        hash ^= u32::from(b); // XOR은 넘치지 않으므로 그대로 씁니다.
        hash = hash.wrapping_mul(FNV_PRIME); // 그냥 *=를 쓰면 디버그 빌드에서 오버플로 패닉입니다.
    }
    hash
}

/// 같은 해시를 Wrapping<u32>로 계산합니다. 넘치면 한 바퀴 도는 것이 이 값의 규칙이므로
/// 연산자를 그대로 써도 됩니다.
fn fnv1a_wrapping(bytes: &[u8]) -> u32 {
    let mut hash = Wrapping(FNV_OFFSET);
    for &b in bytes {
        hash ^= u32::from(b); // Wrapping<u32> ^= u32 (BitXorAssign<u32>)
        hash *= FNV_PRIME; // Wrapping<u32> *= u32 (MulAssign<u32>): 래핑 곱셈
    }
    hash.0 // 안의 값은 튜플 필드 .0으로 꺼냅니다.
}

fn main() {
    println!("1. 메서드로 고르기: 연산마다 이름을 적습니다");
    let x = 250u8;
    println!(
        "    250u8 + 10 → checked {:?}, wrapping {}, saturating {}, overflowing {:?}",
        x.checked_add(10),
        x.wrapping_add(10),
        x.saturating_add(10),
        x.overflowing_add(10)
    );
    // 출력:     250u8 + 10 → checked None, wrapping 4, saturating 255, overflowing (4, true)

    println!("2. Wrapping<T>: 넘치면 한 바퀴 도는 것이 그 값의 규칙일 때");
    let w = Wrapping(250u8) + Wrapping(10);
    // 기본 형식뿐 아니라 디버그 형식도 감싼 모양 없이 안의 값만 보여 줍니다.
    println!("    Wrapping(250u8) + Wrapping(10) = {w} (디버그 형식 {w:?})");
    // 출력:     Wrapping(250u8) + Wrapping(10) = 4 (디버그 형식 4)
    println!(
        "    Wrapping(0u32) - Wrapping(1) = {}",
        Wrapping(0u32) - Wrapping(1)
    );
    // 출력:     Wrapping(0u32) - Wrapping(1) = 4294967295
    let text = "hello";
    println!(
        "    FNV-1a(\"{text}\"): 메서드 {:#010x}, Wrapping {:#010x}",
        fnv1a_methods(text.as_bytes()),
        fnv1a_wrapping(text.as_bytes())
    );
    // 출력:     FNV-1a("hello"): 메서드 0x4f9f2cab, Wrapping 0x4f9f2cab
    // 바이트를 모두 더해 256으로 나눈 나머지를 쓰는 간단한 체크섬: sum도 래핑으로 더합니다.
    let packet = [0x10u8, 0xF0, 0x80, 0x95];
    let checksum: Wrapping<u8> = packet.iter().copied().map(Wrapping).sum();
    println!("    체크섬 = {:#04x}", checksum.0); // 출력:     체크섬 = 0x15

    println!("3. Saturating<T>: 끝에 닿으면 멈추는 것이 그 값의 규칙일 때");
    let mut hp = Saturating(30u8); // 체력은 0 아래로, 255 위로 가지 않습니다.
    hp -= 50; // Saturating<u8> -= u8
    println!("    체력 30에서 50 피해: {hp}"); // 출력:     체력 30에서 50 피해: 0
    hp += 200;
    hp += 200;
    println!("    200씩 두 번 회복: {hp}"); // 출력:     200씩 두 번 회복: 255
    // 부호 있는 정수는 양쪽 끝에서 멈춥니다.
    let low = Saturating(-100i8) * Saturating(2);
    let high = Saturating(100i8) + Saturating(100);
    println!("    -100i8 * 2 = {low}, 100i8 + 100 = {high}"); // 출력:     -100i8 * 2 = -128, 100i8 + 100 = 127

    println!("4. 타입이 정해 주는 것과 정해 주지 않는 것");
    // 감싼 타입끼리, 또는 복합 대입(+=, -=, *= …)의 오른쪽에는 감싸지 않은 정수를 쓸 수 있습니다.
    // 하지만 w + 10u8처럼 감싸지 않은 정수와 + 하는 것은 구현되어 있지 않습니다(E0277, README 4.4절).
    let mut counter = Wrapping(u8::MAX);
    counter += 1;
    println!("    Wrapping(255u8) += 1 → {counter}"); // 출력:     Wrapping(255u8) += 1 → 0
    // 0으로 나누기는 래핑이나 포화로 정의할 수 없으므로 두 타입 모두 여전히 패닉합니다.
    // 나눗셈이 섞이면 checked_div나 NonZero(예제 08)를 씁니다.
    let divisor = Wrapping(0u8);
    let safe = counter.0.checked_div(divisor.0);
    println!("    checked_div로 0 나누기 피하기: {safe:?}"); // 출력:     checked_div로 0 나누기 피하기: None
}
