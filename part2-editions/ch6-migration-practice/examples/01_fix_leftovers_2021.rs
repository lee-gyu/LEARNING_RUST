//! 2018 → 2021 `cargo fix --edition`이 남긴 모양과, 사람이 다듬은 모양을 나란히 실행합니다.
//!
//! 실행: cargo run -p p2-ch6-migration-practice --example 01_fix_leftovers_2021
//!
//! 이 파일은 저장소의 다른 예제처럼 2024 에디션으로 컴파일됩니다. 두 모양 모두 2021 이후의
//! 에디션에서 컴파일되므로, 결과가 같은지 다른지를 같은 에디션 안에서 비교할 수 있습니다.

use std::cell::RefCell;
use std::panic;

/// 버려질 때 이름을 기록하는 값입니다. 정산 작업의 "구간"이 언제 끝나는지 보려고 씁니다.
struct Scope<'a> {
    log: &'a RefCell<Vec<String>>,
    name: &'static str,
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        self.log.borrow_mut().push(format!("{} 끝", self.name));
    }
}

struct Job<'a> {
    title: String,
    _scope: Scope<'a>,
}

fn new_job<'a>(log: &'a RefCell<Vec<String>>) -> Job<'a> {
    Job {
        title: String::from("정산"),
        _scope: Scope {
            log, name: "구간"
        },
    }
}

/// cargo fix가 넣은 모양: 더미 let으로 `job` 전체를 캡처해 2018 에디션과 같은 drop 시점을 지킵니다.
fn fixed_shape(drop_early: bool) -> Vec<String> {
    let log = RefCell::new(Vec::new());
    {
        let job = new_job(&log);
        let header = move || {
            let _ = &job;
            format!("== {} ==", job.title)
        };
        log.borrow_mut().push(header());
        if drop_early {
            drop(header); // job 전체가 클로저 안에 있으므로 여기서 "구간 끝"
        }
        log.borrow_mut().push(String::from("보고서 완성"));
    } // 클로저를 먼저 버리지 않았다면 여기서 클로저와 함께 "구간 끝"
    log.into_inner()
}

/// 다듬은 모양: 2021 에디션의 분리 캡처로 클로저는 `job.title`만 가져갑니다.
fn cleaned_shape(drop_early: bool) -> Vec<String> {
    let log = RefCell::new(Vec::new());
    {
        let job = new_job(&log);
        let header = move || format!("== {} ==", job.title);
        log.borrow_mut().push(header());
        if drop_early {
            drop(header); // job.title만 버려지고 job._scope는 남습니다
        }
        log.borrow_mut().push(String::from("보고서 완성"));
    } // 여기서 job._scope가 버려져 "구간 끝"
    log.into_inner()
}

/// 패닉 페이로드를 문자열로 꺼냅니다. 기본 훅이 표준 에러에 출력하지 않도록 잠시 바꿔 둡니다.
fn panic_message(f: impl FnOnce() + panic::UnwindSafe) -> String {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let payload = panic::catch_unwind(f).unwrap_err();
    panic::set_hook(previous);
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        String::from("(문자열이 아닌 페이로드)")
    }
}

/// 0~100만 백분율로 인정하는 사용자 트레이트(TryInto가 없던 시절의 이름)
trait Percent {
    fn try_into(&self) -> Result<u8, String>;
}

impl Percent for i64 {
    fn try_into(&self) -> Result<u8, String> {
        if (0..=100).contains(self) {
            Ok(*self as u8)
        } else {
            Err(format!("{self}%는 백분율이 아닙니다"))
        }
    }
}

const NAMES: [&str; 3] = ["식비", "교통", "기타"];

fn main() {
    println!("1. 더미 let: 클로저가 블록 끝까지 살아 있을 때");
    println!("    cargo fix 모양: {:?}", fixed_shape(false));
    println!("    다듬은 모양:    {:?}", cleaned_shape(false));
    // 출력:     cargo fix 모양: ["== 정산 ==", "보고서 완성", "구간 끝"]
    // 출력:     다듬은 모양:    ["== 정산 ==", "보고서 완성", "구간 끝"]

    println!("2. 더미 let: 클로저를 먼저 버릴 때");
    println!("    cargo fix 모양: {:?}", fixed_shape(true));
    println!("    다듬은 모양:    {:?}", cleaned_shape(true));
    // 출력:     cargo fix 모양: ["== 정산 ==", "구간 끝", "보고서 완성"]
    // 출력:     다듬은 모양:    ["== 정산 ==", "보고서 완성", "구간 끝"]

    println!("3. 인자가 하나뿐이던 panic!");
    let line = "식비 abc";
    let fixed = panic_message(|| panic!("{}", "잘못된 줄: {line}"));
    let cleaned = panic_message(|| panic!("잘못된 줄: {line}"));
    println!("    cargo fix 모양: {fixed}");
    println!("    다듬은 모양:    {cleaned}");
    // 출력:     cargo fix 모양: 잘못된 줄: {line}
    // 출력:     다듬은 모양:    잘못된 줄: 식비 abc

    println!("4. 사용자 트레이트의 try_into와 프렐류드의 TryInto");
    let share: i64 = 150;
    let by_path = Percent::try_into(&share); // cargo fix 모양: 트레이트 이름을 적어 부름
    let by_method: Result<u8, _> = share.try_into(); // 2021 이후 이 줄은 TryInto::try_into를 부름
    println!("    Percent::try_into(&share) = {by_path:?}");
    println!("    share.try_into()          = {by_method:?}");
    // 출력:     Percent::try_into(&share) = Err("150%는 백분율이 아닙니다")
    // 출력:     share.try_into()          = Ok(150)

    println!("5. 배열의 .into_iter()");
    // cargo fix 모양: 2018의 .into_iter()(참조를 내놓음)를 .iter()로 바꿔 그대로 둠
    let fixed = NAMES.iter().find(|n| n.starts_with('교')).copied();
    // 다듬은 모양: 2021 이후 배열의 .into_iter()는 원소를 값으로 내놓으므로 .copied()가 필요 없음
    let cleaned = NAMES.into_iter().find(|n| n.starts_with('교'));
    println!("    {fixed:?} == {cleaned:?}: {}", fixed == cleaned);
    // 출력:     Some("교통") == Some("교통"): true
}
