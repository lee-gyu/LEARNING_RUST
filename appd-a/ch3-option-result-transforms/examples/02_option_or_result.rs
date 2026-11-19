// 이 예제가 보여 주는 것: 실패를 알려야 하는 함수는 Option보다 Result로, 둘 사이는 ok_or·ok로 오가기
// 실행: cargo run -q -p er-ch3-option-result-transforms --example 02_option_or_result
//
// Option의 None은 "없다"는 사실 하나만 전합니다. 실패한 이유가 여럿인 함수가 Option을 돌려주면
// 호출한 쪽은 무엇을 고쳐야 할지 알려 줄 수 없습니다. 이유를 전할 수 있으면 Result가 낫습니다.
// 반대로 "없는 것"이 정상적인 상태(빈 좌석, 검색 결과 없음)라면 Option이 맞습니다.
// 무엇이 에러인지는 함수 하나가 아니라 그 값을 쓰는 문맥이 정하므로, 경계에서 ok_or·ok로 바꿉니다.

use std::fmt;

#[derive(Clone, Copy, PartialEq)]
struct Seat {
    row: char,   // 'A'~'F'
    number: u32, // 1~30
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.row, self.number)
    }
}

// main에서 좌석 값을 짧게 만들기 위한 도우미입니다.
const fn seat(row: char, number: u32) -> Seat {
    Seat { row, number }
}

// 출력을 짧게 하려고 디버그 형식도 "B12" 모양으로 씁니다.
impl fmt::Debug for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

// 에러 타입 설계(Display, Error 트레이트)는 ch4-idiomatic-errors에서 다룹니다. 여기서는 Debug만 붙입니다.
#[derive(Debug)]
enum SeatError {
    Empty,
    BadRow(char),
    BadNumber(String),
    OutOfRange(u32),
    NotBooked(Seat),
}

// 비권장: 실패 이유가 네 가지인데 모두 None 하나로 뭉개집니다.
fn parse_seat_opt(code: &str) -> Option<Seat> {
    let mut chars = code.chars();
    let row = chars.next().filter(|c| ('A'..='F').contains(c))?;
    let number = chars
        .as_str()
        .parse::<u32>()
        .ok()
        .filter(|n| (1..=30).contains(n))?;
    Some(Seat { row, number })
}

// 권장: 같은 검사를 하되, 실패한 자리마다 이유를 담아 돌려줍니다.
// chars().next()는 Option이므로 ok_or로 Result로 바꿔야 ?가 SeatError를 돌려줄 수 있습니다.
fn parse_seat(code: &str) -> Result<Seat, SeatError> {
    let mut chars = code.chars();
    // SeatError::Empty는 이미 만들어진 값이라 ok_or로 충분합니다.
    let row = chars.next().ok_or(SeatError::Empty)?;
    if !('A'..='F').contains(&row) {
        return Err(SeatError::BadRow(row));
    }
    let rest = chars.as_str();
    // 에러 값을 만들 때 String을 할당하므로, 클로저를 받는 map_err로 실패할 때만 만듭니다.
    let number: u32 = rest
        .parse()
        .map_err(|_| SeatError::BadNumber(rest.to_string()))?;
    if !(1..=30).contains(&number) {
        return Err(SeatError::OutOfRange(number));
    }
    Ok(Seat { row, number })
}

// Result를 받은 쪽은 이유마다 다른 안내를 할 수 있습니다.
// 갈래마다 하는 일이 다르므로 변환 메서드가 아니라 match가 맞는 자리입니다(예제 05).
fn advice(err: &SeatError) -> String {
    match err {
        SeatError::Empty => String::from("좌석 코드를 입력하세요"),
        SeatError::BadRow(row) => format!("{row}열은 없습니다. A~F열 중에서 고르세요"),
        SeatError::BadNumber(text) => format!("번호 {text:?}를 숫자로 읽을 수 없습니다"),
        SeatError::OutOfRange(n) => format!("{n}번은 없습니다. 1~30번 중에서 고르세요"),
        SeatError::NotBooked(seat) => format!("{seat} 좌석에는 예약이 없습니다"),
    }
}

