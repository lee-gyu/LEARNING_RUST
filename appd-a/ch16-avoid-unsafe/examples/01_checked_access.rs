//! 지침 1: `unsafe`를 쓰기 전에 표준 라이브러리에서 찾는다 — 경계 검사를 건너뛰고 싶을 때.
//!
//! 등산 앱이 1분마다 기록한 고도(m) 17개로 세 가지를 계산합니다. 오르막 누적 고도,
//! 5분 묶음 평균, 사용자가 고른 시각의 고도입니다. "경계 검사가 아깝다"며
//! `get_unchecked`로 쓴 판(비권장)과, 처음부터 인덱스를 다루지 않는 반복자·슬라이스
//! 메서드로 쓴 판(권장)을 같은 기록에 실행해 결과를 비교합니다.
//!
//! 비권장 판도 건전하게(어떤 입력에도 미정의 동작이 없게) 썼습니다. 그 대가로 블록마다
//! 인덱스가 범위 안이라는 증명을 주석으로 달아야 했습니다. 권장 판에는 그런 증명이 필요 없습니다.
//! 이 파일의 `unsafe` 블록은 Miri로 검사하지 못했습니다(nightly 툴체인을 설치하지 않음).
//!
//! 실행: cargo run -p er-ch16-avoid-unsafe --example 01_checked_access

/// 1분 간격 고도 기록(m). 17개라 5개씩 묶으면 2개가 남습니다.
const ALTITUDES: [i32; 17] = [
    312, 318, 325, 321, 334, 347, 352, 349, 361, 375, 388, 384, 379, 392, 405, 401, 398,
];

/// 몇 분씩 묶어 평균을 낼지.
const GROUP: usize = 5;

/// 비권장: 인덱스로 돌면서 경계 검사를 `get_unchecked`로 건너뜁니다.
mod unchecked {
    use super::GROUP;

    /// 오르막 누적 고도: 바로 앞 기록보다 높아진 만큼만 더합니다.
    pub fn total_climb(alts: &[i32]) -> i32 {
        let mut climb = 0;
        for i in 1..alts.len() {
            // SAFETY: 반복 범위가 1..len이므로 1 <= i < len이고, i - 1도 범위 안입니다.
            let prev = unsafe { *alts.get_unchecked(i - 1) };
            // SAFETY: 위와 같은 이유로 i < len입니다.
            let cur = unsafe { *alts.get_unchecked(i) };
            if cur > prev {
                climb += cur - prev;
            }
        }
        climb
    }

    /// GROUP개씩 묶은 평균. 묶음에 들지 못한 나머지는 버립니다.
    pub fn group_averages(alts: &[i32]) -> Vec<f64> {
        let groups = alts.len() / GROUP;
        let mut averages = Vec::with_capacity(groups);
        for g in 0..groups {
            let mut sum = 0;
            for k in 0..GROUP {
                // SAFETY: g < len / GROUP이고 k < GROUP이므로
                // g * GROUP + k <= (len / GROUP) * GROUP - 1 <= len - 1입니다.
                sum += unsafe { *alts.get_unchecked(g * GROUP + k) };
            }
            averages.push(f64::from(sum) / GROUP as f64);
        }
        averages
    }

    /// minute분의 고도. 범위 밖이면 None.
    pub fn altitude_at(alts: &[i32], minute: usize) -> Option<i32> {
        if minute < alts.len() {
            // SAFETY: 바로 위에서 minute < len을 확인했습니다.
            Some(unsafe { *alts.get_unchecked(minute) })
        } else {
            None
        }
    }
}

/// 권장: 인덱스를 직접 만들지 않으므로 증명할 것도 없습니다.
mod checked {
    use super::GROUP;

    pub fn total_climb(alts: &[i32]) -> i32 {
        // array_windows(1.94.0)는 겹치는 [앞, 뒤] 쌍을 배열로 내놓습니다.
        alts.array_windows()
            .map(|[prev, cur]| (cur - prev).max(0))
            .sum()
    }

    /// 나머지를 버리지 않고 함께 돌려줍니다.
    pub fn group_averages(alts: &[i32]) -> (Vec<f64>, &[i32]) {
        // as_chunks(1.88.0)는 꽉 찬 묶음들(&[[i32; GROUP]])과 남은 원소를 나눠 줍니다.
        let (groups, rest) = alts.as_chunks::<GROUP>();
        let averages = groups
            .iter()
            .map(|g| f64::from(g.iter().sum::<i32>()) / GROUP as f64)
            .collect();
        (averages, rest)
    }

    pub fn altitude_at(alts: &[i32], minute: usize) -> Option<i32> {
        // 비권장 판의 if/else가 하던 일이 get 한 번입니다.
        alts.get(minute).copied()
    }
}

fn show_averages(label: &str, averages: &[f64]) {
    let text: Vec<String> = averages.iter().map(|a| format!("{a:.1}")).collect();
    println!("    {label}: [{}]", text.join(", "));
}

fn main() {
    println!("기록 {}개 (1분 간격)", ALTITUDES.len()); // 출력: 기록 17개 (1분 간격)

    println!("1. 오르막 누적 고도");
    let unsafe_climb = unchecked::total_climb(&ALTITUDES);
    let safe_climb = checked::total_climb(&ALTITUDES);
    println!("    비권장 get_unchecked: {unsafe_climb}m"); // 출력:     비권장 get_unchecked: 109m
    println!("    권장 array_windows:   {safe_climb}m"); // 출력:     권장 array_windows:   109m

    println!("2. {GROUP}분 묶음 평균");
    show_averages(
        "비권장 get_unchecked",
        &unchecked::group_averages(&ALTITUDES),
    );
    // 출력:     비권장 get_unchecked: [322.0, 356.8, 389.6]
    let (averages, rest) = checked::group_averages(&ALTITUDES);
    show_averages("권장 as_chunks      ", &averages);
    // 출력:     권장 as_chunks      : [322.0, 356.8, 389.6]
    println!("    as_chunks가 따로 돌려준 나머지: {rest:?}"); // 출력:     as_chunks가 따로 돌려준 나머지: [401, 398]

    println!("3. 고른 시각의 고도");
    for minute in [3, 16, 20] {
        let a = unchecked::altitude_at(&ALTITUDES, minute);
        let b = checked::altitude_at(&ALTITUDES, minute);
        println!("    {minute}분 → 비권장 {a:?}, 권장 {b:?}");
    }
    // 출력:     3분 → 비권장 Some(321), 권장 Some(321)
    // 출력:     16분 → 비권장 Some(398), 권장 Some(398)
    // 출력:     20분 → 비권장 None, 권장 None
}
