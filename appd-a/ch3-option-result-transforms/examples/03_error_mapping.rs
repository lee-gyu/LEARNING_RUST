// 이 예제가 보여 주는 것: 에러 갈래를 손으로 쓴 match를 map_err(..)?와 From + ?로 줄이기
// 실행: cargo run -q -p er-ch3-option-result-transforms --example 03_error_mapping
//
// Result를 돌려주는 함수 안에서 다른 에러 타입의 Result를 만나면, 그 에러를 이 함수의 에러 타입으로
// 바꿔서 돌려줘야 합니다. 방법은 세 가지이고, 세 판은 모든 입력에서 같은 값을 돌려줍니다.
//   (가) match로 갈래마다 return Err(..)를 적는다
//   (나) map_err로 에러 타입을 맞춘 뒤 ?를 붙인다
//   (다) From을 구현해 두고 ?만 붙인다 (?가 From::from으로 바꿔 줌)
// 세트 기록 "횟수 x 무게"(예: "12 x 40.5")를 읽어 들어 올린 총 무게를 계산합니다.

use std::num::{ParseFloatError, ParseIntError};

// 에러 타입 설계는 ch4-idiomatic-errors에서 다룹니다. 여기서는 비교에 필요한 Debug와 PartialEq만 붙입니다.
#[derive(Debug, PartialEq)]
enum SetError {
    NoSeparator,
    BadReps(ParseIntError),
    BadWeight(ParseFloatError),
}

// (가) match판. 실패 갈래가 단계마다 세 줄씩 들어가 계산(마지막 줄)보다 길어집니다.
// clippy는 이 함수에 아무 경고도 하지 않습니다. question_mark 린트는 `Err(e) => return Err(e)`나
// `Err(e) => return Err(e.into())`는 ?로 바꾸라고 알려 주지만, 배리언트로 감싸는 갈래나
// `None => return Err(..)` 갈래는 잡지 않습니다. 이 모양은 직접 알아보고 고쳐야 합니다.
fn volume_match(line: &str) -> Result<f64, SetError> {
    let (reps, weight) = match line.split_once('x') {
        Some(pair) => pair,
        None => return Err(SetError::NoSeparator),
    };
    let reps: u32 = match reps.trim().parse() {
        Ok(n) => n,
        Err(e) => return Err(SetError::BadReps(e)),
    };
    let weight: f64 = match weight.trim().parse() {
        Ok(w) => w,
        Err(e) => return Err(SetError::BadWeight(e)),
    };
    Ok(f64::from(reps) * weight)
}

// (나) map_err판. "어떤 배리언트로 감쌀지"만 남고 return과 갈래가 사라집니다.
// 튜플 배리언트의 이름(SetError::BadReps)은 안의 값을 받아 SetError를 만드는 함수처럼 넘길 수 있습니다.
// Option인 split_once는 ok_or로 Result로 바꿔야 ?를 쓸 수 있습니다(예제 02).
fn volume_map_err(line: &str) -> Result<f64, SetError> {
    let (reps, weight) = line.split_once('x').ok_or(SetError::NoSeparator)?;
    let reps: u32 = reps.trim().parse().map_err(SetError::BadReps)?;
    let weight: f64 = weight.trim().parse().map_err(SetError::BadWeight)?;
    Ok(f64::from(reps) * weight)
}

// (다) From판. 변환을 타입 옆에 한 번 적어 두면, ?가 Err를 돌려주기 전에 From::from을 불러 바꿔 줍니다.
impl From<ParseIntError> for SetError {
    fn from(e: ParseIntError) -> Self {
        SetError::BadReps(e)
    }
}

impl From<ParseFloatError> for SetError {
    fn from(e: ParseFloatError) -> Self {
        SetError::BadWeight(e)
    }
}

fn volume_from(line: &str) -> Result<f64, SetError> {
    let (reps, weight) = line.split_once('x').ok_or(SetError::NoSeparator)?;
    let reps: u32 = reps.trim().parse()?; // ParseIntError → SetError::BadReps
    let weight: f64 = weight.trim().parse()?; // ParseFloatError → SetError::BadWeight
    Ok(f64::from(reps) * weight)
}

// ---------------------------------------------------------------------------
// From으로는 할 수 없는 일: 에러가 난 "위치"를 덧붙이기
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct LogError {
    line_no: usize,
    source: SetError,
}

// From::from은 에러 값 하나만 받으므로 "몇 번째 줄이었는가"를 알 수 없습니다.
// 그런 맥락은 그 값을 아는 자리에서 map_err의 클로저로 붙입니다.
fn total_volume(log: &str) -> Result<f64, LogError> {
    let mut total = 0.0;
    for (i, line) in log.lines().enumerate() {
        total += volume_from(line).map_err(|source| LogError {
            line_no: i + 1,
            source,
        })?;
    }
    Ok(total)
}

fn main() {
    println!("== 1. match판, map_err판, From판의 결과 비교 ==");
    for line in ["12 x 40.5", "12 40.5", "열두 x 40.5", "12 x 사십"] {
        let a = volume_match(line);
        let b = volume_map_err(line);
        let c = volume_from(line);
        let same = a == b && b == c;
        println!("{line:?} → {c:?} (세 판 일치: {same})");
    }
    // 출력:
    // "12 x 40.5" → Ok(486.0) (세 판 일치: true)
    // "12 40.5" → Err(NoSeparator) (세 판 일치: true)
    // "열두 x 40.5" → Err(BadReps(ParseIntError { kind: InvalidDigit })) (세 판 일치: true)
    // "12 x 사십" → Err(BadWeight(ParseFloatError { kind: Invalid })) (세 판 일치: true)

    println!();
    println!("== 2. map_err로 위치를 덧붙이기 ==");
    for log in ["12 x 40\n10 x 42.5\n8 x 45", "12 x 40\n10 x 42,5\n8 x 45"] {
        match total_volume(log) {
            Ok(total) => println!("합계 {total}kg"),
            Err(e) => println!("{}번째 줄에서 실패: {:?}", e.line_no, e.source),
        }
    }
    // 출력:
    // 합계 1265kg
    // 2번째 줄에서 실패: BadWeight(ParseFloatError { kind: Invalid })
}
