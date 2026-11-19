// 지침 1: 실패할 수 있는 변환은 `From`이 아니라 `TryFrom`으로 구현합니다.
//
// 분리수거 요일을 번호(1=월 … 7=일)로 받아 `Weekday`로 바꿉니다. 번호 중에는 1~7이 아닌 값이
// 섞여 올 수 있으므로 이 변환은 실패할 수 있습니다.
// 비권장: `From`을 구현하고, 범위 밖의 번호는 `_ =>` 갈래에서 일요일로 덮습니다.
// 권장:   `TryFrom`을 구현하고, 범위 밖의 번호는 `Err`로 돌려줘 부르는 쪽이 처리하게 합니다.
//
// 실행: cargo run -p er-ch5-type-conversions --example 01_try_from_for_fallible

// 관리사무소가 보낸 배출 요일 번호입니다. 8은 입력 실수입니다.
const NOTICE: [u8; 3] = [1, 4, 8];

// 판별값을 번호와 같게 정해 두면 요일 → 번호 방향은 `as`로 정확히 바꿀 수 있습니다(3번 구역).
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Weekday {
    Mon = 1,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    fn korean(self) -> &'static str {
        match self {
            Weekday::Mon => "월",
            Weekday::Tue => "화",
            Weekday::Wed => "수",
            Weekday::Thu => "목",
            Weekday::Fri => "금",
            Weekday::Sat => "토",
            Weekday::Sun => "일",
        }
    }
}

// 비권장 쪽은 이름만 다른 같은 열거형에 구현합니다. 한 타입에 `From<u8>`과 `TryFrom<u8>`을
// 함께 구현할 수는 없기 때문입니다(`From`이 있으면 `TryFrom`이 포괄 구현으로 이미 생깁니다).
mod lenient {
    use super::Weekday;

    pub struct Day(pub Weekday);

    // 비권장: 시그니처는 "언제나 성공한다"고 약속하지만, 실제로는 모르는 번호를 일요일로 바꿉니다.
    // 부르는 쪽은 결과만 보고 원래 번호가 7이었는지 8이었는지 알 수 없습니다.
    impl From<u8> for Day {
        fn from(n: u8) -> Self {
            Day(match n {
                1 => Weekday::Mon,
                2 => Weekday::Tue,
                3 => Weekday::Wed,
                4 => Weekday::Thu,
                5 => Weekday::Fri,
                6 => Weekday::Sat,
                _ => Weekday::Sun,
            })
        }
    }
}

/// `TryFrom`이 실패했을 때 돌려주는 에러입니다. 받은 번호를 그대로 담아 무엇이 틀렸는지 알려 줍니다.
#[derive(Debug)]
struct NotAWeekday(u8);

// 권장: 실패할 수 있다는 사실이 반환 타입 `Result`에 드러납니다.
impl TryFrom<u8> for Weekday {
    type Error = NotAWeekday;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Weekday::Mon),
            2 => Ok(Weekday::Tue),
            3 => Ok(Weekday::Wed),
            4 => Ok(Weekday::Thu),
            5 => Ok(Weekday::Fri),
            6 => Ok(Weekday::Sat),
            7 => Ok(Weekday::Sun),
            other => Err(NotAWeekday(other)),
        }
    }
}

// 반대 방향(요일 → 번호)은 모든 값에서 성공하고 정보도 잃지 않으므로 `From`이 맞습니다.
// 필드 없는 열거형을 판별값으로 바꾸는 것은 `as`가 정확히 하는 일이므로 본문에서는 `as`를 씁니다.
impl From<Weekday> for u8 {
    fn from(day: Weekday) -> u8 {
        day as u8
    }
}

fn main() {
    println!("1. 비권장: From이 모르는 번호를 일요일로 덮습니다");
    let days: Vec<&str> = NOTICE
        .iter()
        .map(|&n| lenient::Day::from(n).0.korean())
        .collect();
    println!("    {NOTICE:?} → {days:?}"); // 출력:     [1, 4, 8] → ["월", "목", "일"]

    println!("2. 권장: TryFrom은 같은 번호를 에러로 돌려줍니다");
    for n in NOTICE {
        match Weekday::try_from(n) {
            Ok(day) => println!("    {n} → {}요일", day.korean()),
            // 에러 값에 받은 번호가 들어 있으므로 무엇이 틀렸는지 그대로 보여 줄 수 있습니다.
            Err(NotAWeekday(bad)) => println!("    {bad} → 거부: 1~7 범위 밖"),
        }
    }
    // 출력:     1 → 월요일
    // 출력:     4 → 목요일
    // 출력:     8 → 거부: 1~7 범위 밖

    // 부르는 쪽은 `try_into()`로도 쓸 수 있고(포괄 구현), 실패를 어떻게 다룰지 직접 고릅니다.
    // 모은 결과가 하나라도 `Err`이면 전체가 `Err`가 됩니다.
    let all: Result<Vec<Weekday>, NotAWeekday> = NOTICE.iter().map(|&n| n.try_into()).collect();
    println!("    전부 모으기: {all:?}"); // 출력:     전부 모으기: Err(NotAWeekday(8))

    println!("3. 반대 방향은 언제나 성공하므로 From");
    let back: Vec<u8> = [Weekday::Mon, Weekday::Sun]
        .into_iter()
        .map(u8::from)
        .collect();
    println!("    [월, 일] → {back:?}"); // 출력:     [월, 일] → [1, 7]
}
