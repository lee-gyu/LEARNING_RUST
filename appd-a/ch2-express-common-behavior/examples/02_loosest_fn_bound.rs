// 지침 2: 받는 쪽은 자신이 클로저를 부르는 방식이 허락하는 가장 느슨한 `Fn*` 바운드를 고릅니다.
//
//   - 많아야 한 번 부른다                      → FnOnce
//   - 여러 번 부르지만 한 번에 한 곳에서만 부른다 → FnMut
//   - 공유된 채로(`&self` 메서드 안 등) 부른다   → Fn
//
// 필요 이상으로 엄격한 `Fn`을 요구하면, 부르는 쪽은 값을 바꾸거나 옮기는 클로저를 넘기지 못해
// `Cell`/`RefCell`과 `Option::take` 같은 우회로를 써야 합니다.
//
// 실행: cargo run -p er-ch2-express-common-behavior --example 02_loosest_fn_bound

use std::cell::{Cell, RefCell};

/// 작업이 끝나면 넘겨줄 보고서입니다. 일부러 `Clone`을 구현하지 않았습니다.
/// "정확히 한 번 넘겨야 하는 값"을 나타냅니다.
struct Report {
    title: String,
    steps: usize,
}

fn titles(reports: &[Report]) -> Vec<String> {
    reports
        .iter()
        .map(|r| format!("{} ({}단계)", r.title, r.steps))
        .collect()
}

/// 비권장: 단계마다 부르는 콜백과 끝에 한 번 부르는 콜백을 모두 `Fn`으로 요구합니다.
fn run_job_strict<S, D>(steps: &[&str], on_step: S, on_done: D)
where
    S: Fn(usize, &str),
    D: Fn(usize),
{
    for (i, step) in steps.iter().enumerate() {
        on_step(i + 1, step);
    }
    on_done(steps.len());
}

/// 권장: 본문은 비권장 판과 똑같고 바운드만 다릅니다.
/// `on_step`은 차례로 여러 번 부르므로 `FnMut`, `on_done`은 마지막에 한 번만 부르므로 `FnOnce`.
fn run_job<S, D>(steps: &[&str], mut on_step: S, on_done: D)
where
    S: FnMut(usize, &str),
    D: FnOnce(usize),
{
    for (i, step) in steps.iter().enumerate() {
        on_step(i + 1, step);
    }
    on_done(steps.len());
}

/// `Fn`이 맞는 경우: 규칙을 필드에 보관하고 `&self` 메서드에서 부릅니다.
/// `&self`로는 필드를 공유 대여로만 쓸 수 있으므로, 가변 대여가 필요한 `FnMut`은 부를 수 없습니다.
struct Gate<F> {
    allow: F,
}

impl<F: Fn(u32) -> bool> Gate<F> {
    fn check(&self, age: u32) -> &'static str {
        if (self.allow)(age) {
            "통과"
        } else {
            "거절"
        }
    }
}

fn main() {
    let steps = ["압축", "전송", "검증"];

    println!("1. 비권장: Fn만 받는 함수에 맞추느라 우회로를 씁니다");
    // 로그에 줄을 추가하려면 값을 바꿔야 하는데 Fn 클로저는 캡처한 값을 바꿀 수 없으므로
    // 내부 가변성(RefCell)으로 감쌉니다.
    let log = RefCell::new(Vec::new());
    let outbox = RefCell::new(Vec::new());
    // 보고서를 클로저 밖으로 옮길 수도 없으므로 Option에 넣어 두고 take로 꺼냅니다.
    let draft = Cell::new(Some(Report {
        title: String::from("야간 백업"),
        steps: 0,
    }));
    run_job_strict(
        &steps,
        |i, name| log.borrow_mut().push(format!("{i}. {name}")),
        |n| {
            // 시그니처만 보면 on_done이 여러 번 불릴 수도 있으므로,
            // 이미 꺼낸 뒤(None)의 경우도 부르는 쪽이 생각해야 합니다.
            if let Some(mut report) = draft.take() {
                report.steps = n;
                outbox.borrow_mut().push(report);
            }
        },
    );
    println!("    log = {:?}", log.into_inner());
    println!("    outbox = {:?}", titles(&outbox.into_inner()));
    // 출력:     log = ["1. 압축", "2. 전송", "3. 검증"]
    // 출력:     outbox = ["야간 백업 (3단계)"]

    println!("2. 권장: 가장 느슨한 바운드라 평범한 클로저를 그대로 넘깁니다");
    let mut log = Vec::new();
    let mut outbox = Vec::new();
    let draft = Report {
        title: String::from("야간 백업"),
        steps: 0,
    };
    run_job(
        &steps,
        |i, name| log.push(format!("{i}. {name}")), // log를 바꾸는 FnMut 클로저
        |n| {
            let mut report = draft; // draft를 클로저 밖으로 옮기는 FnOnce 클로저
            report.steps = n;
            outbox.push(report);
        },
    );
    println!("    log = {log:?}");
    println!("    outbox = {:?}", titles(&outbox));
    // 출력:     log = ["1. 압축", "2. 전송", "3. 검증"]
    // 출력:     outbox = ["야간 백업 (3단계)"]

    println!("3. 느슨한 바운드도 Fn 클로저를 그대로 받습니다");
    // 아무것도 바꾸지 않는 클로저(Fn)는 FnMut·FnOnce 자리에도 들어갑니다.
    let prefix = "단계";
    run_job(
        &steps[..2],
        |i, name| println!("    {prefix} {i}: {name}"),
        |n| println!("    {prefix} {n}개 끝"),
    );
    // 출력:     단계 1: 압축
    // 출력:     단계 2: 전송
    // 출력:     단계 2개 끝

    println!("4. Fn이 맞는 경우: &self 메서드에서 부르는 규칙");
    let min_age = 19;
    let gate = Gate {
        allow: |age: u32| age >= min_age,
    };
    let first = &gate;
    let second = &gate; // 같은 값을 여러 곳에서 공유 대여로 쓸 수 있습니다.
    println!("    17세 {}, 25세 {}", first.check(17), second.check(25));
    // 출력:     17세 거절, 25세 통과
}
