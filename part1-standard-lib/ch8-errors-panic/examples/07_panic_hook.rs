// 패닉 훅(panic hook): 패닉이 일어나면 되감기를 시작하기 전에 실행되는 함수입니다.
// 기본 훅은 "thread '...' panicked at ..." 메시지와 (요청하면) 백트레이스를 표준 에러에 출력합니다.
//
//   1) set_hook으로 훅을 바꾸면 패닉 메시지의 모양을 정할 수 있습니다.
//      훅은 PanicHookInfo로 위치(location)와 페이로드(payload)를 받습니다.
//   2) take_hook은 지금 훅을 떼어 돌려주고 기본 훅을 다시 등록합니다.
//   3) take_hook으로 받은 기본 훅을 새 훅 안에서 부르면 "내 처리 + 기본 출력"을 이어 붙일 수 있습니다.
//   4) 훅은 프로세스 전체에 하나뿐이므로 다른 스레드의 패닉에도 실행됩니다.
//
// 실행: cargo run -p p1-ch8-errors-panic --example 07_panic_hook
//
// 패닉은 모두 catch_unwind나 join으로 붙잡으므로 종료 코드 0으로 끝납니다.
// 훅의 출력은 eprintln!으로 표준 에러에 씁니다. 기본 훅이 출력하는 스레드 ID는 실행할 때마다 달라집니다.

use std::panic::{self, PanicHookInfo};
use std::thread;

// 훅에서 쓸 한 줄 보고서입니다.
fn one_line_report(info: &PanicHookInfo<'_>) -> String {
    let thread = thread::current();
    let name = thread.name().unwrap_or("이름 없음");
    // location()은 Option을 돌려주지만, 문서에 따르면 현재는 늘 Some입니다.
    let place = match info.location() {
        Some(loc) => format!("{}:{}:{}", loc.file(), loc.line(), loc.column()),
        None => "위치 모름".to_string(),
    };
    // payload_as_str()는 페이로드가 &str이나 String이면 Some(&str)을 돌려줍니다(1.91.0).
    // 그 밖의 페이로드는 payload()로 받은 &dyn Any를 직접 다운캐스트합니다.
    let message = match info.payload_as_str() {
        Some(s) => s.to_string(),
        None => match info.payload().downcast_ref::<i32>() {
            Some(code) => format!("(i32 페이로드: {code})"),
            None => "(문자열이 아닌 페이로드)".to_string(),
        },
    };
    format!("스레드 '{name}', 위치 {place}, 메시지: {message}")
}

fn main() {
    println!("1. set_hook으로 패닉 메시지를 바꿉니다");
    // 훅의 타입은 Box<dyn Fn(&PanicHookInfo) + Sync + Send + 'static>입니다.
    // 어느 스레드에서든 불릴 수 있으므로 Sync + Send여야 합니다.
    panic::set_hook(Box::new(|info| {
        eprintln!("[내 훅] {}", one_line_report(info));
    }));
    let _ = panic::catch_unwind(|| {
        let ports: Vec<u16> = Vec::new();
        ports[0] // 범위 밖 인덱스
    });
    // 출력 (표준 에러):
    //     [내 훅] 스레드 'main', 위치 part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:48:14, 메시지: index out of bounds: the len is 0 but the index is 0
    // 기본 훅의 "thread 'main' (...) panicked at ..." 세 줄 대신 훅이 만든 한 줄만 나옵니다.

    println!("2. 페이로드가 문자열이 아니면 payload()를 다운캐스트합니다");
    // panic_any는 아무 타입(Any + Send)의 값을 페이로드로 패닉합니다.
    let _ = panic::catch_unwind(|| panic::panic_any(404_i32));
    // 출력 (표준 에러):
    //     [내 훅] 스레드 'main', 위치 part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:56:36, 메시지: (i32 페이로드: 404)

    println!("3. take_hook은 훅을 떼어 내고 기본 훅을 다시 등록합니다");
    let my_hook = panic::take_hook(); // 1번에서 등록한 훅을 돌려받습니다.
    let _ = panic::catch_unwind(|| panic!("기본 훅으로 돌아왔습니다"));
    // 출력 (표준 에러, 기본 훅):
    //
    //     thread 'main' (스레드 ID) panicked at part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:62:36:
    //     기본 훅으로 돌아왔습니다
    //     note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    drop(my_hook); // 돌려받은 훅은 평범한 Box 값이라 버리거나 다시 set_hook에 넘길 수 있습니다.

    println!("4. 기본 훅을 새 훅 안에서 부르면 두 출력을 이어 붙일 수 있습니다");
    // 지금 등록된 것은 기본 훅이므로 take_hook이 기본 훅을 돌려줍니다.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        eprintln!("[기록] {}", one_line_report(info)); // 예: 로그 파일이나 모니터링으로 보내는 자리
        default_hook(info); // 그다음 원래 메시지를 그대로 출력합니다.
    }));
    let _ = panic::catch_unwind(|| panic!("기록과 기본 출력 둘 다"));
    // 출력 (표준 에러):
    //     [기록] 스레드 'main', 위치 part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:77:36, 메시지: 기록과 기본 출력 둘 다
    //
    //     thread 'main' (스레드 ID) panicked at part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:77:36:
    //     기록과 기본 출력 둘 다
    // "note: run with ..." 줄은 기본 훅이 처음 메시지를 출력할 때(3번) 한 번만 나오므로 여기서는 나오지 않습니다.

    println!("5. 훅은 프로세스 전체에 하나라서 다른 스레드의 패닉에도 실행됩니다");
    let handle = thread::Builder::new()
        .name("worker".to_string())
        .spawn(|| -> u32 { panic!("작업자 스레드에서 실패") })
        .expect("스레드를 만들 수 있어야 합니다");
    let joined = handle.join(); // 스레드가 패닉하면 Err(페이로드)입니다(Part 0 ch15).
    println!("    join 결과가 Err인가: {}", joined.is_err());
    // 출력 (표준 에러):
    //     [기록] 스레드 'worker', 위치 part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:88:28, 메시지: 작업자 스레드에서 실패
    //
    //     thread 'worker' (스레드 ID) panicked at part1-standard-lib/ch8-errors-panic/examples/07_panic_hook.rs:88:28:
    //     작업자 스레드에서 실패
    // 출력 (표준 출력): join 결과가 Err인가: true

    // 훅은 전역 상태이므로 다 쓴 뒤에는 기본 훅으로 되돌려 둡니다.
    let _ = panic::take_hook();
    println!("6. 기본 훅으로 되돌렸습니다");
}
