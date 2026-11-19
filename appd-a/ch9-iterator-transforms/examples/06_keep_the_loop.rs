//! 예제 06: 명시적 반복문이 더 나은 경우
//!
//! 실행: cargo run -q -p er-ch9-iterator-transforms --example 06_keep_the_loop
//!
//! 두 판의 결과는 언제나 같습니다. 비교할 것은 소스 코드입니다.
//! 1번 구역: 멈추는 이유가 여럿이고 돌려줄 것도 여럿이면 for가 낫습니다.
//! 2번 구역: 같은 일이라도 돌려줄 것이 하나뿐이면 체인으로 충분합니다.
//! 3번 구역: 클로저 안에 큰 match가 들어가면 for + match가 낫습니다.

use std::ops::ControlFlow;

// ---- 1. 엘리베이터 탑승: 탈출 조건 셋, 결과 넷 -----------------------------

const CAPACITY: usize = 4; // 정원(명)
const MAX_KG: u32 = 300; // 적재 한도

/// 엘리베이터를 기다리는 사람. 엘리베이터는 올라가는 중입니다.
struct Waiting {
    name: &'static str,
    kg: u32,
    going_up: bool,
}

/// 탑승을 마감한 이유.
#[derive(Debug, PartialEq)]
enum Stop {
    Full,                     // 정원이 참
    Overweight(&'static str), // 이 사람이 타면 한도를 넘음
}

#[derive(Debug, Default, PartialEq)]
struct Boarding {
    riders: Vec<&'static str>,
    skipped: Vec<&'static str>, // 내려가는 사람
    load_kg: u32,
    stop: Option<Stop>,
}

// 권장: 건너뛰기(continue)와 두 가지 마감(break)이 각자 한 줄씩이고,
// 네 가지 상태를 같은 자리에서 고칩니다.
fn board_loop(queue: &[Waiting]) -> Boarding {
    let mut b = Boarding::default();
    for w in queue {
        if !w.going_up {
            b.skipped.push(w.name);
            continue;
        }
        if b.riders.len() == CAPACITY {
            b.stop = Some(Stop::Full);
            break;
        }
        if b.load_kg + w.kg > MAX_KG {
            b.stop = Some(Stop::Overweight(w.name));
            break;
        }
        b.load_kg += w.kg;
        b.riders.push(w.name);
    }
    b
}

// 비권장: 같은 일을 try_fold 하나로 옮겼습니다. 클로저 안에서는 continue와
// break를 쓸 수 없으므로 ControlFlow::Continue / Break로 바꿔 돌려주고,
// 상태는 매번 받아서 다시 돌려줍니다. 본문은 그대로인데 적을 것만 늘었습니다.
fn board_try_fold(queue: &[Waiting]) -> Boarding {
    let flow = queue.iter().try_fold(Boarding::default(), |mut b, w| {
        if !w.going_up {
            b.skipped.push(w.name);
            return ControlFlow::Continue(b);
        }
        if b.riders.len() == CAPACITY {
            b.stop = Some(Stop::Full);
            return ControlFlow::Break(b);
        }
        if b.load_kg + w.kg > MAX_KG {
            b.stop = Some(Stop::Overweight(w.name));
            return ControlFlow::Break(b);
        }
        b.load_kg += w.kg;
        b.riders.push(w.name);
        ControlFlow::Continue(b)
    });
    match flow {
        ControlFlow::Continue(b) | ControlFlow::Break(b) => b,
    }
}

// ---- 2. 명단만 필요하면 체인으로 충분합니다 ---------------------------------

// 건너뛴 사람도, 마감 이유도 필요 없고 탑승자 명단 하나만 돌려준다면
// 조건마다 어댑터 하나씩으로 나뉘어 읽기 쉽습니다.
fn riders_chain(queue: &[Waiting]) -> Vec<&'static str> {
    queue
        .iter()
        .filter(|w| w.going_up) // 올라가는 사람만
        .take(CAPACITY) // 정원까지
        .scan(0, |load, w| {
            // 한도를 넘는 사람이 나오면 None으로 멈춤
            *load += w.kg;
            (*load <= MAX_KG).then_some(w.name)
        })
        .collect()
}

// ---- 3. TV 리모컨: 클로저 안에 match가 들어가는 체인 -------------------------

const MAX_VOLUME: u8 = 30;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Tv {
    volume: u8,
    muted: bool,
    channel: u16,
    previous: u16, // "이전" 버튼으로 돌아갈 채널
}

