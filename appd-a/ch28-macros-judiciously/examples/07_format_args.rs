// 지침 5: 서식을 다루는 매크로는 서식을 새로 만들지 말고 `format_args!`에 넘기며, `format!` 문법을 따릅니다.
//
// 로그 매크로를 직접 만든다고 합시다. 조각을 이어 붙이는 문법을 새로 만들면 너비·정밀도·디버그 형식 같은
// `format!`의 기능을 다시 구현해야 하고, 모든 조각을 미리 문자열로 만들어야 합니다.
// `format_args!`는 서식 문자열을 컴파일할 때 검사하고, 값을 실제로 출력할 때까지 서식화를 미룹니다.
//
// 실행: cargo run -p er-ch28-macros-judiciously --example 07_format_args

use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Level {
    Debug,
    Info,
}

/// 이 수준보다 낮은 로그는 버립니다. 디버그 로그는 출력되지 않습니다.
const MIN_LEVEL: Level = Level::Info;

/// 서식화에 비용이 드는 값입니다. `Display::fmt`가 몇 번 불렸는지 셉니다.
struct Snapshot {
    connections: u32,
}

static SNAPSHOT_FORMATTED: AtomicU32 = AtomicU32::new(0);

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        SNAPSHOT_FORMATTED.fetch_add(1, Ordering::Relaxed);
        write!(f, "연결 {}개", self.connections)
    }
}

// 비권장: 조각을 공백으로 이어 붙이는 문법을 새로 만들었습니다(`;` 뒤에 조각을 나열).
// 조각마다 `to_string()`을 부르므로, 수준이 낮아 버려질 로그도 먼저 서식화됩니다.
fn emit_parts(level: Level, parts: &[String]) {
    if level < MIN_LEVEL {
        return;
    }
    println!("  [{level:?}] {}", parts.join(" "));
}

macro_rules! log_v1 {
    ($level:expr; $($part:expr),+ $(,)?) => {
        emit_parts($level, &[$($part.to_string()),+])
    };
}

// 권장: 나머지 토큰을 그대로 `format_args!`에 넘깁니다. 받는 함수는 `fmt::Arguments`를 받아,
// 수준을 먼저 확인한 뒤에만 서식화합니다.
fn emit(level: Level, args: fmt::Arguments<'_>) {
    if level < MIN_LEVEL {
        return;
    }
    println!("  [{level:?}] {args}");
}

macro_rules! log_v2 {
    ($level:expr, $($arg:tt)+) => {
        emit($level, format_args!($($arg)+))
    };
}

fn main() {
    let port = 8080;
    let avg_ms = 12.345_678;
    let snapshot = Snapshot { connections: 3 };

    println!("1. 비권장: 서식을 새로 만든 매크로");
    log_v1!(Level::Info; "포트", port, "에서 대기 중"); // 출력:   [Info] 포트 8080 에서 대기 중
    log_v1!(Level::Info; "평균 응답", avg_ms, "ms"); // 출력:   [Info] 평균 응답 12.345678 ms
    log_v1!(Level::Debug; "상태:", snapshot); // (출력 없음: 디버그 로그는 버려짐)
    let calls = SNAPSHOT_FORMATTED.swap(0, Ordering::Relaxed);
    println!("  버려진 로그를 위해 Snapshot을 서식화한 횟수: {calls}"); // 출력:   버려진 로그를 위해 Snapshot을 서식화한 횟수: 1

    println!("2. 권장: format_args!에 넘기는 매크로");
    // 서식 문자열 안의 {port}처럼 변수를 바로 쓰는 문법, 정밀도 {:.1}, 너비 {:>6}이 모두 그대로 됩니다.
    log_v2!(Level::Info, "포트 {port}에서 대기 중"); // 출력:   [Info] 포트 8080에서 대기 중
    log_v2!(Level::Info, "평균 응답 {avg_ms:.1}ms"); // 출력:   [Info] 평균 응답 12.3ms
    log_v2!(Level::Info, "포트 [{port:>6}]"); // 출력:   [Info] 포트 [  8080]
    log_v2!(Level::Debug, "상태: {snapshot}"); // (출력 없음: 디버그 로그는 버려짐)
    let calls = SNAPSHOT_FORMATTED.swap(0, Ordering::Relaxed);
    println!("  버려진 로그를 위해 Snapshot을 서식화한 횟수: {calls}"); // 출력:   버려진 로그를 위해 Snapshot을 서식화한 횟수: 0

    // 서식 문자열과 인자 개수가 맞지 않으면 println!과 똑같이 컴파일 에러가 부른 곳을 가리킵니다.
    // 예: log_v2!(Level::Info, "{} {}", port);
    //     → error: 2 positional arguments in format string, but there is 1 argument
    log_v2!(Level::Info, "{} = {:?}", "상태", snapshot.connections); // 출력:   [Info] 상태 = 3
}
