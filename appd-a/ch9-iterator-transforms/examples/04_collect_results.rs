//! 예제 04: 반복문 안의 `?`를 체인으로 옮길 때의 선택지
//!
//! 실행: cargo run -q -p er-ch9-iterator-transforms --example 04_collect_results
//!
//! 동아리 회비 입금 내역("이름 금액" 줄)을 읽는 같은 일을 여섯 가지로 씁니다.
//! 같은 입력에 대해 "결과가 무엇인가"와 "몇 줄을 읽고 멈췄는가"를 비교합니다.
//! 무엇을 고를지는 실패한 줄을 어떻게 다뤄야 하는가로 정해집니다.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 회비 입금 한 건.
#[derive(Debug)]
struct Dues {
    member: String,
    amount: u32,
}

#[derive(Debug)]
enum DuesError {
    NoAmount(String),  // 금액이 없는 줄
    BadAmount(String), // 금액을 숫자로 읽을 수 없는 줄
    Overflow,          // 합계가 u32를 넘음
}

impl fmt::Display for DuesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DuesError::NoAmount(line) => write!(f, "금액 없음 \"{line}\""),
            DuesError::BadAmount(line) => write!(f, "금액 오류 \"{line}\""),
            DuesError::Overflow => write!(f, "합계가 너무 큼"),
        }
    }
}

// 각 판이 몇 줄을 읽었는지 보려고 parse_line이 불린 횟수를 셉니다(관찰용).
static PARSED: AtomicUsize = AtomicUsize::new(0);

fn parse_line(line: &str) -> Result<Dues, DuesError> {
    PARSED.fetch_add(1, Ordering::Relaxed);
    let (member, amount) = line
        .split_once(' ')
        .ok_or_else(|| DuesError::NoAmount(line.to_string()))?;
    let amount = amount
        .parse()
        .map_err(|_| DuesError::BadAmount(line.to_string()))?;
    Ok(Dues {
        member: member.to_string(),
        amount,
    })
}

// ---- 0. 기준: for 반복문 + ? -----------------------------------------------

fn read_all_loop(lines: &[&str]) -> Result<Vec<Dues>, DuesError> {
    let mut all = Vec::new();
    for line in lines {
        all.push(parse_line(line)?); // 실패하면 함수가 끝납니다
    }
    Ok(all)
}

// ---- 1. 첫 에러에서 멈추기: Result<Vec<_>, _>로 collect -------------------------

// 클로저 안에서는 ?로 바깥 함수를 끝낼 수 없으므로, 원소마다 Result를 그대로 두고
// collect가 첫 Err에서 멈추게 합니다. 반환 타입이 collect의 결과 타입을 정합니다.
fn read_all(lines: &[&str]) -> Result<Vec<Dues>, DuesError> {
    lines.iter().map(|line| parse_line(line)).collect()
}

// ---- 2. 실패를 버리기: filter_map + ok() -----------------------------------------

// .ok()가 에러를 버린다는 결정이 코드에 드러납니다. 몇 줄이 버려졌는지는 알 수 없습니다.
fn read_valid(lines: &[&str]) -> Vec<Dues> {
    lines
        .iter()
        .filter_map(|line| parse_line(line).ok())
        .collect()
}

// ---- 3. 성공과 실패를 나눠 모으기: partition ---------------------------------------

fn read_split(lines: &[&str]) -> (Vec<Dues>, Vec<DuesError>) {
    let (oks, errs): (Vec<_>, Vec<_>) = lines
        .iter()
        .map(|line| parse_line(line))
        .partition(Result::is_ok);
    // partition은 Result를 벗기지 않으므로 한 번씩 더 돕니다.
    // Result도 IntoIterator(Ok면 값 하나, Err면 0개)라서 flatten으로 Ok 안의 값만 꺼냅니다.
    let dues = oks.into_iter().flatten().collect();
    let errors = errs.into_iter().filter_map(Result::err).collect();
    (dues, errors)
}

// 같은 일을 반복문과 match로 쓰면 한 번 돌면서 바로 두 곳에 나눠 담습니다.
fn read_split_loop(lines: &[&str]) -> (Vec<Dues>, Vec<DuesError>) {
    let mut dues = Vec::new();
    let mut errors = Vec::new();
    for line in lines {
        match parse_line(line) {
            Ok(d) => dues.push(d),
            Err(e) => errors.push(e),
        }
    }
    (dues, errors)
}

// ---- 4. 실패할 수 있는 접기: try_fold ----------------------------------------------

// 클로저가 Result를 돌려주므로 클로저 안에서 ?를 쓸 수 있습니다. 여기서 ?는
// 클로저를 끝내고, try_fold가 그 Err를 받아 순회를 멈춥니다.
fn total(lines: &[&str]) -> Result<u32, DuesError> {
    lines.iter().try_fold(0u32, |sum, line| {
        let dues = parse_line(line)?;
        sum.checked_add(dues.amount).ok_or(DuesError::Overflow)
    })
}

