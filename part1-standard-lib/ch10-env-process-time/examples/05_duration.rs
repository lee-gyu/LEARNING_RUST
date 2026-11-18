// Duration: 시간의 "길이"를 나타내는 타입입니다. 시각(언제)이 아니라 간격(얼마나)입니다.
//
//   1) 만들기: new, from_secs/millis/micros/nanos, from_mins/from_hours(1.91), from_secs_f64, ZERO, MAX
//   2) 단위로 꺼내기: as_secs(버림), subsec_*, as_millis(u128), as_secs_f64
//   3) 연산: Duration끼리 +, -, 비교, 합계(sum), 정수(u32) 곱셈·나눗셈, 실수 배율(mul_f64), 비율(div_duration_f64)
//   4) 넘칠 수 있는 연산: checked_*(Option), saturating_*(경계값에 멈춤), try_from_secs_f64(Result)
//   5) 출력: Display는 없고 Debug만 있습니다. 원하는 모양은 직접 만듭니다.
//
// 실행: cargo run -p p1-ch10-env-process-time --example 05_duration
//
// 시간을 재지 않으므로 출력은 실행할 때마다 같습니다.

use std::time::Duration;

// "시:분:초.밀리초" 모양으로 출력합니다. Duration에는 Display가 없으므로 직접 만듭니다.
fn hms(d: Duration) -> String {
    let total = d.as_secs();
    let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
    format!("{h:02}:{m:02}:{s:02}.{:03}", d.subsec_millis())
}

fn main() {
    println!("1. 만들기");
    let a = Duration::new(1, 500_000_000); // 1초 + 5억 나노초
    let b = Duration::from_millis(1500);
    println!("    new(1, 500_000_000) == from_millis(1500): {}", a == b); // 출력:     new(1, 500_000_000) == from_millis(1500): true
    println!(
        "    from_mins(90) == from_hours(1) + from_secs(1800): {}",
        Duration::from_mins(90) == Duration::from_hours(1) + Duration::from_secs(1800)
    ); // 출력:     from_mins(90) == from_hours(1) + from_secs(1800): true
    println!(
        "    from_secs_f64(0.25) = {:?}",
        Duration::from_secs_f64(0.25)
    ); // 출력:     from_secs_f64(0.25) = 250ms
    println!(
        "    ZERO = {:?}, default() = {:?}",
        Duration::ZERO,
        Duration::default()
    ); // 출력:     ZERO = 0ns, default() = 0ns
    println!("    MAX = {:?}", Duration::MAX); // 출력:     MAX = 18446744073709551615.999999999s

    println!("2. 단위로 꺼내기");
    let d = Duration::from_millis(2_750);
    // as_secs는 소수점 아래를 버립니다. 나머지는 subsec_*로 따로 꺼냅니다.
    println!(
        "    as_secs = {}, subsec_millis = {}",
        d.as_secs(),
        d.subsec_millis()
    ); // 출력:     as_secs = 2, subsec_millis = 750
    // as_millis/as_micros/as_nanos는 큰 값도 담도록 u128을 돌려줍니다.
    println!(
        "    as_millis = {}, as_nanos = {}",
        d.as_millis(),
        d.as_nanos()
    ); // 출력:     as_millis = 2750, as_nanos = 2750000000
    println!("    as_secs_f64 = {}", d.as_secs_f64()); // 출력:     as_secs_f64 = 2.75

    println!("3. 연산");
    let laps = [
        Duration::from_millis(1_200),
        Duration::from_millis(980),
        Duration::from_millis(1_050),
    ];
    let total: Duration = laps.iter().sum();
    println!("    합계 = {total:?}"); // 출력:     합계 = 3.23s
    // 정수 곱셈·나눗셈은 u32만 받습니다.
    println!("    평균 = {:?}", total / laps.len() as u32); // 출력:     평균 = 1.076666666s
    println!("    가장 긴 구간 = {:?}", laps.iter().max().unwrap()); // 출력:     가장 긴 구간 = 1.2s
    println!("    1.2s와 980ms의 차이 = {:?}", laps[0].abs_diff(laps[1])); // 출력:     1.2s와 980ms의 차이 = 220ms
    // 실수 배율은 연산자 대신 mul_f64/div_f64 메서드로 합니다.
    println!(
        "    250ms × 1.5 = {:?}",
        Duration::from_millis(250).mul_f64(1.5)
    ); // 출력:     250ms × 1.5 = 375ms
    // 두 Duration의 비율은 f64로 얻습니다.
    let ratio = Duration::from_secs(45).div_duration_f64(Duration::from_secs(60));
    println!("    45초는 1분의 {:.0}%", ratio * 100.0); // 출력:     45초는 1분의 75%

    println!("4. 넘칠 수 있는 연산");
    let short = Duration::from_millis(300);
    let long = Duration::from_secs(1);
    // Duration은 음수가 없습니다. short - long은 패닉하므로 결과를 확인하는 메서드를 씁니다.
    println!("    300ms.checked_sub(1s) = {:?}", short.checked_sub(long)); // 출력:     300ms.checked_sub(1s) = None
    println!(
        "    300ms.saturating_sub(1s) = {:?}",
        short.saturating_sub(long)
    ); // 출력:     300ms.saturating_sub(1s) = 0ns
    println!(
        "    MAX.checked_add(1ns) = {:?}",
        Duration::MAX.checked_add(Duration::from_nanos(1))
    ); // 출력:     MAX.checked_add(1ns) = None
    println!(
        "    MAX.checked_mul(2) = {:?}",
        Duration::MAX.checked_mul(2)
    ); // 출력:     MAX.checked_mul(2) = None
    // from_secs_f64는 음수·NaN·너무 큰 값에서 패닉합니다. 외부에서 받은 값이면 try_ 판을 씁니다.
    println!(
        "    try_from_secs_f64(-1.0) = {:?}",
        Duration::try_from_secs_f64(-1.0)
    ); // 출력:     try_from_secs_f64(-1.0) = Err(TryFromFloatSecsError { kind: Negative })
    match Duration::try_from_secs_f64(f64::NAN) {
        Ok(d) => println!("    NaN → {d:?}"),
        Err(e) => println!("    try_from_secs_f64(NaN) 에러: {e}"), // 출력:     try_from_secs_f64(NaN) 에러: cannot convert float seconds to Duration: value is either too big or NaN
    }

    println!("5. 출력");
    for d in [
        Duration::from_nanos(1_500),
        Duration::from_micros(2_500),
        Duration::from_millis(1_001),
        Duration::new(3_725, 42_000_000),
    ] {
        // {:?}는 알맞은 단위를 골라 소수점 아래를 필요한 만큼 보여 줍니다. {:.2?}는 소수점 둘째 자리까지입니다.
        println!(
            "    {:>10} | {:>8} | {}",
            format!("{d:?}"),
            format!("{d:.2?}"),
            hms(d)
        );
    }
    // 출력:          1.5µs |   1.50µs | 00:00:00.000
    // 출력:          2.5ms |   2.50ms | 00:00:00.002
    // 출력:         1.001s |    1.00s | 00:00:01.001
    // 출력:      3725.042s | 3725.04s | 01:02:05.042
}
