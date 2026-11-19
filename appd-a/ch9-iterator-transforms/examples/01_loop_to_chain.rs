//! 예제 01: 인덱스 반복문을 한 단계씩 반복자 체인으로 바꾸기
//!
//! 실행: cargo run -q -p er-ch9-iterator-transforms --example 01_loop_to_chain
//!
//! 악기 연습 일지에서 "처음 2주 동안 인정되는 연습 시간"을 구하는 함수를
//! 인덱스 반복문(0단계)부터 어댑터 체인(4단계)까지 네 번 고쳐 씁니다.
//! 다섯 판은 모든 입력에서 같은 값을 돌려줍니다. 달라지는 것은 읽는 사람이
//! 확인해야 할 것의 양입니다.

/// 하루치 연습 기록. 일지는 날짜순으로 쌓입니다.
struct Session {
    day: u32,             // 연습을 시작한 날을 1일로 센 날짜
    minutes: Option<u32>, // 기록을 빠뜨린 날은 None
}

const LAST_DAY: u32 = 14; // 처음 2주까지만 셉니다
const DAILY_CAP: u32 = 60; // 하루에 60분까지만 인정합니다

// 0단계: 인덱스로 돕니다.
// i는 log를 가리키는 데만 쓰이는데도, 읽는 사람은 i의 범위가 맞는지,
// log[i]가 매번 같은 원소를 가리키는지 확인해야 합니다. log[i]는 실행 중에
// 경계 검사도 거칩니다(범위를 벗어나면 패닉. ch18-dont-panic에서 다룹니다).
#[expect(
    clippy::needless_range_loop,
    reason = "0단계: 인덱스로 도는 비권장 코드를 보여 줍니다"
)]
fn credited_v0_index(log: &[Session]) -> u32 {
    let mut total = 0;
    for i in 0..log.len() {
        if log[i].day > LAST_DAY {
            break;
        }
        let Some(minutes) = log[i].minutes else {
            continue;
        };
        total += minutes.min(DAILY_CAP);
    }
    total
}

// 1단계: 원소를 직접 받습니다(`for x in &v` 꼴).
// 인덱스가 사라졌으므로 인덱스를 잘못 쓸 수도, 범위를 벗어날 수도 없습니다.
fn credited_v1_for(log: &[Session]) -> u32 {
    let mut total = 0;
    for session in log {
        if session.day > LAST_DAY {
            break;
        }
        let Some(minutes) = session.minutes else {
            continue;
        };
        total += minutes.min(DAILY_CAP);
    }
    total
}

// 2단계: `break`를 take_while로 옮깁니다.
// "어디까지 보는가"가 반복문 머리로 올라왔습니다. 조건의 방향이 바뀐 점에
// 주의합니다. break는 "멈출 조건"(> 14), take_while은 "계속할 조건"(<= 14)입니다.
fn credited_v2_take_while(log: &[Session]) -> u32 {
    let mut total = 0;
    for session in log.iter().take_while(|s| s.day <= LAST_DAY) {
        let Some(minutes) = session.minutes else {
            continue;
        };
        total += minutes.min(DAILY_CAP);
    }
    total
}

// 3단계: `continue`를 filter_map으로 옮깁니다.
// 기록이 있는 날만 남기면서 Option을 벗기는 일이 한 어댑터로 끝납니다.
fn credited_v3_filter_map(log: &[Session]) -> u32 {
    let mut total = 0;
    for minutes in log
        .iter()
        .take_while(|s| s.day <= LAST_DAY)
        .filter_map(|s| s.minutes)
    {
        total += minutes.min(DAILY_CAP);
    }
    total
}

// 4단계: 본문의 계산은 map으로, 누적 변수는 sum으로 옮깁니다.
// 가변 변수가 하나도 남지 않고, 단계마다 메서드 이름이 의도를 말합니다.
fn credited_v4_chain(log: &[Session]) -> u32 {
    log.iter()
        .take_while(|s| s.day <= LAST_DAY) // 처음 2주까지
        .filter_map(|s| s.minutes) // 기록이 있는 날만
        .map(|m| m.min(DAILY_CAP)) // 하루 상한 적용
        .sum() // 모두 더함
}

// 인덱스판에서 흔한 실수: 날짜가 1일부터라서 인덱스도 1부터 시작했습니다.
// 컴파일도 되고 패닉도 없지만 첫날을 빠뜨립니다. clippy의 needless_range_loop는
// 이 코드를 `log.iter().skip(1)`로 바꾸라고 제안하는데, 그 제안을 보면
// "첫 원소를 건너뛴다"는 사실이 비로소 눈에 띕니다.
#[expect(
    clippy::needless_range_loop,
    reason = "인덱스를 1부터 시작한 비권장 코드를 보여 줍니다"
)]
fn credited_index_from_one(log: &[Session]) -> u32 {
    let mut total = 0;
    for i in 1..log.len() {
        if log[i].day > LAST_DAY {
            break;
        }
        let Some(minutes) = log[i].minutes else {
            continue;
        };
        total += minutes.min(DAILY_CAP);
    }
    total
}

fn session(day: u32, minutes: Option<u32>) -> Session {
    Session { day, minutes }
}

fn all_versions(log: &[Session]) -> [u32; 5] {
    [
        credited_v0_index(log),
        credited_v1_for(log),
        credited_v2_take_while(log),
        credited_v3_filter_map(log),
        credited_v4_chain(log),
    ]
}

fn main() {
    let diary = vec![
        session(1, Some(45)),
        session(2, None),
        session(3, Some(90)), // 60분만 인정
        session(5, Some(30)),
        session(8, Some(75)), // 60분만 인정
        session(12, None),
        session(14, Some(20)),
        session(15, Some(120)), // 2주가 지났으므로 세지 않음
        session(16, Some(50)),
    ];
    let empty: Vec<Session> = Vec::new();
    let late_start = vec![session(15, Some(40)), session(16, Some(30))];

    println!("== 1. 다섯 판의 결과 비교 ==");
    for (label, log) in [
        ("일지 A (9일치)", &diary),
        ("빈 일지", &empty),
        ("15일째부터 쓴 일지", &late_start),
    ] {
        let results = all_versions(log);
        let same = results.iter().all(|&r| r == results[0]);
        println!("{label} → {}분 (다섯 판 일치: {same})", results[0]);
    }
    // 출력: 일지 A (9일치) → 215분 (다섯 판 일치: true)
    // 출력: 빈 일지 → 0분 (다섯 판 일치: true)
    // 출력: 15일째부터 쓴 일지 → 0분 (다섯 판 일치: true)

    println!();
    println!("== 2. 인덱스판의 실수: 1부터 센 인덱스 ==");
    println!(
        "체인판: {}분, 1부터 센 인덱스판: {}분",
        credited_v4_chain(&diary),
        credited_index_from_one(&diary)
    );
    // 출력: 체인판: 215분, 1부터 센 인덱스판: 170분
}
