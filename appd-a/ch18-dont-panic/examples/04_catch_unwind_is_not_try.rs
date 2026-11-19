//! 지침 4: `catch_unwind`를 예외 처리(try/catch)로 쓰지 않는다.
//!
//! 공용 세탁실의 세탁기 배정표에 같은 요청 넷을 세 방식으로 넣습니다.
//! 1. 비권장: 패닉하는 `start`를 `catch_unwind`로 감싸 예외처럼 처리 → 배정표가 어긋난 채 남음
//! 2. 권장: 실패를 `Result`로 돌려주는 `try_start`
//! 3. 고칠 수 없는 패닉판을 꼭 불러야 할 때: 사본에서 실행하고, 패닉하면 사본을 버림
//!
//! 실행: cargo run -p er-ch18-dont-panic --example 04_catch_unwind_is_not_try
//!
//! 1번과 3번 구역은 일부러 모두 세 번 패닉합니다. `catch_unwind`로 붙잡으므로 종료 코드는 0이지만,
//! 붙잡기 전에 기본 패닉 훅이 실행되어 표준 에러에 패닉 메시지가 세 번 나옵니다.

use std::error::Error;
use std::fmt;
use std::panic::{self, AssertUnwindSafe};

const REQUESTS: [&str; 4] = ["101-1203", "1203", "102-305", "103-1101"];

/// 배정표. 불변식: 쉬는 세탁기와 돌아가는 세탁기를 합하면 언제나 3대입니다.
#[derive(Debug, Clone)]
struct Laundry {
    idle: Vec<u8>,
    running: Vec<(u8, String)>,
}

#[derive(Debug)]
enum StartError {
    BadMember(String),
    AllBusy,
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMember(m) => write!(f, "회원 번호는 '동-호' 형식이어야 합니다: {m:?}"),
            Self::AllBusy => write!(f, "쉬는 세탁기가 없습니다"),
        }
    }
}

impl Error for StartError {}

/// 회원 번호 "101-1203"을 "101동 1203호"로 바꿉니다.
fn label(member: &str) -> Result<String, StartError> {
    let (dong, ho) = member
        .split_once('-')
        .ok_or_else(|| StartError::BadMember(member.to_string()))?;
    Ok(format!("{dong}동 {ho}호"))
}

impl Laundry {
    fn new() -> Laundry {
        Laundry {
            idle: vec![1, 2, 3],
            running: Vec::new(),
        }
    }

    fn machines(&self) -> usize {
        self.idle.len() + self.running.len()
    }

    fn report(&self) {
        let running: Vec<u8> = self.running.iter().map(|(m, _)| *m).collect();
        println!(
            "    세탁기 {}대 (돌아감 {running:?}, 쉼 {:?})",
            self.machines(),
            self.idle
        );
    }

    /// 비권장: 세탁기를 먼저 꺼낸 뒤 실패할 수 있는 일을 하고, 실패는 패닉으로 알립니다.
    fn start(&mut self, member: &str) -> u8 {
        let machine = self.idle.pop().expect("쉬는 세탁기가 있어야 합니다");
        // 여기서 패닉하면 꺼낸 세탁기는 idle에도 running에도 없습니다.
        let label = label(member).expect("회원 번호가 올발라야 합니다");
        self.running.push((machine, label));
        machine
    }

    /// 권장: 실패할 수 있는 일을 먼저 끝내고, 상태는 마지막에 바꿉니다.
    fn try_start(&mut self, member: &str) -> Result<u8, StartError> {
        let label = label(member)?;
        let machine = self.idle.pop().ok_or(StartError::AllBusy)?;
        self.running.push((machine, label)); // 여기부터는 실패할 일이 없습니다.
        Ok(machine)
    }
}

fn main() {
    println!("1. 비권장: 패닉하는 start를 catch_unwind로 감싸 예외처럼 처리");
    let mut laundry = Laundry::new();
    for member in REQUESTS {
        // &mut Laundry를 캡처한 클로저는 UnwindSafe가 아니라서 AssertUnwindSafe 없이는
        // 컴파일되지 않습니다. 이 감싸기가 "패닉 뒤의 상태는 내가 책임진다"는 약속입니다.
        match panic::catch_unwind(AssertUnwindSafe(|| laundry.start(member))) {
            Ok(m) => println!("    {member:?} → {m}번 세탁기"),
            Err(_) => println!("    {member:?} → 실패 (패닉을 붙잡음)"),
        }
    }
    laundry.report();
    // 출력: 세탁기 2대 (돌아감 [3, 1], 쉼 []) — 2번 세탁기가 배정표에서 사라졌습니다.

    println!("2. 권장: 실패를 Result로 돌려주는 try_start");
    let mut laundry = Laundry::new();
    for member in REQUESTS {
        match laundry.try_start(member) {
            Ok(m) => println!("    {member:?} → {m}번 세탁기"),
            Err(e) => println!("    {member:?} → Err: {e}"),
        }
    }
    laundry.report();
    // 출력: 세탁기 3대 (돌아감 [3, 2, 1], 쉼 [])

    println!("3. 고칠 수 없는 패닉판을 불러야 한다면: 사본에서 실행하고 패닉하면 사본을 버림");
    let mut laundry = Laundry::new();
    for member in REQUESTS {
        let mut draft = laundry.clone();
        match panic::catch_unwind(AssertUnwindSafe(|| draft.start(member))) {
            Ok(m) => {
                laundry = draft; // 성공했을 때만 반영합니다.
                println!("    {member:?} → {m}번 세탁기");
            }
            Err(_) => println!("    {member:?} → 실패 (패닉이 지나간 사본은 버림)"),
        }
    }
    laundry.report();
    // 출력: 세탁기 3대 (돌아감 [3, 2, 1], 쉼 [])

    let strategy = if cfg!(panic = "unwind") {
        "unwind(되감기)"
    } else {
        "abort(중단)"
    };
    println!("이 예제의 패닉 전략: {strategy}");
    // 출력: 이 예제의 패닉 전략: unwind(되감기)
}