// 권장: 버튼마다 갈래 하나. "전원"은 남은 버튼을 무시하고 끝냅니다.
fn press_loop(start: Tv, buttons: &[&str]) -> Tv {
    let mut tv = start;
    for &button in buttons {
        match button {
            "전원" => break,
            "볼륨+" => {
                tv.volume = (tv.volume + 1).min(MAX_VOLUME);
                tv.muted = false; // 볼륨을 올리면 음소거가 풀립니다
            }
            "볼륨-" => tv.volume = tv.volume.saturating_sub(1),
            "음소거" => tv.muted = !tv.muted,
            "이전" => (tv.channel, tv.previous) = (tv.previous, tv.channel),
            other => {
                if let Some(n) = other.strip_prefix("채널 ").and_then(|n| n.parse().ok()) {
                    tv.previous = tv.channel;
                    tv.channel = n;
                }
            }
        }
    }
    tv
}

// 비권장: take_while이 "전원"을 맡고, fold의 클로저가 나머지를 맡습니다.
// 클로저 본문이 곧 반복문 본문이라 체인이 덜어 준 것이 거의 없고,
// "전원" 처리만 본문에서 떨어져 나가 버튼 목록을 두 곳에서 읽어야 합니다.
fn press_chain(start: Tv, buttons: &[&str]) -> Tv {
    buttons
        .iter()
        .take_while(|&&b| b != "전원")
        .fold(start, |mut tv, &button| {
            match button {
                "볼륨+" => {
                    tv.volume = (tv.volume + 1).min(MAX_VOLUME);
                    tv.muted = false;
                }
                "볼륨-" => tv.volume = tv.volume.saturating_sub(1),
                "음소거" => tv.muted = !tv.muted,
                "이전" => (tv.channel, tv.previous) = (tv.previous, tv.channel),
                other => {
                    if let Some(n) = other.strip_prefix("채널 ").and_then(|n| n.parse().ok()) {
                        tv.previous = tv.channel;
                        tv.channel = n;
                    }
                }
            }
            tv
        })
}

fn waiting(name: &'static str, kg: u32, going_up: bool) -> Waiting {
    Waiting { name, kg, going_up }
}

fn main() {
    let queues = [
        (
            "대기열 A",
            vec![
                waiting("가람", 70, true),
                waiting("나리", 80, false),
                waiting("다온", 95, true),
                waiting("루다", 60, true),
                waiting("마루", 90, true),
                waiting("바다", 50, true),
            ],
        ),
        (
            "대기열 B",
            vec![
                waiting("가람", 50, true),
                waiting("나리", 55, true),
                waiting("다온", 45, true),
                waiting("루다", 60, true),
                waiting("마루", 40, true),
            ],
        ),
        (
            "대기열 C",
            vec![waiting("가람", 70, true), waiting("나리", 80, false)],
        ),
    ];

    println!("== 1. 탈출 조건이 여럿이고 결과도 여럿: for판과 try_fold판 ==");
    for (label, queue) in &queues {
        let b = board_loop(queue);
        let same = b == board_try_fold(queue);
        println!(
            "{label} → 탑승 {:?} {}kg, 건너뜀 {:?}, 마감 {:?} (두 판 일치: {same})",
            b.riders, b.load_kg, b.skipped, b.stop
        );
    }
    // 출력: 대기열 A → 탑승 ["가람", "다온", "루다"] 225kg, 건너뜀 ["나리"], 마감 Some(Overweight("마루")) (두 판 일치: true)
    // 출력: 대기열 B → 탑승 ["가람", "나리", "다온", "루다"] 210kg, 건너뜀 [], 마감 Some(Full) (두 판 일치: true)
    // 출력: 대기열 C → 탑승 ["가람"] 70kg, 건너뜀 ["나리"], 마감 None (두 판 일치: true)

    println!();
    println!("== 2. 명단만 필요하면: filter + take + scan ==");
    for (label, queue) in &queues {
        let riders = riders_chain(queue);
        let same = riders == board_loop(queue).riders;
        println!("{label} → {riders:?} (for판의 명단과 일치: {same})");
    }
    // 출력: 대기열 A → ["가람", "다온", "루다"] (for판의 명단과 일치: true)
    // 출력: 대기열 B → ["가람", "나리", "다온", "루다"] (for판의 명단과 일치: true)
    // 출력: 대기열 C → ["가람"] (for판의 명단과 일치: true)

    println!();
    println!("== 3. 클로저 안의 match: for + match판과 take_while + fold판 ==");
    let start = Tv {
        volume: 10,
        muted: false,
        channel: 5,
        previous: 5,
    };
    let buttons = [
        "채널 7",
        "음소거",
        "볼륨+",
        "볼륨+",
        "채널 11",
        "이전",
        "볼륨-",
        "음소거",
        "전원",
        "채널 9",
    ];
    let tv = press_loop(start, &buttons);
    println!(
        "{tv:?} (두 판 일치: {})",
        tv == press_chain(start, &buttons)
    );
    // 출력: Tv { volume: 11, muted: true, channel: 7, previous: 11 } (두 판 일치: true)
}
