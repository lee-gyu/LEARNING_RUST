// NonZero<T>: "0이 아니다"를 타입으로 보장하는 정수.
// 만들 때 한 번 검사하는 이유, 니치 덕분에 Option이 공짜인 것, 0으로 나누기가 사라지는 나눗셈,
// 그리고 정수 파싱 에러(ParseIntError, IntErrorKind)를 보여 줍니다.
//
// 실행: cargo run -p p1-ch14-ffi-any-misc --example 08_nonzero

use std::num::{IntErrorKind, NonZero};

// const의 값은 컴파일할 때 계산되므로, 여기서 unwrap이 실패하면 컴파일 에러(E0080)가 됩니다.
// 0을 넣어 보면 README 4.5절의 에러를 볼 수 있습니다.
const WORKERS: NonZero<usize> = NonZero::new(4).unwrap();

/// count가 0일 수 없으므로 이 나눗셈은 패닉하지 않습니다.
fn average(total: u32, count: NonZero<u32>) -> u32 {
    total / count // u32 / NonZero<u32> (Div<NonZero<u32>> for u32)
}

/// 파싱 실패 이유를 한국어로 바꿉니다.
fn reason(kind: &IntErrorKind) -> &'static str {
    match kind {
        IntErrorKind::Empty => "빈 문자열",
        IntErrorKind::InvalidDigit => "숫자가 아닌 글자",
        IntErrorKind::PosOverflow => "너무 큼",
        IntErrorKind::NegOverflow => "너무 작음",
        IntErrorKind::Zero => "0은 안 됨",
        // IntErrorKind는 #[non_exhaustive]라서 다른 크레이트에서는 _ 갈래가 반드시 필요합니다.
        // 나중에 배리언트가 늘어나도 이 코드가 깨지지 않게 하려는 표시입니다.
        _ => "그 밖의 이유",
    }
}

fn main() {
    println!("1. 만들 때 한 번 검사합니다");
    // NonZero::new는 0이면 None입니다. 실패할 수 있으니 Option을 돌려줍니다.
    println!(
        "    NonZero::new(5u32) = {:?}, NonZero::new(0u32) = {:?}",
        NonZero::new(5u32),
        NonZero::new(0u32)
    );
    // 출력:     NonZero::new(5u32) = Some(5), NonZero::new(0u32) = None
    println!(
        "    NonZero::<u32>::try_from(0) = {:?}",
        NonZero::<u32>::try_from(0)
    );
    // 출력:     NonZero::<u32>::try_from(0) = Err(TryFromIntError(Zero))
    // 부호 있는 정수도 됩니다. 음수는 0이 아니므로 괜찮습니다.
    let minus = NonZero::new(-3i32).unwrap();
    // 한 번 만든 뒤에는 get()으로 원래 정수를 꺼냅니다. 다시 검사할 필요가 없습니다.
    println!(
        "    get() = {}, WORKERS = {WORKERS}, NonZero::<u8>::MIN = {}",
        minus.get(),
        NonZero::<u8>::MIN
    );
    // 출력:     get() = -3, WORKERS = 4, NonZero::<u8>::MIN = 1

    println!("2. 니치: Option<NonZero<T>>는 T와 크기가 같습니다 (문서가 보장)");
    println!(
        "    NonZero<u32> {}, Option<NonZero<u32>> {}, Option<u32> {}",
        size_of::<NonZero<u32>>(),
        size_of::<Option<NonZero<u32>>>(),
        size_of::<Option<u32>>()
    );
    // 출력:     NonZero<u32> 4, Option<NonZero<u32>> 4, Option<u32> 8
    // 0이라는 비트 조합을 쓰지 않으므로 None을 0으로 적으면 태그 칸이 필요 없습니다(Part 1 ch7).
    let ids: Vec<Option<NonZero<u32>>> = vec![None; 1000];
    let plain: Vec<Option<u32>> = vec![None; 1000];
    println!(
        "    원소 1000개의 데이터 크기: Option<NonZero<u32>> {}바이트, Option<u32> {}바이트",
        size_of_val(ids.as_slice()),
        size_of_val(plain.as_slice())
    );
    // 출력:     원소 1000개의 데이터 크기: Option<NonZero<u32>> 4000바이트, Option<u32> 8000바이트

    println!("3. 0으로 나누기가 사라지는 나눗셈");
    let scores = [70u32, 85, 90];
    // 길이가 0일 수 있는 값은 경계에서 한 번 NonZero로 바꿉니다. 0인 경우는 그 자리에서 처리합니다.
    match NonZero::new(scores.len() as u32) {
        Some(count) => println!("    평균 = {}", average(scores.iter().sum(), count)),
        None => println!("    점수가 없습니다"),
    }
    // 출력:     평균 = 81
    let empty: [u32; 0] = [];
    match NonZero::new(empty.len() as u32) {
        Some(count) => println!("    평균 = {}", average(0, count)),
        None => println!("    빈 배열: 점수가 없습니다"),
    }
    // 출력:     빈 배열: 점수가 없습니다
    // 0이 아닌 값에 양수를 더해도 0이 아니므로 결과도 NonZero입니다. 넘칠 때만 None입니다.
    let n = NonZero::new(250u8).unwrap();
    println!(
        "    250 checked_add(5) = {:?}, checked_add(6) = {:?}, saturating_add(100) = {}",
        n.checked_add(5),
        n.checked_add(6),
        n.saturating_add(100)
    );
    // 출력:     250 checked_add(5) = Some(255), checked_add(6) = None, saturating_add(100) = 255

    println!("4. 문자열에서 읽기: ParseIntError와 IntErrorKind");
    for input in ["8", "0", "", "12a", "-1", "99999999999"] {
        match input.parse::<NonZero<u32>>() {
            Ok(v) => println!("    {input:?} → Ok({v})"),
            Err(e) => println!("    {input:?} → {:?}: {} ({e})", e.kind(), reason(e.kind())),
        }
    }
    // 출력:     "8" → Ok(8)
    // 출력:     "0" → Zero: 0은 안 됨 (number would be zero for non-zero type)
    // 출력:     "" → Empty: 빈 문자열 (cannot parse integer from empty string)
    // 출력:     "12a" → InvalidDigit: 숫자가 아닌 글자 (invalid digit found in string)
    // 출력:     "-1" → InvalidDigit: 숫자가 아닌 글자 (invalid digit found in string)
    // 출력:     "99999999999" → PosOverflow: 너무 큼 (number too large to fit in target type)
    // 부호 있는 타입에서는 너무 작은 값이 NegOverflow입니다.
    let e = "-200".parse::<i8>().unwrap_err();
    println!("    \"-200\" as i8 → {:?}: {}", e.kind(), reason(e.kind()));
    // 출력:     "-200" as i8 → NegOverflow: 너무 작음
}
