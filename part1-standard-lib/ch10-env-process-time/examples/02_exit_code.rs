// 프로세스가 끝나면서 돌려주는 것: 종료 코드(exit code)
//
//   1) main의 반환 타입마다 종료 코드가 어떻게 정해지는지: (), Result, ExitCode, Result<ExitCode, E>
//      main이 돌려준 값은 표준 라이브러리가 Termination::report()로 ExitCode로 바꿔 운영체제에 넘깁니다.
//   2) process::exit는 그 자리에서 끝나므로 소멸자(Drop)가 실행되지 않습니다.
//      BufWriter에 남아 있던 출력이 사라지는 것으로 확인합니다.
//   3) 유닉스에서는 종료 코드의 하위 8비트만 부모에게 전달됩니다.
//
// 실행: cargo run -p p1-ch10-env-process-time --example 02_exit_code
//
// 종료 코드는 프로세스 밖에서만 보이므로, 이 예제는 자기 자신을 "자식 모드"로 다시 실행하고
// 그 결과를 읽어 출력합니다. 자식 프로세스를 실행하는 Command는 예제 03에서 자세히 다룹니다.
// 자식 하나를 직접 실행해 보려면(셸에서 종료 코드 확인):
//   cargo run -q -p p1-ch10-env-process-time --example 02_exit_code -- child code42; echo $?
// 부모(이 예제 자체)는 종료 코드 0으로 끝납니다.

use std::env;
use std::io::{self, BufWriter, Write};
use std::process::{self, Command, ExitCode, Termination};

// 블록을 벗어날 때(drop될 때) 표준 출력에 한 줄을 남기는 값입니다.
struct Cleanup(&'static str);

impl Drop for Cleanup {
    fn drop(&mut self) {
        println!("[drop] {} 정리", self.0);
    }
}

// ---- 자식 모드: 모드마다 "main이 이런 값을 돌려줬다면"을 흉내 냅니다 ----

fn main_returns_ok() -> Result<(), &'static str> {
    Ok(())
}

fn main_returns_err() -> Result<(), &'static str> {
    Err("port 설정이 없습니다")
}

fn main_returns_nested() -> Result<ExitCode, &'static str> {
    // Result<T, E>는 T가 Termination이기만 하면 됩니다. Ok 안의 ExitCode가 그대로 쓰입니다.
    Ok(ExitCode::from(2))
}

fn child_main(mode: &str) -> ExitCode {
    // 실제 main에서는 표준 라이브러리가 해 주는 report() 호출을 여기서는 직접 합니다.
    match mode {
        "unit" => ().report(), // fn main() { ... }는 ()를 돌려주는 main입니다
        "ok" => main_returns_ok().report(),
        "err" => main_returns_err().report(), // Err면 "Error: {:?}"를 표준 에러에 쓰고 FAILURE
        "nested" => main_returns_nested().report(),
        "code42" => ExitCode::from(42), // From<u8>: 0~255만 만들 수 있습니다
        "failure" => ExitCode::FAILURE,
        "return3" => {
            let _guard = Cleanup("return");
            let mut out = BufWriter::new(io::stdout());
            writeln!(out, "BufWriter에 쓴 줄").unwrap();
            // 함수를 정상적으로 빠져나가므로 out(BufWriter)과 _guard가 차례로 drop됩니다.
            // BufWriter는 drop될 때 버퍼를 비우므로 줄이 출력됩니다.
            ExitCode::from(3)
        }
        "exit3" => {
            let _guard = Cleanup("exit");
            let mut out = BufWriter::new(io::stdout());
            writeln!(out, "BufWriter에 쓴 줄").unwrap();
            // process::exit는 돌아오지 않습니다(반환 타입 !). 스택의 값이 drop되지 않으므로
            // [drop] 줄도, BufWriter에 남아 있던 줄도 나오지 않습니다.
            process::exit(3)
        }
        "exit256" => process::exit(256), // 유닉스에서는 하위 8비트(0)만 전달됩니다
        "exit-1" => process::exit(-1),   // 하위 8비트는 255입니다
        "panic" => panic!("일부러 일으킨 패닉"),
        other => {
            eprintln!("알 수 없는 모드: {other}");
            ExitCode::from(64)
        }
    }
}

