// 이 예제가 보여 주는 것: 실패할 수 있는 변환 TryFrom/TryInto, 그리고 as·From·TryFrom 중 무엇을 고를지.
//                        직접 만든 검증 타입에 TryFrom 구현하기
// 실행: cargo run -p p1-ch6-core-traits --example 02_try_from_as
//
// From은 "언제나 성공하고 정보를 잃지 않는" 변환에만 구현합니다.
// 범위를 벗어날 수 있는 변환은 TryFrom으로 Result를 돌려주고, 실패를 어떻게 처리할지는 호출자가 정합니다.
// as는 실패를 알리지 않습니다. 넘치면 조용히 자르거나(정수) 끝값에 붙입니다(실수 → 정수).

// 1~12만 담을 수 있는 "월" 타입. 생성 경로를 TryFrom 하나로 두면 잘못된 값이 들어갈 수 없습니다.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Month(u8);

// 실패 이유를 담을 에러 타입. 여기서는 받은 값만 기억합니다.
// (에러 타입을 제대로 설계하는 법은 ch8-errors-panic에서 다룹니다.)
#[derive(Debug, PartialEq)]
struct InvalidMonth(i32);

impl TryFrom<i32> for Month {
    type Error = InvalidMonth; // 실패할 때 돌려줄 타입을 연관 타입으로 정합니다.

    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            // n이 1..=12이면 u8에 반드시 들어가므로 as가 안전합니다.
            1..=12 => Ok(Month(n as u8)),
            _ => Err(InvalidMonth(n)),
        }
    }
}

fn days_in(month: Month) -> u8 {
    match month.0 {
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn main() {
    // 1. as는 넘치는 값을 조용히 바꿉니다. TryFrom은 Err로 알려 줍니다.
    println!("== 1. as와 try_from ==");
    let values = [200_i32, 300, -1];
    for v in values {
        println!(
            "{v:>4}: as u8 = {:>3}, u8::try_from = {:?}",
            v as u8,
            u8::try_from(v)
        );
    }
    // 출력:
    //  200: as u8 = 200, u8::try_from = Ok(200)
    //  300: as u8 =  44, u8::try_from = Err(TryFromIntError(PosOverflow))
    //   -1: as u8 = 255, u8::try_from = Err(TryFromIntError(NegOverflow))

    // 2. 실수 → 정수의 as는 소수점을 버리고 범위 끝에 붙입니다(포화). NaN은 0이 됩니다.
    //    표준 라이브러리에는 실수 → 정수의 From/TryFrom이 없습니다. 반올림 방식을 직접 정하라는 뜻입니다.
    println!("\n== 2. 실수 → 정수의 as ==");
    let x = 300.7_f64;
    let nan: f64 = "NaN".parse().unwrap(); // 입력에서 NaN이 들어온 상황
    println!("{} {} {}", x as u8, -1.5_f64 as u8, nan as i32); // 출력: 255 0 0
    println!("반올림 후 변환: {}", x.round() as u16); // 출력: 반올림 후 변환: 301

    // 3. 정수 → 실수: i32는 f64에 언제나 정확히 들어가므로 From이 있습니다.
    //    i64는 f64의 정밀도(2^53)를 넘을 수 있어 From이 없고, as는 가장 가까운 값으로 바꿉니다.
    println!("\n== 3. 정수 → 실수 ==");
    println!("{}", f64::from(16_777_217_i32)); // 출력: 16777217
    let big = 2_i64.pow(53) + 1;
    println!("{big} as f64 = {}", big as f64); // 출력: 9007199254740993 as f64 = 9007199254740992

    // 4. 플랫폼에 따라 크기가 다른 usize. u32 → usize의 From은 없습니다(16비트 플랫폼을 고려).
    //    u16 → usize는 언제나 들어가므로 From이 있습니다.
    println!("\n== 4. usize ==");
    let short: u16 = 500;
    let index = usize::from(short);
    let id: u32 = 70_000;
    let checked = usize::try_from(id); // 64비트에서는 언제나 Ok이지만 타입이 Result라서 처리를 강제합니다.
    println!("{index} {checked:?}"); // 출력: 500 Ok(70000)

    // 5. 직접 구현한 TryFrom. try_into()는 포괄 구현으로 따라옵니다.
    println!("\n== 5. 검증 타입 Month ==");
    for n in [2, 9, 13, 0] {
        match Month::try_from(n) {
            Ok(m) => println!("{n}월: {}일까지", days_in(m)),
            Err(e) => println!("{n}: 거절 {e:?}"),
        }
    }
    // 출력:
    // 2월: 28일까지
    // 9월: 30일까지
    // 13: 거절 InvalidMonth(13)
    // 0: 거절 InvalidMonth(0)

    // try_into는 목적 타입을 알아야 합니다. 여기서는 변수의 타입 표기로 알려 줍니다.
    let m: Result<Month, _> = 12.try_into();
    println!("12.try_into() = {m:?}"); // 출력: 12.try_into() = Ok(Month(12))

    // 6. 글자와 정수 사이: char → u8, u32 → char도 실패할 수 있습니다.
    println!("\n== 6. char와 TryFrom ==");
    println!("{:?}", u8::try_from('A')); // 출력: Ok(65)
    println!("{:?}", u8::try_from('가')); // 출력: Err(TryFromCharError(()))
    println!("{:?}", char::try_from(0xAC00_u32)); // 출력: Ok('가')
    // 0xD800~0xDFFF는 UTF-16용으로 예약된 자리라 char가 될 수 없습니다.
    if let Err(e) = char::try_from(0xD800_u32) {
        println!("0xD800: {e}"); // 출력: 0xD800: converted integer out of range for `char`
    }
}
