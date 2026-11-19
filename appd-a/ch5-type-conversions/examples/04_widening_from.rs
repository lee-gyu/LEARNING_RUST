// 지침 4: 넓히는 변환도 `as` 대신 `from`으로 씁니다.
//
// `as`는 변환의 방향을 따지지 않습니다. 오늘은 넓히는 변환인 `x as u32`가, 나중에 `x`의 타입이
// 넓어지면 아무 알림 없이 좁히는 변환이 됩니다. `u32::from(x)`는 그때 컴파일을 멈춥니다.
// 글 조회 수를 `u16`으로 세던 시절(v1)과 `u64`로 넓힌 뒤(v2)를 나란히 둡니다. 통계 차트는 `u32`를
// 받으므로 두 버전 모두 조회 수를 `u32`로 바꾸는 줄이 있습니다.
// 비권장: `views as u32` — v1에서는 맞지만, v2에서는 같은 줄이 조용히 값을 자릅니다.
// 권장:   `u32::from(views)` — v2로 바꾸는 순간 컴파일 에러(E0277)가 나서 `try_from`으로 고치게 됩니다.
//
// 실행: cargo run -p er-ch5-type-conversions --example 04_widening_from

mod v1 {
    pub struct PageStats {
        pub views: u16,
    }

    // 비권장. 지금은 u16 → u32라서 결과가 맞습니다.
    pub fn chart_value_as(s: &PageStats) -> u32 {
        s.views as u32
    }

    // 권장. 같은 결과이고, 손실이 없다는 사실을 컴파일러가 보장합니다.
    pub fn chart_value(s: &PageStats) -> u32 {
        u32::from(s.views)
    }
}

mod v2 {
    use std::num::TryFromIntError;

    // 조회 수가 u16을 넘기 시작해서 필드를 u64로 넓혔습니다.
    pub struct PageStats {
        pub views: u64,
    }

    // 비권장. v1에서 그대로 가져온 줄입니다. 경고 없이 컴파일되지만 이제는 좁히는 변환입니다.
    pub fn chart_value_as(s: &PageStats) -> u32 {
        s.views as u32
    }

    // 권장. v1의 `u32::from(s.views)`를 그대로 두면 컴파일되지 않습니다.
    //   error[E0277]: the trait bound `u32: From<u64>` is not satisfied
    // 에러가 가리킨 자리에서 실패를 처리하도록 고친 결과입니다.
    pub fn chart_value(s: &PageStats) -> Result<u32, TryFromIntError> {
        u32::try_from(s.views)
    }
}

fn main() {
    println!("1. v1(u16): 두 방법의 결과가 같습니다");
    let old = v1::PageStats { views: 50_000 };
    println!("    as   → {}", v1::chart_value_as(&old)); // 출력:     as   → 50000
    println!("    from → {}", v1::chart_value(&old)); // 출력:     from → 50000

    println!("2. v2(u64): 같은 as 줄이 값을 자릅니다");
    let new = v2::PageStats {
        views: 4_294_968_530, // u32::MAX보다 1235 큼
    };
    println!("    as       → {}", v2::chart_value_as(&new)); // 출력:     as       → 1234
    println!("    try_from → {:?}", v2::chart_value(&new)); // 출력:     try_from → Err(TryFromIntError(PosOverflow))

    println!("3. 부호가 바뀌는 변환은 넓어 보여도 넓히는 변환이 아닙니다");
    // 하루 사이 조회 수 증감입니다. i32 → u64는 비트 수가 늘지만 음수를 담을 수 없습니다.
    let delta: i32 = -3;
    println!("    as u64        → {}", delta as u64); // 출력:     as u64        → 18446744073709551613
    // `u64::from(delta)`는 없으므로(E0277) 컴파일러가 처음부터 try_from을 고르게 합니다.
    println!("    u64::try_from → {:?}", u64::try_from(delta)); // 출력:     u64::try_from → Err(TryFromIntError(NegOverflow))
    // 부호를 버리고 크기만 원했다면 그 뜻을 가진 메서드를 씁니다.
    println!("    unsigned_abs  → {}", delta.unsigned_abs()); // 출력:     unsigned_abs  → 3

    println!("4. usize: 플랫폼마다 크기가 달라 From이 적습니다");
    let small: u16 = 640;
    let page: u32 = 70_000;
    // usize는 16비트 이상이라는 것만 보장되므로 From은 u16까지만 있습니다.
    println!("    usize::from(u16)     = {}", usize::from(small)); // 출력:     usize::from(u16)     = 640
    // u32 → usize는 이 컴퓨터(64비트)에서는 언제나 성공하지만 타입으로는 보장되지 않습니다.
    println!("    usize::try_from(u32) = {:?}", usize::try_from(page)); // 출력:     usize::try_from(u32) = Ok(70000)
}
