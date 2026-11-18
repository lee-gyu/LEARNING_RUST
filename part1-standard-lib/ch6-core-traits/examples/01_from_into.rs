// 이 예제가 보여 주는 것: From을 구현하면 Into가 따라온다(포괄 구현).
//                        표준 라이브러리의 From 구현들, Into<T>를 바운드로 쓰는 매개변수,
//                        ?가 에러를 From::from으로 바꾼다는 것
// 실행: cargo run -p p1-ch6-core-traits --example 01_from_into
//
// 표준 라이브러리에는 다음 포괄 구현이 있습니다.
//     impl<T, U> Into<U> for T where U: From<T>
// "U: From<T>이면 T: Into<U>"라는 뜻입니다. 그래서 From 하나만 구현하면 into()도 저절로 생깁니다.

use std::error::Error;

// 시간 단위를 서로 다른 타입으로 구분합니다(뉴타입 패턴, Part 0 ch19).
#[derive(Debug, Clone, Copy)]
struct Seconds(u64);
#[derive(Debug, Clone, Copy)]
struct Minutes(u64);
#[derive(Debug, Clone, Copy)]
struct Hours(u64);

// 분 → 초: 실패하지 않고, 정보를 잃지 않고, 같은 "시간 길이"를 나타내는 변환이므로 From이 어울립니다.
impl From<Minutes> for Seconds {
    fn from(m: Minutes) -> Self {
        Seconds(m.0 * 60)
    }
}

impl From<Hours> for Seconds {
    fn from(h: Hours) -> Self {
        Seconds(h.0 * 3600)
    }
}

// Seconds → u64. 트레이트(From)도 u64도 표준 라이브러리 것이지만, 타입 인자 자리에
// 내 타입 Seconds가 있으므로 고아 규칙이 허용합니다.
impl From<Seconds> for u64 {
    fn from(s: Seconds) -> Self {
        s.0
    }
}

// Into<Seconds>를 바운드로 쓰면 Seconds, Minutes, Hours를 모두 받습니다.
// Seconds 자신도 받는 이유: 표준 라이브러리에 impl<T> From<T> for T(반사 구현)가 있기 때문입니다.
fn timer_label(duration: impl Into<Seconds>) -> String {
    let secs = duration.into();
    format!("{}초 타이머", secs.0)
}

// 문자열을 저장하는 생성자의 매개변수로 자주 쓰는 모양입니다.
struct Member {
    name: String,
}

impl Member {
    // &str을 받으면 String을 가진 호출자도 다시 복사해야 하고,
    // String을 받으면 &str을 가진 호출자가 to_string()을 붙여야 합니다.
    // Into<String>은 둘 다 받고, String이 오면 버퍼를 그대로 옮깁니다(복사 없음).
    fn new(name: impl Into<String>) -> Self {
        Member { name: name.into() }
    }
}

// ?는 Err(e)를 만나면 return Err(From::from(e))를 실행합니다.
// Box<dyn Error>는 Error를 구현한 모든 타입과 &str, String에서 From으로 만들 수 있습니다.
fn parse_minutes(text: &str) -> Result<Seconds, Box<dyn Error>> {
    let n: u64 = text.trim().parse()?; // ParseIntError → Box<dyn Error>
    if n == 0 {
        return Err("0분 타이머는 만들 수 없습니다".into()); // &str → Box<dyn Error>
    }
    Ok(Minutes(n).into())
}

// 위 함수의 ?를 손으로 풀어 쓴 모양입니다.
fn parse_minutes_by_hand(text: &str) -> Result<Seconds, Box<dyn Error>> {
    let n: u64 = match text.trim().parse::<u64>() {
        Ok(n) => n,
        Err(e) => return Err(From::from(e)),
    };
    Ok(Seconds::from(Minutes(n)))
}

fn main() {
    // 1. From을 구현하면 두 가지 방법으로 부를 수 있습니다.
    println!("== 1. From과 Into ==");
    let a = Seconds::from(Minutes(3)); // 목적 타입::from(값)
    let b: Seconds = Hours(2).into(); // 값.into() — 목적 타입은 변수 타입에서 알아냅니다.
    println!("{a:?} {b:?}"); // 출력: Seconds(180) Seconds(7200)
    let raw: u64 = b.into(); // From<Seconds> for u64 덕분에 생긴 into
    println!("u64로: {raw}"); // 출력: u64로: 7200

    // 2. 표준 라이브러리의 From 구현 몇 가지. 모두 실패하지 않고 정보를 잃지 않습니다.
    println!("\n== 2. 표준 라이브러리의 From ==");
    let big = i64::from(-3_i32); // 정수 넓히기
    let code = u32::from('가'); // 글자 → 코드 포인트
    let letter = char::from(65_u8); // u8 → char (0~255는 모두 올바른 글자)
    println!("{big} {code} {letter:?}"); // 출력: -3 44032 'A'
    let text = String::from('한'); // char → String
    let list = Vec::from([1, 2, 3]); // 배열 → Vec
    let some = Option::from(5); // 값 → Some(값)
    println!("{text:?} {list:?} {some:?}"); // 출력: "한" [1, 2, 3] Some(5)

    // 3. Into를 바운드로 쓰면 여러 타입을 받는 함수가 됩니다.
    println!("\n== 3. impl Into<Seconds> 매개변수 ==");
    println!("{}", timer_label(Seconds(45))); // 출력: 45초 타이머
    println!("{}", timer_label(Minutes(5))); // 출력: 300초 타이머
    println!("{}", timer_label(Hours(1))); // 출력: 3600초 타이머

    // 4. impl Into<String>: String을 넘기면 힙 버퍼가 그대로 옮겨집니다.
    println!("\n== 4. impl Into<String> 매개변수 ==");
    let from_str = Member::new("민수"); // &str → 새 String 할당
    let owned = String::from("지영");
    let before = owned.as_ptr(); // 힙 버퍼의 주소(값은 출력하지 않고 비교만 합니다)
    let from_string = Member::new(owned); // String → String: 반사 구현이라 이동만 합니다.
    println!("{}, {}", from_str.name, from_string.name); // 출력: 민수, 지영
    println!("같은 버퍼인가? {}", before == from_string.name.as_ptr()); // 출력: 같은 버퍼인가? true

    // 5. ?와 From: 에러 타입이 달라도 From 구현이 있으면 ?가 바꿔 줍니다.
    println!("\n== 5. ?는 From::from을 부릅니다 ==");
    for input in ["2", "0", "두 시간"] {
        match parse_minutes(input) {
            Ok(s) => println!("{input:?} → {s:?}"),
            Err(e) => println!("{input:?} → 에러: {e}"),
        }
    }
    // 출력:
    // "2" → Seconds(120)
    // "0" → 에러: 0분 타이머는 만들 수 없습니다
    // "두 시간" → 에러: invalid digit found in string
    let by_hand = parse_minutes_by_hand("x").map_err(|e| e.to_string());
    println!("손으로 푼 버전: {by_hand:?}"); // 출력: 손으로 푼 버전: Err("invalid digit found in string")
}
