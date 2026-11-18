// Command로 자식 프로세스(child process)를 만들고 실행합니다.
//
//   1) 인자는 셸을 거치지 않고 한 개씩 그대로 전달됩니다. 공백, *, $, ;, | 가 해석되지 않습니다.
//   2) 자식은 부모의 환경 변수를 물려받고, env / env_remove / env_clear로 자식 몫만 바꿀 수 있습니다.
//   3) current_dir로 자식의 현재 디렉터리를 정합니다.
//   4) 실행하는 세 가지 방법: status(출력은 그대로 흘려보냄), output(출력을 모아 옴), spawn(기다리지 않음)
//   5) 프로그램을 실행하지 못한 것(io::Error)과 자식이 실패한 것(0이 아닌 종료 상태)은 다릅니다.
//   6) (macOS/Linux) 셸 문법이 필요하면 sh -c로 셸을 직접 실행합니다.
//
// 실행: cargo run -p p1-ch10-env-process-time --example 03_command
//
// 자식 프로세스는 이 예제 자신(current_exe)을 "child <모드>" 인자로 다시 실행한 것입니다.
// 6번 구역만 유닉스 명령(sh, echo, tr)을 씁니다.

use std::env;
use std::fs;
use std::io;
use std::process::{Command, ExitCode};

// ---- 자식 모드 ----

fn child_main(mode: &str, rest: &[String]) -> ExitCode {
    match mode {
        // 받은 인자를 디버그 형식으로 보여 줍니다. 따옴표 안이 인자 하나입니다.
        "args" => println!("[자식] 받은 인자 {}개: {rest:?}", rest.len()),
        "env" => {
            let color = env::var("P1CH10_COLOR").unwrap_or_else(|_| "(없음)".to_string());
            let has_path = env::var_os("PATH").is_some();
            let count = env::vars_os().count();
            // 환경 변수의 개수는 환경마다 다르므로 0인지만 출력합니다.
            println!(
                "[자식] P1CH10_COLOR={color}, PATH 있음={has_path}, 환경 변수 0개={}",
                count == 0
            );
        }
        "cwd" => {
            let cwd = env::current_dir().expect("현재 디렉터리");
            println!(
                "[자식] 현재 디렉터리 이름: {:?}",
                cwd.file_name().unwrap_or_default()
            );
        }
        "hello" => {
            println!("[자식] 표준 출력으로 한 줄");
            eprintln!("[자식] 표준 에러로 한 줄");
        }
        "fail" => return ExitCode::from(3),
        _ => return ExitCode::from(64),
    }
    ExitCode::SUCCESS
}

// ---- 부모 ----

// 이 예제 자신을 자식 모드로 실행하는 Command를 만듭니다.
// 빌더 메서드(arg, args, env ...)는 &mut Command를 받아 &mut Command를 돌려줍니다.
// 그래서 Command 값을 돌려주려면 먼저 변수에 담고, 설정한 뒤, 변수를 돌려줍니다.
fn child(mode: &str) -> Command {
    let exe = env::current_exe().expect("실행 파일 경로를 알 수 있어야 합니다");
    let mut cmd = Command::new(exe);
    cmd.args(["child", mode]);
    cmd
}

