// 지침 1: 함수나 제네릭으로 충분하면 매크로를 쓰지 않습니다.
//
// 반복을 없애는 도구는 사다리처럼 고릅니다.
//   1칸 함수   — 같은 일을 같은 타입에 되풀이할 때
//   2칸 제네릭 — 같은 일을 트레이트 바운드를 공유하는 여러 타입에 되풀이할 때
//   3칸 매크로 — 타입으로는 묶이지 않고 "코드의 모양"만 같을 때
// 아래 칸으로 충분한데 매크로를 쓰면 타입 검사가 펼친 뒤로 미뤄지고, 값처럼 넘길 수도 없습니다.
//
// 실행: cargo run -p er-ch28-macros-judiciously --example 01_abstraction_ladder

// 1칸: 함수. f64 조각의 평균만 필요하면 이것으로 끝입니다.
fn mean_f64(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

// 2칸: 제네릭. "f64로 손실 없이 바뀌는 타입"이라는 바운드가 받을 수 있는 타입을 정확히 말해 줍니다.
// u8, i32, f32는 Into<f64>를 구현하지만 i64, u64는 구현하지 않습니다(f64로 바꾸면 값을 잃을 수 있어서).
fn mean<T: Copy + Into<f64>>(xs: &[T]) -> f64 {
    xs.iter().map(|&x| x.into()).sum::<f64>() / xs.len() as f64
}

// 비권장: 여러 타입에 쓰고 싶어서 2칸 대신 매크로로 썼습니다.
// 바운드가 없으니 `as f64`가 받아 주는 타입이면 무엇이든 받습니다. u64처럼 값을 잃는 타입도 조용히 통과합니다.
macro_rules! mean {
    ($($x:expr),+ $(,)?) => {{
        let xs = [$($x as f64),+];
        xs.iter().sum::<f64>() / xs.len() as f64
    }};
}

// 3칸: 매크로가 필요한 경우.
// 정수 타입마다 MIN과 MAX가 있지만 그 둘을 묶어 주는 표준 트레이트는 없습니다.
// 그래서 `impl<T> Limits for T { const LOW: T = T::MIN; ... }` 같은 제네릭 구현은 쓸 수 없고,
// 타입 이름만 다른 impl 블록을 여섯 번 손으로 써야 합니다. 이 "타입 이름 자리만 다른 반복"이 매크로의 몫입니다.
trait Limits: Sized {
    const LOW: Self;
    const HIGH: Self;
}

macro_rules! impl_limits {
    ($($t:ty)*) => {
        $(
            impl Limits for $t {
                const LOW: Self = <$t>::MIN;
                const HIGH: Self = <$t>::MAX;
            }
        )*
    };
}

impl_limits!(i8 u8 i16 u16 i32 u32);

// 매크로가 만든 구현을 다시 제네릭 함수가 씁니다. 매크로는 사다리의 꼭대기에서 아래 칸을 돕는 데 그칩니다.
// i64를 T로 바꾸되, 범위를 벗어나면 가장 가까운 끝값으로 자릅니다.
fn saturate<T>(v: i64) -> T
where
    T: Limits + TryFrom<i64> + Into<i64> + Copy,
{
    match T::try_from(v) {
        Ok(x) => x,
        Err(_) if v < T::LOW.into() => T::LOW,
        Err(_) => T::HIGH,
    }
}

fn main() {
    println!("1. 함수: 한 타입");
    let m = mean_f64(&[1.5, 2.5, 3.5]);
    println!("  mean_f64([1.5, 2.5, 3.5]) = {m}"); // 출력:   mean_f64([1.5, 2.5, 3.5]) = 2.5

    println!("2. 제네릭: 바운드를 공유하는 여러 타입");
    println!("  mean(&[1u8, 2, 6]) = {}", mean(&[1u8, 2, 6])); // 출력:   mean(&[1u8, 2, 6]) = 3
    println!("  mean(&[10i32, -20, 40]) = {}", mean(&[10i32, -20, 40])); // 출력:   mean(&[10i32, -20, 40]) = 10
    println!("  mean(&[0.5f32, 1.5]) = {}", mean(&[0.5f32, 1.5])); // 출력:   mean(&[0.5f32, 1.5]) = 1

    // 함수는 값이라서 다른 함수에 넘길 수 있습니다. 매크로는 `map(mean!)`처럼 넘길 수 없습니다.
    let groups: [&[i32]; 2] = [&[1, 2, 3], &[10, 20]];
    let means: Vec<f64> = groups.into_iter().map(mean::<i32>).collect();
    println!("  map(mean::<i32>) = {means:?}"); // 출력:   map(mean::<i32>) = [2.0, 15.0]

    // 2^53 + 1은 f64로 정확히 나타낼 수 없는 가장 작은 양의 정수입니다.
    // 제네릭 판은 `mean(&[big, 1])`을 컴파일하지 않습니다(E0277 `f64: From<u64>` 없음 — u64는 Into<f64>를 구현하지 않음).
    // 매크로 판은 받아들이고, `as`가 조용히 반올림한 결과를 돌려줍니다.
    let big: u64 = (1 << 53) + 1;
    println!("  비권장 mean!(2^53 + 1, 1) = {}", mean!(big, 1)); // 출력:   비권장 mean!(2^53 + 1, 1) = 4503599627370496
    let exact = [big, 1].iter().sum::<u64>() / 2; // 합이 짝수라 정수 나눗셈이 정확합니다.
    println!("  정확한 평균                = {exact}"); // 출력:   정확한 평균                = 4503599627370497

    println!("3. 매크로: 타입 이름만 다른 impl 블록");
    println!("  saturate::<u8>(300)    = {}", saturate::<u8>(300)); // 출력:   saturate::<u8>(300)    = 255
    println!("  saturate::<i8>(-200)   = {}", saturate::<i8>(-200)); // 출력:   saturate::<i8>(-200)   = -128
    println!("  saturate::<u16>(70000) = {}", saturate::<u16>(70000)); // 출력:   saturate::<u16>(70000) = 65535
    println!("  saturate::<i32>(42)    = {}", saturate::<i32>(42)); // 출력:   saturate::<i32>(42)    = 42
}
