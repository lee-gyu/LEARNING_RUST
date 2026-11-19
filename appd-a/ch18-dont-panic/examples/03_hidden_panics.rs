//! 지침 3: 숨은 패닉 지점을 찾아 검사 연산으로 바꾼다.
//!
//! 강의 시청 기록을 한 줄로 요약하는 같은 함수를 두 판으로 씁니다.
//! - `summary_panicky`: 슬라이싱·나눗셈·뺄셈을 그대로 쓴 판. 입력에 따라 네 곳 중 한 곳에서 패닉합니다.
//! - `summary_checked`: 같은 일을 `chars`·`checked_*`·반복자로 하는 판. 실패를 값으로 알립니다.
//!
//! 실행: cargo run -p er-ch18-dont-panic --example 03_hidden_panics
//!
//! 패닉판은 `catch_unwind`로 붙잡아 페이로드를 표준 출력에 보여 줍니다. 같은 메시지가 표준 에러에
//! 한 번 더 나오지 않도록, main의 처음에서 패닉 훅을 아무것도 하지 않는 훅으로 바꿉니다.
//! 정수 오버플로 검사는 디버그 빌드에서만 켜지므로 `--release`로 실행하면 1번 구역의 결과 일부와
//! 2번 구역의 첫 줄이 달라집니다.

use std::any::Any;
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};

struct Course {
    title: &'static str,
    goal_min: u32,
    sessions: Vec<u32>, // 회차별 시청 시간(분)
}

fn summary_panicky(c: &Course) -> String {
    // ① 제목이 10바이트를 넘으면 앞 10바이트만 보입니다. 한글은 한 글자가 3바이트라
    //    10번째 바이트가 글자 중간이면 패닉합니다.
    let title = if c.title.len() > 10 {
        format!("{}…", &c.title[..10])
    } else {
        c.title.to_string()
    };
    // ② 평균: 기록이 없으면 0으로 나눕니다.
    let total: u32 = c.sessions.iter().sum();
    let average = total / c.sessions.len() as u32;
    // ③ 최근 3회: 기록이 3회보다 적으면 len() - 3이 usize 아래로 넘칩니다.
    let recent = &c.sessions[c.sessions.len() - 3..];
    // ④ 남은 시간: 목표보다 많이 보면 u32 아래로 넘칩니다.
    let left = c.goal_min - total;
    format!("{title} | 평균 {average}분 | 최근 {recent:?} | 목표까지 {left}분")
}

fn summary_checked(c: &Course) -> String {
    // ① 글자(char) 단위로 자르면 글자 중간에서 끊길 일이 없습니다.
    let title = if c.title.chars().count() > 10 {
        format!("{}…", c.title.chars().take(10).collect::<String>())
    } else {
        c.title.to_string()
    };
    // 합계도 넘칠 수 있으므로(sum은 디버그 빌드에서 패닉) checked_add로 접습니다.
    let Some(total) = c
        .sessions
        .iter()
        .try_fold(0u32, |acc, &m| acc.checked_add(m))
    else {
        return format!("{title} | 합계가 너무 큼");
    };
    // ② checked_div는 0으로 나누면 None입니다.
    let Some(average) = total.checked_div(c.sessions.len() as u32) else {
        return format!("{title} | 기록 없음");
    };
    // ③ saturating_sub는 0 아래로 내려가지 않고, skip은 길이를 넘어도 패닉하지 않습니다.
    let skip = c.sessions.len().saturating_sub(3);
    let recent: Vec<u32> = c.sessions.iter().skip(skip).copied().collect();
    // ④ checked_sub가 None이면 목표를 넘긴 것입니다. 차이는 abs_diff로 구합니다.
    let left = match c.goal_min.checked_sub(total) {
        Some(m) => format!("목표까지 {m}분"),
        None => format!("목표보다 {}분 더 봄", total.abs_diff(c.goal_min)),
    };
    format!("{title} | 평균 {average}분 | 최근 {recent:?} | {left}")
}

fn panic_message(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("(문자열이 아닌 페이로드)")
}