// output()으로 실행하고 자식의 표준 출력을 한 줄 문자열로 돌려줍니다.
fn stdout_of(cmd: &mut Command) -> String {
    let output = cmd.output().expect("자식을 실행할 수 있어야 합니다");
    // 표준 출력은 바이트(Vec<u8>)입니다. 문자열로 쓰려면 UTF-8로 해석해야 합니다.
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 && args[1] == "child" {
        return child_main(&args[2], &args[3..]);
    }
    // 부모의 본문은 io::Result를 돌려주는 함수로 빼고, main은 결과를 종료 코드로 바꾸기만 합니다.
    match run_parent() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("에러: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_parent() -> io::Result<()> {
    println!("1. 인자는 한 개씩, 셸 해석 없이 전달됩니다");
    let mut cmd = child("args");
    cmd.arg("공백 있는 인자")
        .arg("*.rs")
        .arg("$HOME")
        .args(["a; echo 주입", "x | y"]);
    println!("    {}", stdout_of(&mut cmd));
    // 출력:     [자식] 받은 인자 5개: ["공백 있는 인자", "*.rs", "$HOME", "a; echo 주입", "x | y"]
    // 디버그 형식은 셸 명령처럼 보여 주지만 경로가 들어 있으므로 프로그램 이름은 빼고 인자만 봅니다.
    let shown: Vec<_> = cmd.get_args().collect();
    println!("    get_args(): {shown:?}");
    // 출력:     get_args(): ["child", "args", "공백 있는 인자", "*.rs", "$HOME", "a; echo 주입", "x | y"]

    println!("2. 환경 변수는 물려받고, 자식 몫만 바꿀 수 있습니다");
    println!("    그대로:      {}", stdout_of(&mut child("env")));
    println!(
        "    env 추가:    {}",
        stdout_of(child("env").env("P1CH10_COLOR", "blue"))
    );
    println!(
        "    env_remove:  {}",
        stdout_of(child("env").env_remove("PATH"))
    );
    println!("    env_clear:   {}", stdout_of(child("env").env_clear()));
    // 출력:     그대로:      [자식] P1CH10_COLOR=(없음), PATH 있음=true, 환경 변수 0개=false
    // 출력:     env 추가:    [자식] P1CH10_COLOR=blue, PATH 있음=true, 환경 변수 0개=false
    // 출력:     env_remove:  [자식] P1CH10_COLOR=(없음), PATH 있음=false, 환경 변수 0개=false
    // 출력:     env_clear:   [자식] P1CH10_COLOR=(없음), PATH 있음=false, 환경 변수 0개=true
    // Command::env는 자식의 환경만 정합니다. 부모 자신의 환경은 그대로입니다.
    println!("    부모의 P1CH10_COLOR: {:?}", env::var("P1CH10_COLOR")); // 출력:     부모의 P1CH10_COLOR: Err(NotPresent)

    println!("3. current_dir로 자식의 현재 디렉터리를 정합니다");
    let workdir = env::temp_dir().join("p1_ch10_03_workdir");
    fs::create_dir_all(&workdir)?;
    println!("    {}", stdout_of(child("cwd").current_dir(&workdir))); // 출력:     [자식] 현재 디렉터리 이름: "p1_ch10_03_workdir"
    fs::remove_dir(&workdir)?;

    println!("4. 실행하는 세 가지 방법");
    // status: 자식의 표준 입출력을 부모의 것과 공유(inherit)하고, 끝날 때까지 기다립니다.
    //         자식의 출력은 부모를 거치지 않고 바로 터미널로 갑니다.
    println!("  [status]");
    let status = child("hello").status()?;
    println!("    종료 상태: {status}, success() = {}", status.success());
    // 출력:   [status]
    // 출력: [자식] 표준 출력으로 한 줄      ← 자식이 직접 쓴 줄 (들여쓰기 없음)
    // 출력: [자식] 표준 에러로 한 줄        ← 표준 에러라서 터미널에는 보이지만 > 리다이렉션에는 걸리지 않습니다
    // 출력:     종료 상태: exit status: 0, success() = true

    // output: 표준 출력과 표준 에러를 파이프로 받아 Vec<u8>에 모읍니다. 표준 입력은 빈 입력입니다.
    println!("  [output]");
    let output = child("hello").output()?;
    println!("    status = {}", output.status); // 출력:     status = exit status: 0
    println!("    stdout = {:?}", String::from_utf8_lossy(&output.stdout)); // 출력:     stdout = "[자식] 표준 출력으로 한 줄\n"
    println!("    stderr = {:?}", String::from_utf8_lossy(&output.stderr)); // 출력:     stderr = "[자식] 표준 에러로 한 줄\n"

    // spawn: 자식을 시작하자마자 Child를 돌려줍니다. 기다리는 것은 wait로 따로 합니다.
    println!("  [spawn]");
    let mut running = child("fail").spawn()?;
    // id()는 운영체제가 준 프로세스 ID입니다. 값은 실행할 때마다 다르므로 출력하지 않습니다.
    println!("    spawn이 돌아옴, 자식 PID가 있음: {}", running.id() > 0); // 출력:     spawn이 돌아옴, 자식 PID가 있음: true
    let status = running.wait()?;
    println!("    wait 결과: {status}, code() = {:?}", status.code()); // 출력:     wait 결과: exit status: 3, code() = Some(3)

    println!("5. 실행 실패(Err)와 자식의 실패(Ok + 0이 아닌 종료 상태)는 다릅니다");
    match Command::new("p1-ch10-no-such-program").status() {
        Ok(status) => println!("    실행됨: {status}"),
        Err(e) => println!("    실행하지 못함: kind = {:?}, {e}", e.kind()),
    }
    // 출력:     실행하지 못함: kind = NotFound, No such file or directory (os error 2)
    let status = child("fail").status()?; // ?를 통과합니다. 자식은 실행됐기 때문입니다.
    println!(
        "    실행됨: success() = {}, code() = {:?}",
        status.success(),
        status.code()
    );
    // 출력:     실행됨: success() = false, code() = Some(3)

    #[cfg(unix)]
    {
        println!("6. (macOS/Linux) 셸 문법은 셸에게 맡겨야 해석됩니다");
        // echo에 "$HOME | tr a-z A-Z"를 인자 하나로 넘기면 글자 그대로 출력됩니다.
        let literal = stdout_of(Command::new("echo").arg("$HOME | tr a-z A-Z"));
        println!("    echo 직접 실행: {literal}"); // 출력:     echo 직접 실행: $HOME | tr a-z A-Z
        // sh -c "<스크립트>"는 셸이 스크립트를 해석하므로 파이프(|)가 동작합니다.
        // 스크립트 문자열에 사용자 입력을 끼워 넣으면 셸 인젝션이 생기므로, 값은 위치 인자($1)로 넘깁니다.
        let piped = stdout_of(
            Command::new("sh")
                .arg("-c")
                .arg(r#"printf '%s\n' "$1" | tr a-z A-Z"#)
                .arg("sh") // $0 자리
                .arg("hello; echo injected"), // $1 자리: 셸이 명령으로 해석하지 않는 데이터입니다
        );
        println!("    sh -c 실행: {piped}"); // 출력:     sh -c 실행: HELLO; ECHO INJECTED
    }

    Ok(())
}
