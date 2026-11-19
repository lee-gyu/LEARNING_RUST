// 03_env_unsafe: 2024 에디션에서 unsafe가 된 함수와, 옮길 때 고를 수 있는 길
//
// env::set_var, env::remove_var, 유닉스의 CommandExt::before_exec는 2021 에디션까지 안전한 함수처럼
// 부를 수 있었지만, 2024 에디션 코드에서는 unsafe 블록 안에서만 부를 수 있습니다. 표준 라이브러리의
// 함수는 하나인데, 부르는 코드의 에디션에 따라 unsafe가 필요한지가 달라집니다(README 2.5절).
// 왜 unsafe인지는 Part 1 ch10 2.4절에서 보았습니다. 이 예제는 옮길 때 고를 수 있는 길을 보여 줍니다.
//
//   1번 구역: 정말 자기 환경을 바꿔야 할 때 — 스레드를 만들기 전에, unsafe 블록과 SAFETY 주석으로
//   2번 구역: 자식 프로세스에게 넘기려는 것일 때 — Command::env, env_remove (unsafe 아님)
//   3번 구역: 함수에 설정을 넘기려는 것일 때 — 환경 변수는 한 번만 읽고 값을 인자로 넘기기
//   4번 구역: (유닉스) before_exec 대신 pre_exec
//
// 2번 구역은 이 예제가 자기 자신을 "child" 인자와 함께 자식 프로세스로 다시 실행합니다.
// CARGO_PKG_NAME은 cargo run이 넣어 주는 환경 변수이므로 cargo run으로 실행합니다.
//
// 실행: cargo run -p p2-ch5-rust-2024-ecosystem --example 03_env_unsafe

use std::env;
use std::process::Command;

const LEVEL_KEY: &str = "P2CH5_LEVEL";

fn main() {
    // 자식 모드: 부모가 넘겨준 환경을 출력하고 끝납니다.
    if env::args().nth(1).as_deref() == Some("child") {
        println!("    자식이 본 {LEVEL_KEY}    = {:?}", env::var(LEVEL_KEY));
        println!(
            "    자식이 본 CARGO_PKG_NAME = {:?}",
            env::var("CARGO_PKG_NAME")
        );
        return;
    }

    println!("1. set_var와 remove_var는 unsafe 블록 안에서 부릅니다");
    // SAFETY: main의 첫머리라 이 프로세스에는 아직 main 스레드 하나뿐이고, 이 예제는 끝까지
    // 스레드를 만들지 않습니다. 환경을 동시에 읽거나 쓰는 다른 스레드가 없으므로 안전합니다.
    unsafe {
        env::set_var(LEVEL_KEY, "info");
    }
    println!("    set_var 뒤:    {:?}", env::var(LEVEL_KEY)); // 출력: set_var 뒤:    Ok("info")
    // SAFETY: 위와 같습니다. 아직 스레드를 만들지 않았습니다.
    unsafe {
        env::remove_var(LEVEL_KEY);
    }
    println!("    remove_var 뒤: {:?}", env::var(LEVEL_KEY)); // 출력: remove_var 뒤: Err(NotPresent)

    println!("2. 자식에게만 넘기기: Command::env와 env_remove는 unsafe가 아닙니다");
    let exe = env::current_exe().expect("실행 파일 경로");
    let output = Command::new(exe)
        .arg("child")
        .env(LEVEL_KEY, "debug") // 자식의 환경에만 더합니다
        .env_remove("CARGO_PKG_NAME") // 자식의 환경에서만 뺍니다
        .output()
        .expect("자식 실행");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    // 출력:     자식이 본 P2CH5_LEVEL    = Ok("debug")
    // 출력:     자식이 본 CARGO_PKG_NAME = Err(NotPresent)
    println!("    부모가 본 {LEVEL_KEY}    = {:?}", env::var(LEVEL_KEY)); // 출력: 부모가 본 P2CH5_LEVEL    = Err(NotPresent)
    println!(
        "    부모가 본 CARGO_PKG_NAME = {:?}",
        env::var("CARGO_PKG_NAME")
    ); // 출력: 부모가 본 CARGO_PKG_NAME = Ok("p2-ch5-rust-2024-ecosystem")

    println!("3. 설정은 한 번 읽어 인자로 넘깁니다");
    // 환경 변수를 읽는 것(env::var)은 안전합니다. 읽은 값을 해석하는 함수를 따로 두면,
    // 다른 값으로 시험할 때 set_var로 환경을 바꿀 필요가 없습니다.
    let from_env = parse_level(env::var(LEVEL_KEY).ok().as_deref());
    println!("    환경에서 읽은 값: {from_env:?}"); // 출력: 환경에서 읽은 값: Info
    for raw in [Some("quiet"), Some("debug"), Some("엉뚱한 값")] {
        println!("    parse_level({raw:?}) = {:?}", parse_level(raw));
    }
    // 출력: parse_level(Some("quiet")) = Quiet
    // 출력: parse_level(Some("debug")) = Debug
    // 출력: parse_level(Some("엉뚱한 값")) = Info

    println!("4. before_exec 대신 pre_exec (유닉스)");
    pre_exec_demo();
}

#[derive(Debug)]
enum Level {
    Quiet,
    Info,
    Debug,
}

/// 값이 없거나 알 수 없는 값이면 Info로 봅니다.
fn parse_level(raw: Option<&str>) -> Level {
    match raw {
        Some("quiet") => Level::Quiet,
        Some("debug") => Level::Debug,
        _ => Level::Info,
    }
}

#[cfg(unix)]
fn pre_exec_demo() {
    use std::io;
    use std::os::unix::process::CommandExt;

    // pre_exec의 클로저는 자식 프로세스가 fork된 뒤, 새 프로그램을 exec하기 직전에 자식 쪽에서
    // 실행됩니다. 이 시점에는 할당이나 잠금처럼 "비동기 시그널 안전"하지 않은 일을 하면 안 되는데,
    // 컴파일러가 이것을 검사할 수 없어서 pre_exec는 처음부터 unsafe 함수입니다. 같은 일을 하던
    // before_exec는 1.37.0에서 deprecated되었고, 2024 에디션에서는 unsafe로도 표시됩니다.
    let mut cmd = Command::new("echo");
    cmd.arg("이 줄은 출력되지 않습니다");
    // SAFETY: 클로저는 할당하지 않고 OS 에러 번호로 io::Error 값을 만들어 돌려주기만 합니다.
    unsafe {
        cmd.pre_exec(|| Err(io::Error::from_raw_os_error(1)));
    }
    // 클로저가 Err를 돌려주면 exec는 일어나지 않고, 그 에러가 부모의 spawn 결과로 전달됩니다.
    match cmd.output() {
        Ok(out) => println!(
            "    실행됨: {}",
            String::from_utf8_lossy(&out.stdout).trim()
        ),
        Err(e) => println!("    exec 전에 멈춤: {e} ({:?})", e.kind()),
    }
    // 출력: exec 전에 멈춤: Operation not permitted (os error 1) (PermissionDenied)
}

#[cfg(not(unix))]
fn pre_exec_demo() {
    println!("    유닉스 계열에서만 있는 함수라 건너뜁니다");
}
