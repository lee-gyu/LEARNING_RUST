// 제네릭 함수는 쓰인 타입마다 본문 전체가 복사됩니다. 본문이 길고 부르는 타입이 많으면
// 그만큼 코드가 늘어나므로, 제네릭 부분은 얇은 껍데기로 두고 실제 일은 한 벌만 있는
// 제네릭이 아닌 함수에 맡깁니다. 안쪽 함수는 구체 타입(`&str`)이나 트레이트 객체
// (`&mut dyn fmt::Write`)를 받습니다.
//
// 실행: cargo run -p er-ch12-generics-vs-trait-objects --example 02_thin_generic_shell
//
// 만들어진 함수의 수는 실행 결과로는 보이지 않습니다. README 2.3절의 `nm` 명령으로
// 빌드한 바이너리의 함수 이름을 세어 확인합니다.

use std::borrow::Cow;
use std::fmt::{self, Write};
use std::rc::Rc;

// ── 1. 문자열을 여러 모양으로 받는 함수 ─────────────────────────────

/// 껍데기가 두꺼운 꼴: 본문 전체가 `S`에 대해 제네릭이라, 부르는 `S`마다 본문 전체가
/// 한 벌씩 만들어집니다. 본문이 `S`에 기대는 곳은 첫 줄의 `as_ref()`뿐입니다.
fn shelf_label_fat<S: AsRef<str>>(title: S) -> String {
    let title = title.as_ref();
    let words: Vec<&str> = title.split_whitespace().collect();
    let initials: String = words.iter().filter_map(|w| w.chars().next()).collect();
    let cleaned = words.join(" ");
    format!("{cleaned} [{initials}-{:02}]", cleaned.chars().count())
}

/// 껍데기가 얇은 꼴: 제네릭 부분은 변환 한 줄입니다.
fn shelf_label<S: AsRef<str>>(title: S) -> String {
    shelf_label_str(title.as_ref())
}

/// 실제 일은 제네릭이 아닌 함수가 합니다. 이 본문은 바이너리에 한 벌만 있습니다.
fn shelf_label_str(title: &str) -> String {
    let words: Vec<&str> = title.split_whitespace().collect();
    let initials: String = words.iter().filter_map(|w| w.chars().next()).collect();
    let cleaned = words.join(" ");
    format!("{cleaned} [{initials}-{:02}]", cleaned.chars().count())
}

// ── 2. 출력 대상을 여러 타입으로 받는 함수 ────────────────────────────

/// 껍데기가 두꺼운 꼴: 출력 대상 `W`마다 본문 전체가 복사됩니다.
fn write_slip_fat<W: Write>(out: &mut W, title: &str, days: u32) -> fmt::Result {
    writeln!(out, "대출증")?;
    writeln!(out, "  자료: {title}")?;
    writeln!(out, "  기간: {days}일")
}

/// 껍데기가 얇은 꼴: `&mut W`를 `&mut dyn Write`로 바꿔 넘기기만 합니다.
/// 부르는 쪽은 여전히 `&mut String`처럼 구체 타입을 그대로 넘깁니다.
fn write_slip<W: Write>(out: &mut W, title: &str, days: u32) -> fmt::Result {
    write_slip_dyn(out, title, days)
}

/// 본문은 트레이트 객체를 받으므로 한 벌뿐입니다. `out`에 대한 호출은 동적 디스패치입니다.
fn write_slip_dyn(out: &mut dyn Write, title: &str, days: u32) -> fmt::Result {
    writeln!(out, "대출증")?;
    writeln!(out, "  자료: {title}")?;
    writeln!(out, "  기간: {days}일")
}

/// 받은 글자 수만 세는 출력 대상. `String`과 다른 두 번째 `Write` 구현입니다.
struct CharCounter(usize);

impl Write for CharCounter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0 += s.chars().count();
        Ok(())
    }
}

fn main() -> fmt::Result {
    let owned = String::from("러스트   프로그래밍 공식 가이드");
    let boxed: Box<str> = "함께 배우는 러스트".into();
    let shared: Rc<str> = Rc::from("러스트 동시성");
    let cow: Cow<str> = Cow::Borrowed("러스트 실전");

    println!("1. 문자열을 여러 모양으로 받는 함수 (부르는 타입 5가지)");
    // 두 꼴의 결과는 같습니다. 달라지는 것은 바이너리 안의 함수 수입니다.
    let fat = [
        shelf_label_fat("러스트 입문"),
        shelf_label_fat(&owned),
        shelf_label_fat(boxed.clone()),
        shelf_label_fat(Rc::clone(&shared)),
        shelf_label_fat(cow.clone()),
    ];
    let thin = [
        shelf_label("러스트 입문"),
        shelf_label(&owned),
        shelf_label(boxed),
        shelf_label(shared),
        shelf_label(cow),
    ];
    for label in &thin {
        println!("    {label}");
    }
    // 출력:     러스트 입문 [러입-06]
    // 출력:     러스트 프로그래밍 공식 가이드 [러프공가-16]
    // 출력:     함께 배우는 러스트 [함배러-10]
    // 출력:     러스트 동시성 [러동-07]
    // 출력:     러스트 실전 [러실-06]
    println!("    두 꼴의 결과가 같은가: {}", fat == thin);
    // 출력:     두 꼴의 결과가 같은가: true

    println!("2. 출력 대상을 여러 타입으로 받는 함수 (출력 대상 2가지)");
    // 두 꼴 모두 `String`과 `CharCounter`, 두 타입으로 부릅니다.
    let mut slip_fat = String::new();
    let mut slip_thin = String::new();
    let mut counter_fat = CharCounter(0);
    let mut counter_thin = CharCounter(0);
    write_slip_fat(&mut slip_fat, "우주 다큐", 7)?;
    write_slip_fat(&mut counter_fat, "우주 다큐", 7)?;
    write_slip(&mut slip_thin, "우주 다큐", 7)?;
    write_slip(&mut counter_thin, "우주 다큐", 7)?;
    for line in slip_thin.lines() {
        println!("    | {line}");
    }
    // 출력:     | 대출증
    // 출력:     |   자료: 우주 다큐
    // 출력:     |   기간: 7일
    println!(
        "    두 꼴의 결과가 같은가: {}, 센 글자 수: {} / {}",
        slip_fat == slip_thin,
        counter_fat.0,
        counter_thin.0
    );
    // 출력:     두 꼴의 결과가 같은가: true, 센 글자 수: 25 / 25
    Ok(())
}