/// 클로저가 패닉하면 그 메시지를 `Err`로 돌려줍니다.
fn caught<R>(f: impl FnOnce() -> R) -> Result<R, String> {
    panic::catch_unwind(AssertUnwindSafe(f)).map_err(|p| panic_message(&*p).to_string())
}

fn main() {
    // 패닉 메시지는 caught가 표준 출력에 보여 주므로, 기본 훅의 표준 에러 출력은 끕니다.
    panic::set_hook(Box::new(|_| {}));

    let courses = [
        Course {
            title: "Rust 101",
            goal_min: 180,
            sessions: vec![30, 45, 50],
        },
        Course {
            title: "Go 입문",
            goal_min: 120,
            sessions: vec![],
        },
        Course {
            title: "Git 기초",
            goal_min: 90,
            sessions: vec![20, 25],
        },
        Course {
            title: "소유권과 대여 완전 정복",
            goal_min: 200,
            sessions: vec![40, 50, 35],
        },
        Course {
            title: "Rust 101",
            goal_min: 100,
            sessions: vec![30, 45, 50],
        },
    ];

    println!("1. 같은 요약 함수의 패닉판과 검사판");
    for c in &courses {
        let n = c.sessions.len();
        println!("    [{}] 기록 {n}회, 목표 {}분", c.title, c.goal_min);
        match caught(|| summary_panicky(c)) {
            Ok(line) => println!("      패닉판: {line}"),
            Err(message) => println!("      패닉판: 패닉 — {message}"),
        }
        println!("      검사판: {}", summary_checked(c));
    }
    // 출력: 첫 강의만 두 판이 같고, 나머지 넷은 패닉판이 ②·③·①·④에서 차례로 패닉합니다.

    println!("2. 오버플로는 빌드에 따라 패닉하거나 조용히 틀린 값이 됨");
    let (goal, total) = (100u32, 125u32);
    let build = if cfg!(debug_assertions) {
        "디버그"
    } else {
        "릴리스"
    };
    match caught(|| goal - total) {
        Ok(v) => println!("    {build} 빌드의 goal - total → {v}"),
        Err(message) => println!("    {build} 빌드의 goal - total → 패닉: {message}"),
    }
    println!(
        "    wrapping_sub(릴리스 빌드의 -와 같음) → {}",
        goal.wrapping_sub(total)
    );
    match caught(|| goal.strict_sub(total)) {
        Ok(v) => println!("    strict_sub(두 빌드 모두 검사) → {v}"),
        Err(message) => println!("    strict_sub(두 빌드 모두 검사) → 패닉: {message}"),
    }
    println!(
        "    checked_sub → {:?}, saturating_sub → {}",
        goal.checked_sub(total),
        goal.saturating_sub(total)
    );
    // 출력: 디버그 빌드의 goal - total → 패닉: attempt to subtract with overflow
    //       wrapping_sub(릴리스 빌드의 -와 같음) → 4294967271

    println!("3. RefCell: 빌려 둔 동안 고치려 하면 패닉");
    let log = RefCell::new(vec![30, 45]);
    let reading = log.borrow(); // 요약을 만드는 동안 기록을 빌려 둡니다.
    match caught(|| log.borrow_mut().push(50)) {
        Ok(()) => println!("    borrow_mut → 추가함"),
        Err(message) => println!("    borrow_mut → 패닉: {message}"),
    }
    match log.try_borrow_mut() {
        Ok(mut sessions) => sessions.push(50),
        Err(e) => println!("    try_borrow_mut → Err: {e}"),
    }
    println!("    빌려 둔 기록: {:?}", *reading);
    drop(reading); // 대여를 끝내면 고칠 수 있습니다.
    if let Ok(mut sessions) = log.try_borrow_mut() {
        sessions.push(50);
    }
    println!("    대여를 끝낸 뒤 try_borrow_mut → {:?}", log.borrow());
    // 출력: borrow_mut → 패닉: RefCell already borrowed
    //       try_borrow_mut → Err: RefCell already borrowed
}
