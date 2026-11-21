// Q6.3 · Q6.7 — 생성자 오버로딩, 가변 인자, 타입별 오버로딩 대신 쓰는 것
//
// Rust에는 같은 이름의 함수를 매개변수만 바꿔 여러 개 두는 오버로딩이 없습니다
// (compile_fail/e0592_overloaded_new.rs). 생성자도 따로 없고, `new`는 관례로 붙이는 이름일 뿐입니다.
// Java·C#·C++에서 오버로딩으로 하던 일을 이렇게 옮깁니다.
//
//   1. 생성자 오버로딩 → 이름이 다른 연관 함수(`new`, `from_minutes`). `impl Into<String>`으로
//      `&str`과 `String`을 모두 받음
//   2. 가변 인자(Java의 `int...`) → 슬라이스 `&[T]`
//   3. 인자 타입에 따른 오버로딩 → 그 타입들이 구현하는 트레이트
//
// 기본 인자 대신 쓰는 `Default`와 구조체 갱신 문법, 빌더는 부록 A ch7에서 다룹니다.
//
// 실행: cargo run -p faq-ch6-no-classes-inheritance --example 03_overloading_alternatives

use std::time::Duration;

struct Timer {
    label: String,
    duration: Duration,
}

impl Timer {
    fn new(label: impl Into<String>, secs: u64) -> Timer {
        Timer {
            label: label.into(),
            duration: Duration::from_secs(secs),
        }
    }

    fn from_minutes(label: impl Into<String>, minutes: u64) -> Timer {
        Timer::new(label, minutes * 60)
    }
}

/// 개수가 정해지지 않은 인자 대신 슬라이스를 받습니다.
fn total(timers: &[Timer]) -> Duration {
    timers.iter().map(|t| t.duration).sum()
}

/// "초로 바꿀 수 있는 것". `wait(90)`과 `wait(Duration)`을 둘 다 받으려고 만든 트레이트입니다.
trait IntoSecs {
    fn into_secs(self) -> u64;
}

impl IntoSecs for u64 {
    fn into_secs(self) -> u64 {
        self
    }
}

impl IntoSecs for Duration {
    fn into_secs(self) -> u64 {
        self.as_secs() // 1초 미만은 버립니다.
    }
}

fn wait(time: impl IntoSecs) -> String {
    format!("{}초 기다림", time.into_secs())
}

fn main() {
    println!("1. 생성자 오버로딩 대신 이름이 다른 연관 함수");
    let tea = Timer::new("차", 180); // &str
    let meeting = Timer::from_minutes(String::from("회의"), 30); // String
    println!("  {}: {:?}", tea.label, tea.duration);
    println!("  {}: {:?}", meeting.label, meeting.duration);
    // 출력:   차: 180s
    // 출력:   회의: 1800s

    println!("2. 가변 인자 대신 슬라이스");
    println!("  합계: {:?}", total(&[tea, meeting]));
    println!("  빈 목록: {:?}", total(&[]));
    // 출력:   합계: 1980s
    // 출력:   빈 목록: 0ns

    println!("3. 인자 타입에 따른 오버로딩 대신 트레이트");
    // 90은 IntoSecs를 구현한 정수 타입이 u64뿐이라 u64로 정해집니다. u32에도 구현하면 리터럴이 i32가 되어 E0277.
    println!("  {}", wait(90));
    println!("  {}", wait(Duration::from_millis(2500)));
    // 출력:   90초 기다림
    // 출력:   2초 기다림
}
