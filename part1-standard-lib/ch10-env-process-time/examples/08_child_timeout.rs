// 자식 프로세스를 기다리는 방법과 제한 시간(timeout)
//
//   1) wait는 끝날 때까지 멈춰 기다리고, try_wait는 기다리지 않고 "아직이면 None"을 돌려줍니다.
//   2) Instant로 자식 프로세스의 실행 시간을 잽니다.
//   3) try_wait와 Instant로 제한 시간이 있는 기다리기를 만들고, 시간을 넘기면 kill로 끝냅니다.
//      kill한 뒤에도 wait로 거둬들여야 합니다.
//
// 실행: cargo run -p p1-ch10-env-process-time --example 08_child_timeout
//
// 자식 프로세스는 이 예제 자신을 "child sleep <밀리초>"로 다시 실행한 것입니다.
// 시간은 "N ms 이상인가" 같은 비교 결과로 출력합니다. 모든 자식은 wait로 거둬들이므로 남지 않습니다.
// 시그널 번호(3번 구역)는 유닉스 계열에서만 출력합니다.

use std::env;
use std::io;
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn sleeper(millis: u64) -> Command {
    let exe = env::current_exe().expect("실행 파일 경로를 알 수 있어야 합니다");
    let mut cmd = Command::new(exe);
    cmd.args(["child", "sleep", &millis.to_string()]);
    // 이 예제의 자식은 출력할 것이 없으므로 표준 출력을 버립니다.
    cmd.stdout(Stdio::null());
    cmd
}

// limit 안에 자식이 끝나면 Some(종료 상태), 끝나지 않으면 None을 돌려줍니다.
// 끝나지 않은 자식을 어떻게 할지는 부른 쪽이 정합니다.
fn wait_timeout(child: &mut Child, limit: Duration) -> io::Result<Option<ExitStatus>> {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        // 너무 자주 물어보면 CPU를 낭비하므로 조금씩 쉬면서 확인합니다(폴링).
        thread::sleep(Duration::from_millis(10));
    }
}

fn run_parent() -> io::Result<()> {
    println!("1. try_wait는 기다리지 않습니다");
    let start = Instant::now();
    let mut child = sleeper(150).spawn()?;
    println!("    바로 물어본 결과: {:?}", child.try_wait()?); // 출력:     바로 물어본 결과: None
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        thread::sleep(Duration::from_millis(10));
    };
    println!("    끝난 뒤의 결과: {status}"); // 출력:     끝난 뒤의 결과: exit status: 0
    println!(
        "    150ms 이상 지남: {}",
        start.elapsed() >= Duration::from_millis(150)
    ); // 출력:     150ms 이상 지남: true
    // 이미 거둬들인 자식에게 다시 wait하면 같은 종료 상태를 돌려줍니다.
    println!("    다시 wait해도 같음: {}", child.wait()? == status); // 출력:     다시 wait해도 같음: true

    println!("2. 자식 프로세스의 실행 시간 재기");
    let start = Instant::now();
    let status = sleeper(200).status()?;
    let took = start.elapsed();
    println!(
        "    {status}, 200ms 이상: {}, 2초 미만: {}",
        took >= Duration::from_millis(200),
        took < Duration::from_secs(2)
    );
    // 출력:     exit status: 0, 200ms 이상: true, 2초 미만: true

    println!("3. 제한 시간을 넘긴 자식은 kill로 끝냅니다");
    let start = Instant::now();
    let mut fast = sleeper(50).spawn()?;
    match wait_timeout(&mut fast, Duration::from_secs(1))? {
        Some(status) => println!("    50ms 자식, 제한 1초: 제때 끝남, {status}"), // 출력:     50ms 자식, 제한 1초: 제때 끝남, exit status: 0
        None => println!("    50ms 자식, 제한 1초: 시간 초과"),
    }

    let mut slow = sleeper(5_000).spawn()?;
    let status = match wait_timeout(&mut slow, Duration::from_millis(200))? {
        Some(status) => status,
        None => {
            println!("    5초 자식, 제한 200ms: 시간 초과 → kill"); // 출력:     5초 자식, 제한 200ms: 시간 초과 → kill
            slow.kill()?; // 유닉스에서는 SIGKILL을 보냅니다. 자식은 정리 작업을 할 기회 없이 끝납니다.
            slow.wait()? // 끝난 자식을 거둬들입니다. 빠뜨리면 좀비 프로세스가 남습니다.
        }
    };
    // 시그널로 끝난 프로세스는 종료 코드가 없습니다.
    println!(
        "    {status}, success() = {}, code() = {:?}",
        status.success(),
        status.code()
    ); // 출력:     signal: 9 (SIGKILL), success() = false, code() = None
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        println!("    signal() = {:?}", status.signal()); // 출력:     signal() = Some(9)
    }
    let took = start.elapsed();
    println!(
        "    5초를 다 기다리지 않음: {}",
        took < Duration::from_secs(2)
    ); // 출력:     5초를 다 기다리지 않음: true
    // 이미 끝난 자식에게 kill해도 에러가 아닙니다.
    println!("    끝난 자식에게 kill: {:?}", slow.kill()); // 출력:     끝난 자식에게 kill: Ok(())

    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 4 && args[1] == "child" && args[2] == "sleep" {
        let millis: u64 = args[3].parse().unwrap_or(0);
        thread::sleep(Duration::from_millis(millis));
        return ExitCode::SUCCESS;
    }
    match run_parent() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("에러: {e}");
            ExitCode::FAILURE
        }
    }
}
