// 이 예제가 보여 주는 것: 정수 타입의 연관 상수와 메서드 계열
//   연관 상수, 오버플로 처리 계열(checked/wrapping/saturating/overflowing), 수학 메서드,
//   비트 조작, 바이트 변환, 진법 변환, `as`의 잘림
// 실행: cargo run -p p1-ch1-std-overview --example 04_integers

fn main() {
    // ── 1. 연관 상수(associated constant) ──────────────────────────────────
    // `타입::상수` 모양입니다. `std::i32::MAX` 같은 모듈 상수는 예전 방식이라 문서에 "Deprecation planned"로 표시됩니다.
    println!("u8: {} ~ {}", u8::MIN, u8::MAX); // 출력: u8: 0 ~ 255
    println!("i64::MAX = {}", i64::MAX); // 출력: i64::MAX = 9223372036854775807
    println!("u128::MAX = {}", u128::MAX); // 출력: u128::MAX = 340282366920938463463374607431768211455
    println!("i32::BITS = {}, usize::BITS = {}", i32::BITS, usize::BITS); // 출력: i32::BITS = 32, usize::BITS = 64  (64비트 환경 기준)

    // ── 2. 오버플로 처리 계열은 덧셈만이 아닙니다 ─────────────────────────────
    // Part 0 ch3에서 본 `*_add` 네 가지가 뺄셈, 곱셈, 나눗셈, 거듭제곱, 부호 반전, 절댓값에도 있습니다.
    // 부호 없는 정수의 뺄셈은 결과가 음수가 되는 순간 오버플로입니다.
    let stock: u8 = 3;
    println!("checked_sub     = {:?}", stock.checked_sub(5)); // 출력: checked_sub     = None
    println!("wrapping_sub    = {}", stock.wrapping_sub(5)); // 출력: wrapping_sub    = 254
    println!("saturating_sub  = {}", stock.saturating_sub(5)); // 출력: saturating_sub  = 0
    println!("overflowing_sub = {:?}", stock.overflowing_sub(5)); // 출력: overflowing_sub = (254, true)

    // `i32::MIN`(-2147483648)의 절댓값은 `i32`에 들어가지 않습니다(최댓값이 2147483647).
    println!("{:?}", i32::MIN.checked_abs()); // 출력: None
    println!("{}", i32::MIN.unsigned_abs()); // 출력: 2147483648  (u32로 돌려주므로 담을 수 있음)

    // 0으로 나누기도 `checked_div`라면 패닉 대신 None입니다.
    println!("{:?} {:?}", 7_i32.checked_div(0), 2_i32.checked_pow(31)); // 출력: None None
    println!("{}", 2_i32.saturating_pow(31)); // 출력: 2147483647

    // `Option`은 Part 0에서 배운 `match`나 `unwrap_or`로 꺼냅니다.
    // (`map`, `and_then` 같은 `Option` 콤비네이터는 `ch2-option-result`에서 다룹니다.)
    let price: u32 = 4_000_000_000;
    let total = match price.checked_mul(2) {
        Some(t) => t,
        None => {
            println!("합계가 u32 범위를 넘었습니다"); // 출력: 합계가 u32 범위를 넘었습니다
            u32::MAX
        }
    };
    println!("total = {total}"); // 출력: total = 4294967295
    println!("{}", 250_u8.checked_add(10).unwrap_or(0)); // 출력: 0

    // ── 3. 수학 메서드 ───────────────────────────────────────────────────
    println!("pow = {}, signum = {}", 3_i32.pow(4), (-8_i32).signum()); // 출력: pow = 81, signum = -1
    // `abs_diff`는 차이를 부호 없는 타입으로 돌려줍니다. `a - b`처럼 빼다가 넘칠 걱정이 없습니다.
    println!(
        "abs_diff = {}, {}",
        3_u8.abs_diff(250),
        (-3_i32).abs_diff(5)
    ); // 출력: abs_diff = 247, 8

    // `%`는 나머지의 부호가 왼쪽 피연산자를 따릅니다. 항상 0 이상인 나머지가 필요하면 `rem_euclid`를 씁니다.
    // 예: 0시에서 30시간 전은 이틀 전 18시입니다.
    let hours = -30_i32;
    println!("-30 % 24 = {}", hours % 24); // 출력: -30 % 24 = -6
    println!("(-30).rem_euclid(24) = {}", hours.rem_euclid(24)); // 출력: (-30).rem_euclid(24) = 18
    println!("(-30).div_euclid(24) = {}", hours.div_euclid(24)); // 출력: (-30).div_euclid(24) = -2

    println!("isqrt(99) = {}", 99_u32.isqrt()); // 출력: isqrt(99) = 9
    println!("ilog10(12345) = {}", 12345_u32.ilog10()); // 출력: ilog10(12345) = 4  (자릿수 - 1)
    println!("midpoint = {}", u8::MAX.midpoint(253)); // 출력: midpoint = 254  ((255 + 253) / 2를 넘치지 않고 계산)

    // 같은 계열이라도 부호 있는/없는 타입에 따라 있는 메서드가 다릅니다. 문서에서 타입별로 확인합니다.
    // `is_power_of_two`는 부호 없는 정수에만 있고, `div_ceil`은 `u32`에서는 안정(1.73.0),
    // `i32`에서는 아직 nightly 전용 실험 API입니다.
    println!("{} {}", 64_u32.is_power_of_two(), 7_u32.div_ceil(2)); // 출력: true 4

    // ── 4. 비트 조작 ─────────────────────────────────────────────────────
    // `{:08b}`는 2진수로, 8자리가 되도록 앞을 0으로 채워 출력하는 서식입니다(서식 문법은 `ch3-strings-fmt`).
    let flags: u8 = 0b1011_0000;
    println!("flags = {flags:08b}"); // 출력: flags = 10110000
    println!(
        "count_ones = {}, leading_zeros = {}, trailing_zeros = {}",
        flags.count_ones(),
        flags.leading_zeros(),
        flags.trailing_zeros()
    ); // 출력: count_ones = 3, leading_zeros = 0, trailing_zeros = 4
    println!("rotate_left(2) = {:08b}", flags.rotate_left(2)); // 출력: rotate_left(2) = 11000010
    println!("<< 2           = {:08b}", flags << 2); // 출력: << 2           = 11000000  (밀려난 비트는 버려짐)
    println!("reverse_bits   = {:08b}", flags.reverse_bits()); // 출력: reverse_bits   = 00001101

    // ── 5. 바이트 변환 ───────────────────────────────────────────────────
    // 파일이나 네트워크에서 정수를 바이트로 주고받을 때 씁니다. be = 빅 엔디언, le = 리틀 엔디언.
    // `{:02x?}`는 디버그 형식을 16진수 두 자리로 보여 주는 서식입니다.
    let n: u32 = 0x1234_5678;
    println!("to_be_bytes = {:02x?}", n.to_be_bytes()); // 출력: to_be_bytes = [12, 34, 56, 78]
    println!("to_le_bytes = {:02x?}", n.to_le_bytes()); // 출력: to_le_bytes = [78, 56, 34, 12]
    let back = u32::from_le_bytes([0x78, 0x56, 0x34, 0x12]);
    println!("from_le_bytes = {back:#x}"); // 출력: from_le_bytes = 0x12345678
    // `to_ne_bytes`(ne = native)는 실행하는 CPU의 엔디언을 따르므로 결과가 플랫폼마다 다를 수 있습니다.

    // ── 6. 진법 변환 ─────────────────────────────────────────────────────
    // 문서: `pub const fn from_str_radix(src: &str, radix: u32) -> Result<i32, ParseIntError>`
    println!("{:?}", i32::from_str_radix("ff", 16)); // 출력: Ok(255)
    println!("{:?}", u8::from_str_radix("1010", 2)); // 출력: Ok(10)
    println!("{:?}", u8::from_str_radix("12", 2)); // 출력: Err(ParseIntError { kind: InvalidDigit })  (2진수에 2는 없음)
    // 10진수라면 `from_str_radix(s, 10)` 대신 `parse`를 씁니다(`ch3-strings-fmt`). Clippy도 그렇게 권합니다.

    // ── 7. as 변환은 조용히 잘라 냅니다 ────────────────────────────────────
    // 넓은 타입 → 좁은 타입으로 `as` 변환하면 넘치는 위쪽 비트를 버립니다. 에러도 패닉도 없습니다.
    println!("300 as u8 = {}", 300_i32 as u8); // 출력: 300 as u8 = 44  (300 - 256)
    println!("-1 as u8 = {}", -1_i32 as u8); // 출력: -1 as u8 = 255
    println!("3_000_000_000 as i32 = {}", 3_000_000_000_u32 as i32); // 출력: 3_000_000_000 as i32 = -1294967296
    // 값이 바뀌면 안 되는 변환에는 `try_from`을 씁니다. 범위를 벗어나면 `Err`가 됩니다.
    // (`From`/`TryFrom` 트레이트는 `ch6-core-traits`에서 자세히 다룹니다.)
    println!("{:?} {:?}", u8::try_from(44_i32), u8::try_from(300_i32)); // 출력: Ok(44) Err(TryFromIntError(PosOverflow))
}