// 조회 함수: 예약되지 않은 좌석은 정상적인 상태이므로 Option이 맞습니다.
fn occupant<'a>(bookings: &'a [(Seat, &'a str)], seat: Seat) -> Option<&'a str> {
    bookings
        .iter()
        .find(|(booked, _)| *booked == seat)
        .map(|(_, name)| *name)
}

// 같은 조회를 "예약 취소" 문맥에서 쓰면, 예약이 없다는 것은 이제 에러입니다.
// 조회 함수를 고치지 않고, 그것을 에러로 보는 이 함수가 ok_or로 바꿉니다.
fn cancel(bookings: &[(Seat, &str)], code: &str) -> Result<String, SeatError> {
    let seat = parse_seat(code)?;
    let name = occupant(bookings, seat).ok_or(SeatError::NotBooked(seat))?;
    Ok(format!("{seat} 좌석의 {name} 예약을 취소합니다"))
}

// 반대 방향: 신청서의 희망 좌석은 적지 않아도 되고, 잘못 적었으면 무시하고 자동 배정합니다.
// 이 함수의 호출자는 이유를 쓰지 않으므로 Option이 맞고, parse_seat의 이유는 ok()로 여기서 버립니다.
// 이유를 버리는 결정이 이 한 곳에 드러나 있다는 점이 parse_seat_opt와 다릅니다.
fn wished_seat(form: &str) -> Option<Seat> {
    let (_, wish) = form.split_once("희망=")?;
    parse_seat(wish.trim()).ok()
}

fn main() {
    println!("== 1. Option은 실패했다는 사실만, Result는 이유까지 전합니다 ==");
    for code in ["B12", "", "Z3", "B", "B99"] {
        let opt = parse_seat_opt(code);
        let res = parse_seat(code);
        println!("{code:?}");
        // Option을 받은 쪽이 할 수 있는 안내는 한 가지뿐입니다.
        let opt_message = match opt {
            Some(seat) => format!("{seat} 좌석을 골랐습니다"),
            None => String::from("좌석 코드가 잘못되었습니다"),
        };
        let res_message = match &res {
            Ok(seat) => format!("{seat} 좌석을 골랐습니다"),
            Err(e) => advice(e),
        };
        println!("  Option: {opt:?} → {opt_message}");
        println!("  Result: {res:?} → {res_message}");
    }
    // 출력:
    // "B12"
    //   Option: Some(B12) → B12 좌석을 골랐습니다
    //   Result: Ok(B12) → B12 좌석을 골랐습니다
    // ""
    //   Option: None → 좌석 코드가 잘못되었습니다
    //   Result: Err(Empty) → 좌석 코드를 입력하세요
    // "Z3"
    //   Option: None → 좌석 코드가 잘못되었습니다
    //   Result: Err(BadRow('Z')) → Z열은 없습니다. A~F열 중에서 고르세요
    // "B"
    //   Option: None → 좌석 코드가 잘못되었습니다
    //   Result: Err(BadNumber("")) → 번호 ""를 숫자로 읽을 수 없습니다
    // "B99"
    //   Option: None → 좌석 코드가 잘못되었습니다
    //   Result: Err(OutOfRange(99)) → 99번은 없습니다. 1~30번 중에서 고르세요

    println!();
    println!("== 2. 없는 것이 에러인지는 문맥이 정합니다 ==");
    let bookings = [(seat('B', 12), "가온"), (seat('C', 3), "나래")];
    for s in [seat('B', 12), seat('B', 13)] {
        println!("조회 {s} → {:?}", occupant(&bookings, s));
    }
    for code in ["C3", "C4", "C99"] {
        match cancel(&bookings, code) {
            Ok(message) => println!("취소 {code} → {message}"),
            Err(e) => println!("취소 {code} → {e:?}: {}", advice(&e)),
        }
    }
    // 출력:
    // 조회 B12 → Some("가온")
    // 조회 B13 → None
    // 취소 C3 → C3 좌석의 나래 예약을 취소합니다
    // 취소 C4 → NotBooked(C4): C4 좌석에는 예약이 없습니다
    // 취소 C99 → OutOfRange(99): 99번은 없습니다. 1~30번 중에서 고르세요

    println!();
    println!("== 3. 이유를 쓰지 않는 곳에서는 ok()로 버립니다 ==");
    for form in ["이름=다솜, 희망=A7", "이름=라온, 희망=맨 앞", "이름=미르"] {
        let assigned = wished_seat(form).unwrap_or(seat('F', 30));
        println!("{form:?} → {assigned}");
    }
    // 출력:
    // "이름=다솜, 희망=A7" → A7
    // "이름=라온, 희망=맨 앞" → F30
    // "이름=미르" → F30

    // 4. Result에는 #[must_use]가 붙어 있어 결과를 버리면 컴파일러가 경고합니다(unused_must_use).
    //    Option에는 없어서 실패를 그냥 버려도 아무 말이 없습니다. 아래 #[expect]를 지우고 빌드해 보세요.
    parse_seat_opt("Z3");
    #[expect(
        unused_must_use,
        reason = "Result를 버리면 경고가 난다는 것을 보이려고 남겨 둡니다"
    )]
    parse_seat("Z3");
}
