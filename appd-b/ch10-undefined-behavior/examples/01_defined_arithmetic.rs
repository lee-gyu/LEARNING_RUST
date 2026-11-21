// Q10.3 · Q10.5 — C에서 미정의 동작인 연산이 Rust에서는 무엇이 되는가
//
// C에서 폭 이상으로 시프트하기(compare/02_shift_width.c), 0으로 나누기와 INT_MIN / -1(compare/03_division.c)은
// 미정의 동작입니다. Rust에서는 같은 연산의 결과가 정해져 있고, 원하는 결과를 메서드 이름으로 고를 수 있습니다.
//
//   1. 시프트: checked_shl은 폭 이상이면 None, wrapping_shl은 시프트 양을 폭으로 나눈 나머지만큼 밀기
//   2. 나눗셈: checked_div는 0으로 나누기와 MIN / -1에서 None, wrapping_div와 overflowing_div는 MIN을 돌려줌
//
// 연산자를 그대로 쓰면 어떻게 되는지는 README Q10.5에 적었습니다(시프트는 디버그에서 패닉·릴리스에서 위와 같은
// 래핑, 나눗셈은 빌드와 상관없이 패닉). 덧셈 오버플로의 같은 메서드들은 Part 0 ch3 예제 04, checked_div(0)은
// Part 0 ch6 예제 03과 Part 1 ch1 예제 04, 0으로 나눈 패닉을 catch_unwind로 잡는 것은 Part 1 ch8 예제 06에서
// 봤습니다. 여기서 더하는 것은 시프트와 MIN / -1을 C의 미정의 동작과 나란히 놓는 것입니다.
//
// 실행: cargo run -p faq-ch10-undefined-behavior --example 01_defined_arithmetic

fn main() {
    println!("1. 시프트");
    println!("  1u32.checked_shl(31) = {:?}", 1u32.checked_shl(31));
    println!("  1u32.checked_shl(32) = {:?}", 1u32.checked_shl(32));
    println!("  1u32.wrapping_shl(32) = {}", 1u32.wrapping_shl(32));
    println!("  1u32.wrapping_shl(33) = {}", 1u32.wrapping_shl(33));
    // 출력:   1u32.checked_shl(31) = Some(2147483648)
    // 출력:   1u32.checked_shl(32) = None
    // 출력:   1u32.wrapping_shl(32) = 1
    // 출력:   1u32.wrapping_shl(33) = 2
    // wrapping_shl은 시프트 양을 32로 나눈 나머지(32 → 0, 33 → 1)만큼 밉니다. 릴리스 빌드의 <<도 이렇게 동작했습니다.
    // 폭 이상이면 0을 원하면 unbounded_shl, 언제나 패닉하길 원하면 strict_shl을 씁니다.

    println!("2. 나눗셈");
    println!("  10i32.checked_div(0) = {:?}", 10i32.checked_div(0));
    println!(
        "  i32::MIN.checked_div(-1) = {:?}",
        i32::MIN.checked_div(-1)
    );
    println!(
        "  i32::MIN.wrapping_div(-1) = {}",
        i32::MIN.wrapping_div(-1)
    );
    println!(
        "  i32::MIN.overflowing_div(-1) = {:?}",
        i32::MIN.overflowing_div(-1)
    );
    // 출력:   10i32.checked_div(0) = None
    // 출력:   i32::MIN.checked_div(-1) = None
    // 출력:   i32::MIN.wrapping_div(-1) = -2147483648
    // 출력:   i32::MIN.overflowing_div(-1) = (-2147483648, true)
    // 0으로 나누기에는 wrapping_div가 정할 결과가 없으므로, wrapping_div(0)도 패닉합니다.
}
