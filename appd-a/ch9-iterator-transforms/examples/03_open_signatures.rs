//! 예제 03: 받는 쪽 시그니처를 `&[T]` 대신 `impl IntoIterator`로 열기
//!
//! 실행: cargo run -q -p er-ch9-iterator-transforms --example 03_open_signatures
//!
//! 카페의 하루 판매량(잔 수)에서 "목표를 넘긴 날 수"를 세는 함수를 두 가지로
//! 선언합니다. 본문은 원소를 앞에서부터 한 번 읽기만 하는데, `&[u32]`로 받으면
//! 부르는 쪽은 판매량을 먼저 Vec에 모아야 합니다. `impl IntoIterator`로 받으면
//! 부르는 쪽이 가진 것을 그대로 넘깁니다.

use std::collections::BTreeMap;

const TARGET: u32 = 120; // 하루 목표 판매량(잔)

// ---- 1. 한 번 앞에서부터 읽는 함수 ------------------------------------------

// 비권장: 본문이 슬라이스의 기능(길이, 인덱스, 여러 번 읽기)을 쓰지 않는데도
// 슬라이스만 받습니다.
fn hit_days_slice(cups: &[u32], target: u32) -> usize {
    cups.iter().filter(|&&c| c >= target).count()
}

// 권장: 본문이 실제로 하는 일(원소를 차례로 한 번 읽기)만 요구합니다.
// Vec, 배열, 맵의 values(), 어댑터 체인을 모두 받을 수 있습니다.
fn hit_days(cups: impl IntoIterator<Item = u32>, target: u32) -> usize {
    cups.into_iter().filter(|&c| c >= target).count()
}

// ---- 2. 결과를 반복자로 돌려주기 --------------------------------------------

// 포스기 기록("날짜 잔수" 줄)에서 잔 수만 꺼냅니다. Vec을 만들지 않고 반복자를
// 돌려주므로, 부르는 쪽이 세든 더하든 앞의 몇 개만 보든 중간 Vec 없이 이어 씁니다.
// (형식이 틀린 줄을 filter_map으로 버려도 되는지는 예제 04에서 따집니다.)
fn cups_in(log: &str) -> impl Iterator<Item = u32> {
    log.lines()
        .filter_map(|line| line.split_once(' '))
        .filter_map(|(_, cups)| cups.parse().ok())
}

// ---- 3. 슬라이스로 받는 것이 맞는 함수 --------------------------------------

// 이웃한 두 날을 비교하려면 windows(2)가 필요하고, windows는 슬라이스의 메서드입니다.
// 반복자로 받으면 본문에서 다시 Vec으로 모아야 하므로, 이때는 &[u32]가 정직한 시그니처입니다.
fn growth_days(cups: &[u32]) -> usize {
    cups.windows(2).filter(|pair| pair[1] > pair[0]).count()
}

fn main() {
    let this_week = vec![132, 98, 120, 151, 117, 128, 86];
    let last_week = vec![90, 125, 119];
    let by_date: BTreeMap<&str, u32> =
        BTreeMap::from([("11-17", 140), ("11-18", 119), ("11-19", 163)]);
    let pos_log = "11-20 128\n11-21 75\n11-22 131";

    println!("== 1. &[u32]판: 부르는 쪽이 먼저 Vec을 만듭니다 ==");
    // Vec이면 그대로 넘깁니다.
    let a = hit_days_slice(&this_week, TARGET);
    // 맵의 값, 두 주를 이은 것, 기록에서 꺼낸 값은 모두 Vec으로 모은 뒤에야 넘길 수 있습니다.
    let values: Vec<u32> = by_date.values().copied().collect();
    let b = hit_days_slice(&values, TARGET);
    let joined: Vec<u32> = [last_week.as_slice(), this_week.as_slice()].concat();
    let c = hit_days_slice(&joined, TARGET);
    let parsed: Vec<u32> = cups_in(pos_log).collect();
    let d = hit_days_slice(&parsed, TARGET);
    println!("이번 주 {a}일, 날짜별 기록 {b}일, 두 주 합쳐 {c}일, 포스기 기록 {d}일");
    // 출력: 이번 주 4일, 날짜별 기록 2일, 두 주 합쳐 5일, 포스기 기록 2일

    println!();
    println!("== 2. impl IntoIterator판: 가진 것을 그대로 넘깁니다 ==");
    let a = hit_days(this_week.iter().copied(), TARGET); // 빌린 Vec: 원소를 값으로
    let b = hit_days(by_date.values().copied(), TARGET); // 맵의 값
    let c = hit_days(last_week.iter().chain(&this_week).copied(), TARGET); // 이어 붙이기만
    let d = hit_days(cups_in(pos_log), TARGET); // 반복자를 돌려주는 함수와 바로 연결
    let e = hit_days([120, 119], TARGET); // 배열
    println!("이번 주 {a}일, 날짜별 기록 {b}일, 두 주 합쳐 {c}일, 포스기 기록 {d}일, 배열 {e}일");
    // 출력: 이번 주 4일, 날짜별 기록 2일, 두 주 합쳐 5일, 포스기 기록 2일, 배열 1일
    let owned = hit_days(this_week, TARGET); // 더 쓰지 않을 Vec은 통째로 넘겨도 됩니다
    println!("이번 주 Vec을 통째로 넘김: {owned}일");
    // 출력: 이번 주 Vec을 통째로 넘김: 4일

    println!();
    println!("== 3. 반복자를 돌려받은 쪽이 고릅니다 ==");
    let first_two: Vec<u32> = cups_in(pos_log).take(2).collect();
    let sum: u32 = cups_in(pos_log).sum();
    println!("앞의 두 줄 {first_two:?}, 합계 {sum}잔");
    // 출력: 앞의 두 줄 [128, 75], 합계 334잔

    println!();
    println!("== 4. 슬라이스가 맞는 경우: 이웃한 원소를 함께 봄 ==");
    let week = [132, 98, 120, 151, 117, 128, 86];
    println!("전날보다 많이 판 날: {}일", growth_days(&week));
    // 출력: 전날보다 많이 판 날: 3일
}