// ---- 부모: 자식을 실행하고 종료 코드와 출력을 모읍니다 ----

fn run_child(mode: &str) -> String {
    let exe = env::current_exe().expect("실행 파일 경로를 알 수 있어야 합니다");
    // output()은 자식이 끝날 때까지 기다리고 종료 상태·표준 출력·표준 에러를 모아 돌려줍니다.
    let output = Command::new(exe)
        .args(["child", mode])
        .output()
        .expect("자식 프로세스를 실행할 수 있어야 합니다");
    let code = match output.status.code() {
        Some(code) => code.to_string(),
        None => "없음(시그널로 끝남)".to_string(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut report = format!("종료 코드 {code}");
    if !stdout.is_empty() {
        report += &format!(", 표준 출력 {:?}", stdout.trim_end());
    }
    // 표준 에러는 비어 있지 않은 첫 줄만 보여 줍니다(패닉 메시지는 빈 줄로 시작합니다).
    // 패닉 메시지에는 실행마다 다른 스레드 ID가 들어 있으므로 앞부분만 남깁니다.
    if let Some(first) = stderr.lines().find(|line| !line.is_empty()) {
        let shown = if first.starts_with("thread 'main'") {
            "thread 'main' (…) panicked at …"
        } else {
            first
        };
        report += &format!(", 표준 에러 {shown:?}");
    }
    report
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[1] == "child" {
        return child_main(&args[2]);
    }

    println!("1. main이 돌려준 값과 종료 코드");
    for (label, mode) in [
        ("()", "unit"),
        ("Ok::<(), &str>(())", "ok"),
        ("Err::<(), &str>(\"..\")", "err"),
        ("Ok::<ExitCode, &str>(2)", "nested"),
        ("ExitCode::from(42)", "code42"),
        ("ExitCode::FAILURE", "failure"),
        ("panic!(\"..\")", "panic"),
    ] {
        println!("    {label:<24} → {}", run_child(mode));
    }
    // 출력:     ()                       → 종료 코드 0
    // 출력:     Ok::<(), &str>(())       → 종료 코드 0
    // 출력:     Err::<(), &str>("..")    → 종료 코드 1, 표준 에러 "Error: \"port 설정이 없습니다\""
    // 출력:     Ok::<ExitCode, &str>(2)  → 종료 코드 2
    // 출력:     ExitCode::from(42)       → 종료 코드 42
    // 출력:     ExitCode::FAILURE        → 종료 코드 1
    // 출력:     panic!("..")             → 종료 코드 101, 표준 에러 "thread 'main' (…) panicked at …"

    println!("2. 돌려주며 끝내기와 process::exit의 차이");
    println!("    return ExitCode::from(3) → {}", run_child("return3"));
    println!("    process::exit(3)         → {}", run_child("exit3"));
    // 출력:     return ExitCode::from(3) → 종료 코드 3, 표준 출력 "BufWriter에 쓴 줄\n[drop] return 정리"
    // 출력:     process::exit(3)         → 종료 코드 3

    println!("3. 유닉스에서는 하위 8비트만 전달됩니다");
    println!("    process::exit(256) → {}", run_child("exit256"));
    println!("    process::exit(-1)  → {}", run_child("exit-1"));
    // 출력:     process::exit(256) → 종료 코드 0
    // 출력:     process::exit(-1)  → 종료 코드 255

    // ExitCode는 값을 꺼내 볼 수 없고 비교만 됩니다(PartialEq).
    println!("4. ExitCode는 비교만 할 수 있습니다");
    println!(
        "    ExitCode::default() == SUCCESS: {}",
        ExitCode::default() == ExitCode::SUCCESS
    ); // 출력:     ExitCode::default() == SUCCESS: true
    println!(
        "    ExitCode::from(1) == FAILURE: {}",
        ExitCode::from(1) == ExitCode::FAILURE
    ); // 출력:     ExitCode::from(1) == FAILURE: true

    ExitCode::SUCCESS
}
