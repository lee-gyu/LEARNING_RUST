// Q7.3 — 정수의 폭을 고르는 언어
//
// Python의 정수는 크기 제한이 없고(Python 문서: "Integers have unlimited precision"), JavaScript의 수는
// 기본적으로 64비트 실수입니다. Rust의 정수는 타입마다 폭이 정해져 있어서, 값이 얼마나 커질지 보고
// 타입을 고릅니다. 타입을 적지 않은 정수 리터럴은 `i32`가 됩니다.
//
//   1. 타입을 적지 않으면 i32: 20억 + 20억이 i32에서는 오버플로
//   2. 계승(n!)을 담을 수 있는 한계: u32, u64, u128
//
// 타입을 적지 않은 리터럴이 i32라는 것과 MIN·MAX 상수는 Part 0 ch3 예제 03, 오버플로를 다루는 메서드
// 네 종류(`wrapping_*`, `checked_*`, `overflowing_*`, `saturating_*`)는 Part 0 ch3 예제 04, 2^53을 넘는 정수는
// f64로 바꿀 때 값이 바뀔 수 있다는 것은 Part 1 ch6 예제 02에서 봤습니다. 이 예제는 `size_of_val`로 크기를
// 직접 보고, 20억 + 20억과 n!이라는 구체적인 경계를 더합니다.
//
// 실행: cargo run -p faq-ch7-syntax-type-surprises --example 02_integer_widths

/// 1부터 곱해 가다가 처음 오버플로가 나는 n을 찾습니다. `T`마다 따로 쓰지 않으려고 곱셈 함수를 받습니다.
fn first_overflow<T: Copy>(
    one: T,
    checked_mul: impl Fn(T, T) -> Option<T>,
    from_n: impl Fn(u32) -> T,
) -> u32 {
    let mut acc = one;
    let mut n = 1;
    loop {
        match checked_mul(acc, from_n(n)) {
            Some(next) => acc = next,
            None => return n,
        }
        n += 1;
    }
}

fn main() {
    println!("1. 타입을 적지 않은 정수는 i32");
    let a = 2_000_000_000; // 타입을 적지 않았고 다른 곳에서도 정해지지 않으므로 i32
    println!("  크기: {}바이트, i32::MAX = {}", size_of_val(&a), i32::MAX);
    println!(
        "  i32: 20억 + 20억 = {:?}",
        2_000_000_000_i32.checked_add(2_000_000_000)
    );
    println!(
        "  i64: 20억 + 20억 = {:?}",
        2_000_000_000_i64.checked_add(2_000_000_000)
    );
    // 출력:   크기: 4바이트, i32::MAX = 2147483647
    // 출력:   i32: 20억 + 20억 = None
    // 출력:   i64: 20억 + 20억 = Some(4000000000)
    // (a.checked_add(a)처럼 타입이 정해지지 않은 리터럴에 메서드를 부르면 E0689가 납니다. Part 1 ch1 4.1절)

    println!("2. n!에서 처음 오버플로가 나는 n");
    let n32 = first_overflow(1u32, u32::checked_mul, u32::from);
    let n64 = first_overflow(1u64, u64::checked_mul, u64::from);
    let n128 = first_overflow(1u128, u128::checked_mul, u128::from);
    println!("  u32: {n32}!, u64: {n64}!, u128: {n128}!");
    // 출력:   u32: 13!, u64: 21!, u128: 35!
}