// ---- 5. 원소마다 일하다 실패하면 멈추기: try_for_each ------------------------------

fn record(lines: &[&str], ledger: &mut BTreeMap<String, u32>) -> Result<(), DuesError> {
    lines.iter().try_for_each(|line| {
        let dues = parse_line(line)?;
        *ledger.entry(dues.member).or_default() += dues.amount;
        Ok(())
    })
}

// ---- 출력 도우미 ------------------------------------------------------------

/// 한 판의 결과를 출력하고, 그 판이 parse_line을 몇 번 불렀는지 덧붙입니다.
fn report(label: &str, outcome: String, total_lines: usize) {
    let parsed = PARSED.swap(0, Ordering::Relaxed); // 읽고 0으로 되돌립니다
    println!("{label:<15}: {outcome} (읽은 줄 {parsed}/{total_lines})");
}

fn show<T: fmt::Debug>(result: &Result<T, DuesError>) -> String {
    match result {
        Ok(value) => format!("Ok({value:?})"),
        Err(e) => format!("Err({e})"),
    }
}

fn members(all: Vec<Dues>) -> Vec<String> {
    all.into_iter().map(|d| d.member).collect()
}

fn messages(errors: Vec<DuesError>) -> Vec<String> {
    errors.iter().map(|e| e.to_string()).collect()
}

fn main() {
    let clean = ["가온 15000", "나래 15000", "라온 20000"];
    let mixed = [
        "가온 15000",
        "나래 15000",
        "다솜 만오천",
        "라온 20000",
        "미르",
        "보람 15000",
    ];

    for (label, lines) in [("깨끗한 입력", &clean[..]), ("섞인 입력", &mixed[..])] {
        let n = lines.len();
        println!("==== {label} ({n}줄) ====");

        report("0. for + ?", show(&read_all_loop(lines).map(members)), n);
        report("1. collect", show(&read_all(lines).map(members)), n);
        report(
            "2. filter_map",
            format!("{:?}", members(read_valid(lines))),
            n,
        );

        let (dues, errors) = read_split(lines);
        let (dues, errors) = (members(dues), messages(errors));
        report("3. partition", format!("{dues:?} / 에러 {errors:?}"), n);
        let (loop_dues, loop_errors) = read_split_loop(lines);
        let same = members(loop_dues) == dues && messages(loop_errors) == errors;
        report("   for + match", format!("partition과 같은 결과 {same}"), n);

        report("4. try_fold", show(&total(lines)), n);

        let mut ledger = BTreeMap::new();
        let r = record(lines, &mut ledger);
        report(
            "5. try_for_each",
            format!("{} / 장부 {ledger:?}", show(&r)),
            n,
        );
        println!();
    }
    // 출력: ==== 깨끗한 입력 (3줄) ====
    // 출력: 0. for + ?     : Ok(["가온", "나래", "라온"]) (읽은 줄 3/3)
    // 출력: 1. collect     : Ok(["가온", "나래", "라온"]) (읽은 줄 3/3)
    // 출력: 2. filter_map  : ["가온", "나래", "라온"] (읽은 줄 3/3)
    // 출력: 3. partition   : ["가온", "나래", "라온"] / 에러 [] (읽은 줄 3/3)
    // 출력:    for + match : partition과 같은 결과 true (읽은 줄 3/3)
    // 출력: 4. try_fold    : Ok(50000) (읽은 줄 3/3)
    // 출력: 5. try_for_each: Ok(()) / 장부 {"가온": 15000, "나래": 15000, "라온": 20000} (읽은 줄 3/3)
    // 출력:
    // 출력: ==== 섞인 입력 (6줄) ====
    // 출력: 0. for + ?     : Err(금액 오류 "다솜 만오천") (읽은 줄 3/6)
    // 출력: 1. collect     : Err(금액 오류 "다솜 만오천") (읽은 줄 3/6)
    // 출력: 2. filter_map  : ["가온", "나래", "라온", "보람"] (읽은 줄 6/6)
    // 출력: 3. partition   : ["가온", "나래", "라온", "보람"] / 에러 ["금액 오류 \"다솜 만오천\"", "금액 없음 \"미르\""] (읽은 줄 6/6)
    // 출력:    for + match : partition과 같은 결과 true (읽은 줄 6/6)
    // 출력: 4. try_fold    : Err(금액 오류 "다솜 만오천") (읽은 줄 3/6)
    // 출력: 5. try_for_each: Err(금액 오류 "다솜 만오천") / 장부 {"가온": 15000, "나래": 15000} (읽은 줄 3/6)

    println!("==== try_fold만의 실패: 합계가 u32를 넘음 ====");
    let huge = ["가온 4000000000", "나래 500000000", "다솜 1"];
    report("4. try_fold", show(&total(&huge)), huge.len());
    // 출력: 4. try_fold    : Err(합계가 너무 큼) (읽은 줄 2/3)
}
