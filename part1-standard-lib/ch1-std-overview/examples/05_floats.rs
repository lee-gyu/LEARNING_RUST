// 이 예제가 보여 주는 것: 실수(f64) 타입의 연관 상수와 메서드
//   NaN과 비교, `total_cmp`, `EPSILON`을 이용한 근삿값 비교, 반올림 계열, 수학 상수, 실수 → 정수 `as` 변환
// 실행: cargo run -p p1-ch1-std-overview --example 05_floats

use std::f64::consts; // 수학 상수 모듈. `f64` 원시 타입 페이지가 아니라 `std::f64` 모듈 안에 있습니다.

fn main() {
    // ── 1. 연관 상수 ─────────────────────────────────────────────────────
    println!("f64::MAX = {:e}", f64::MAX); // 출력: f64::MAX = 1.7976931348623157e308
    println!("f64::EPSILON = {:e}", f64::EPSILON); // 출력: f64::EPSILON = 2.220446049250313e-16
    println!("{} {}", f64::INFINITY, f64::NEG_INFINITY); // 출력: inf -inf
    println!("PI = {}, TAU = {}", consts::PI, consts::TAU); // 출력: PI = 3.141592653589793, TAU = 6.283185307179586

    // ── 2. NaN은 자기 자신과도 같지 않습니다 ─────────────────────────────────
    // 음수의 제곱근이나 0.0 / 0.0처럼 수학적으로 정해지지 않는 계산의 결과가 NaN(Not a Number)입니다.
    let nan = (-1.0_f64).sqrt();
    // IEEE 754 표준이 "NaN은 어떤 값과도 같지 않다"고 정했기 때문에 `==`로는 NaN을 찾을 수 없습니다.
    // 같은 값끼리 비교하는 코드는 보통 실수라서 Clippy가 막지만(`eq_op`), 여기서는 그 사실을 보이려고 일부러 씁니다.
    #[expect(
        clippy::eq_op,
        reason = "NaN이 자기 자신과도 같지 않다는 것을 보이려는 비교"
    )]
    let equal_to_itself = nan == nan;
    println!("nan = {nan}, nan == nan ? {equal_to_itself}"); // 출력: nan = NaN, nan == nan ? false
    println!("nan.is_nan() = {}", nan.is_nan()); // 출력: nan.is_nan() = true

    // 그래서 `f64`는 `PartialOrd`만 구현하고 `Ord`는 구현하지 않습니다(문서의 Trait Implementations 목록 참고).
    // `partial_cmp`는 비교할 수 없으면 `None`을 돌려줍니다.
    println!(
        "{:?} {:?}",
        1.0_f64.partial_cmp(&2.0),
        1.0_f64.partial_cmp(&nan)
    ); // 출력: Some(Less) None

    // `total_cmp`는 모든 값(NaN 포함)에 순서를 정해 주는 비교입니다. 반환 타입이 `Option`이 아니라 `Ordering`입니다.
    // 문서: `pub fn total_cmp(&self, other: &f64) -> Ordering`
    // 순서: 음의 NaN < -inf < 음수 < -0.0 < 0.0 < 양수 < inf < 양의 NaN
    // `==`에서는 같은 -0.0과 0.0도 구별합니다. NaN은 부호에 따라 맨 앞이나 맨 뒤에 놓이는데,
    // 산술 연산이 돌려주는 NaN의 부호는 정해져 있지 않으므로(`f32` 문서의 "NaN bit patterns" 절) 여기서는 출력하지 않습니다.
    println!(
        "{:?} {:?} {}",
        1.0_f64.total_cmp(&2.0),
        (-0.0_f64).total_cmp(&0.0),
        -0.0_f64 == 0.0
    ); // 출력: Less Less true

    // `max`/`min`은 한쪽이 NaN이면 다른 쪽을 돌려줍니다.
    println!("max = {}, min = {}", nan.max(3.0), 2.0_f64.min(nan)); // 출력: max = 3, min = 2

    // ── 3. 근삿값 비교 ───────────────────────────────────────────────────
    // 0.1과 0.2는 2진수로 정확히 나타낼 수 없어서 더한 값이 0.3과 미세하게 다릅니다.
    // `: f64`를 빼면 아래 `.abs()`에서 E0689가 납니다. 메서드를 부르는 시점에 `f32`인지 `f64`인지 정해져 있어야 합니다.
    let sum: f64 = 0.1 + 0.2;
    println!("0.1 + 0.2 == 0.3 ? {}", sum == 0.3); // 출력: 0.1 + 0.2 == 0.3 ? false
    // 차이가 충분히 작은지로 비교합니다. `EPSILON`은 "1.0과 그다음 표현 가능한 수의 차이"이므로,
    // 1 근처의 값을 비교할 때 기준으로 쓸 수 있습니다. 값이 크면 허용 오차도 값의 크기에 비례해 키워야 합니다.
    println!("차이 < EPSILON ? {}", (sum - 0.3).abs() < f64::EPSILON); // 출력: 차이 < EPSILON ? true

    // ── 4. 반올림 계열 ───────────────────────────────────────────────────
    // 결과는 여전히 f64입니다. 정수가 필요하면 그 뒤에 `as`로 변환합니다.
    for x in [2.5_f64, -2.5, 2.7] {
        println!(
            "{x:>4}: floor {:>2} ceil {:>2} round {:>2} round_ties_even {:>2} trunc {:>2} fract {:.1}",
            x.floor(),
            x.ceil(),
            x.round(),
            x.round_ties_even(),
            x.trunc(),
            x.fract()
        );
    }
    // 출력:  2.5: floor  2 ceil  3 round  3 round_ties_even  2 trunc  2 fract 0.5
    // 출력: -2.5: floor -3 ceil -2 round -3 round_ties_even -2 trunc -2 fract -0.5
    // 출력:  2.7: floor  2 ceil  3 round  3 round_ties_even  3 trunc  2 fract 0.7
    // `round`는 0.5를 0에서 먼 쪽으로, `round_ties_even`은 짝수 쪽으로 보냅니다.

    // ── 5. 그 밖의 수학 메서드 ─────────────────────────────────────────────
    println!(
        "sqrt = {}, powi = {}, powf = {}",
        2.0_f64.sqrt(),
        2.0_f64.powi(10),
        8.0_f64.powf(1.0 / 3.0)
    ); // 출력: sqrt = 1.4142135623730951, powi = 1024, powf = 2
    println!("hypot = {}, abs = {}", 3.0_f64.hypot(4.0), (-1.5_f64).abs()); // 출력: hypot = 5, abs = 1.5
    println!("clamp = {}", 1.7_f64.clamp(0.0, 1.0)); // 출력: clamp = 1
    println!("sin(PI/2) = {}", consts::FRAC_PI_2.sin()); // 출력: sin(PI/2) = 1

    // ── 6. 실수 → 정수 as 변환 ─────────────────────────────────────────────
    // 소수점 아래는 0 쪽으로 버리고, 범위를 넘으면 그 타입의 최댓값/최솟값에 붙고, NaN은 0이 됩니다.
    println!(
        "{} {} {}",
        3.99_f64 as i32, -3.99_f64 as i32, 1e10_f64 as i32
    ); // 출력: 3 -3 2147483647
    println!("{} {}", -1.0_f64 as u8, nan as i32); // 출력: 0 0
}
