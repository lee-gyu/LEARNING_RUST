// 지침: 실패할 수 있는 작업의 결과는 에러 코드가 아니라 Result<T, E>로 돌려줍니다.
// 비권장: "HH:MM"을 분으로 바꾸되 실패하면 -1(형식 오류)이나 -2(범위 오류)를 돌려줌
// 권장:   Result<u32, TimeError>
//
// 실행: cargo run -p er-ch1-express-data-structures --example 06_error_code_to_result

// 회의 시작·끝 시각입니다. 둘째 줄의 끝 시각에는 숫자 0 대신 영문자 O가 들어갔습니다.
const MEETINGS: [(&str, &str); 4] = [
    ("09:30", "10:15"),
    ("09:30", "10:3O"),
    ("23:00", "24:30"),
    ("14:00", "13:00"),
];

// ---- 비권장 ----------------------------------------------------------------

// 성공하면 0 이상의 분, 실패하면 음수 코드입니다. 성공 값과 실패 코드가 같은 i32에 섞여 있습니다.
fn minutes_or_code(hhmm: &str) -> i32 {
    let Some((h, m)) = hhmm.split_once(':') else {
        return -1;
    };
    let (Ok(h), Ok(m)) = (h.parse::<i32>(), m.parse::<i32>()) else {
        return -1;
    };
    if !(0..24).contains(&h) || !(0..60).contains(&m) {
        return -2;
    }
    h * 60 + m
}

fn duration_discouraged(start: &str, end: &str) -> i32 {
    // 코드를 확인하는 것을 잊어도 i32끼리의 뺄셈이므로 아무 문제 없이 계산됩니다.
    minutes_or_code(end) - minutes_or_code(start)
}

// ---- 권장 ------------------------------------------------------------------

// 실패의 종류를 열거형으로 나눕니다. 에러 타입을 설계하는 방법은 ch4-idiomatic-errors에서 다룹니다.
#[derive(Debug)]
enum TimeError {
    BadFormat,
    OutOfRange,
    EndBeforeStart,
}

// 성공하면 Ok(분), 실패하면 Err(이유). 호출하는 쪽은 Ok인지 확인하지 않고는 분을 꺼낼 수 없습니다.
fn minutes(hhmm: &str) -> Result<u32, TimeError> {
    let (h, m) = hhmm.split_once(':').ok_or(TimeError::BadFormat)?;
    let (Ok(h), Ok(m)) = (h.parse::<u32>(), m.parse::<u32>()) else {
        return Err(TimeError::BadFormat);
    };
    if h >= 24 || m >= 60 {
        return Err(TimeError::OutOfRange);
    }
    Ok(h * 60 + m)
}

fn duration(start: &str, end: &str) -> Result<u32, TimeError> {
    // ?는 Err를 만나면 그 Err를 그대로 돌려주고 함수를 끝냅니다. 표준 타입이라 쓸 수 있는 문법입니다.
    let start = minutes(start)?;
    let end = minutes(end)?;
    end.checked_sub(start).ok_or(TimeError::EndBeforeStart)
}

fn main() {
    println!("[비권장] 실패하면 음수 코드");
    for (start, end) in MEETINGS {
        println!("  {start}~{end}: {}분", duration_discouraged(start, end));
    }
    // 출력:
    //   09:30~10:15: 45분
    //   09:30~10:3O: -571분
    //   23:00~24:30: -1382분
    //   14:00~13:00: -60분

    println!("[권장] Result<u32, TimeError>");
    for (start, end) in MEETINGS {
        match duration(start, end) {
            Ok(min) => println!("  {start}~{end}: {min}분"),
            Err(e) => println!("  {start}~{end}: 계산할 수 없음 ({e:?})"),
        }
    }
    // 출력:
    //   09:30~10:15: 45분
    //   09:30~10:3O: 계산할 수 없음 (BadFormat)
    //   23:00~24:30: 계산할 수 없음 (OutOfRange)
    //   14:00~13:00: 계산할 수 없음 (EndBeforeStart)
}
